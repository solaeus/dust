use std::io::{stdin, stdout, Write};
use std::ops::Range;

use crate::vm::{Register, ThreadSignal};
use crate::{vm::Record, ConcreteValue, NativeFunctionError, Value};

pub fn read_line(
    record: &mut Record,
    destination: Option<u8>,
    _argument_range: Range<u8>,
) -> Result<ThreadSignal, NativeFunctionError> {
    let destination = destination.unwrap();
    let mut buffer = String::new();

    match stdin().read_line(&mut buffer) {
        Ok(_) => {
            let length = buffer.len();

            buffer.truncate(length.saturating_sub(1));

            let register = Register::Value(Value::Concrete(ConcreteValue::string(buffer)));

            record.set_register(destination, register);
        }
        Err(error) => {
            return Err(NativeFunctionError::Io {
                error: error.kind(),
                position: record.current_position(),
            })
        }
    }

    Ok(ThreadSignal::Continue)
}

pub fn write(
    record: &mut Record,
    _destination: Option<u8>,
    argument_range: Range<u8>,
) -> Result<ThreadSignal, NativeFunctionError> {
    let mut stdout = stdout();

    for register_index in argument_range {
        if let Some(value) = record.open_register_allow_empty(register_index) {
            let string = value.display(record);

            stdout
                .write(string.as_bytes())
                .map_err(|io_error| NativeFunctionError::Io {
                    error: io_error.kind(),
                    position: record.current_position(),
                })?;
        }
    }

    stdout.flush().map_err(|io_error| NativeFunctionError::Io {
        error: io_error.kind(),
        position: record.current_position(),
    })?;

    Ok(ThreadSignal::Continue)
}

pub fn write_line(
    record: &mut Record,
    _destination: Option<u8>,
    argument_range: Range<u8>,
) -> Result<ThreadSignal, NativeFunctionError> {
    let mut stdout = stdout().lock();

    for register_index in argument_range {
        if let Some(value) = record.open_register_allow_empty(register_index) {
            let string = value.display(record);

            stdout
                .write(string.as_bytes())
                .map_err(|io_error| NativeFunctionError::Io {
                    error: io_error.kind(),
                    position: record.current_position(),
                })?;
            stdout
                .write(b"\n")
                .map_err(|io_error| NativeFunctionError::Io {
                    error: io_error.kind(),
                    position: record.current_position(),
                })?;
        }
    }

    stdout.flush().map_err(|io_error| NativeFunctionError::Io {
        error: io_error.kind(),
        position: record.current_position(),
    })?;

    Ok(ThreadSignal::Continue)
}

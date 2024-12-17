use std::io::{stdin, stdout, Write};

use smallvec::SmallVec;

use crate::{vm::Record, ConcreteValue, NativeFunctionError, Value};

pub fn read_line(
    record: &mut Record,
    _: SmallVec<[&Value; 4]>,
) -> Result<Option<Value>, NativeFunctionError> {
    let mut buffer = String::new();

    match stdin().read_line(&mut buffer) {
        Ok(_) => {
            let length = buffer.len();

            buffer.truncate(length.saturating_sub(1));

            Ok(Some(Value::Concrete(ConcreteValue::string(buffer))))
        }
        Err(error) => Err(NativeFunctionError::Io {
            error: error.kind(),
        }),
    }
}

pub fn write(
    record: &mut Record,
    arguments: SmallVec<[&Value; 4]>,
) -> Result<Option<Value>, NativeFunctionError> {
    let mut stdout = stdout();

    for argument in arguments {
        let string = argument.display(record);

        stdout
            .write_all(string.as_bytes())
            .map_err(|io_error| NativeFunctionError::Io {
                error: io_error.kind(),
            })?;
    }

    Ok(None)
}

pub fn write_line(
    record: &mut Record,
    arguments: SmallVec<[&Value; 4]>,
) -> Result<Option<Value>, NativeFunctionError> {
    let mut stdout = stdout();

    for argument in arguments {
        let string = argument.display(record);

        stdout
            .write_all(string.as_bytes())
            .map_err(|io_error| NativeFunctionError::Io {
                error: io_error.kind(),
            })?;
    }

    stdout
        .write(b"\n")
        .map_err(|io_error| NativeFunctionError::Io {
            error: io_error.kind(),
        })?;

    Ok(None)
}

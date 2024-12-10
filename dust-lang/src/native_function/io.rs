use std::io::{stdin, stdout, Write};

use smallvec::SmallVec;

use crate::{ConcreteValue, NativeFunctionError, Value, ValueRef, Vm};

pub fn read_line(
    vm: &Vm,
    _: SmallVec<[ValueRef; 4]>,
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
            position: vm.current_position(),
        }),
    }
}

pub fn write(
    vm: &Vm,
    arguments: SmallVec<[ValueRef; 4]>,
) -> Result<Option<Value>, NativeFunctionError> {
    let mut stdout = stdout();

    for argument in arguments {
        let string = match argument.display(vm) {
            Ok(string) => string,
            Err(error) => return Err(NativeFunctionError::Vm(Box::new(error))),
        };

        stdout
            .write_all(string.as_bytes())
            .map_err(|io_error| NativeFunctionError::Io {
                error: io_error.kind(),
                position: vm.current_position(),
            })?;
    }

    Ok(None)
}

pub fn write_line(
    vm: &Vm,
    arguments: SmallVec<[ValueRef; 4]>,
) -> Result<Option<Value>, NativeFunctionError> {
    let mut stdout = stdout();

    for argument in arguments {
        let string = match argument.display(vm) {
            Ok(string) => string,
            Err(error) => return Err(NativeFunctionError::Vm(Box::new(error))),
        };

        stdout
            .write_all(string.as_bytes())
            .map_err(|io_error| NativeFunctionError::Io {
                error: io_error.kind(),
                position: vm.current_position(),
            })?;
    }

    stdout
        .write(b"\n")
        .map_err(|io_error| NativeFunctionError::Io {
            error: io_error.kind(),
            position: vm.current_position(),
        })?;

    Ok(None)
}

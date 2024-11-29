use std::io::{self, stdout, Write};

use crate::{ConcreteValue, Instruction, NativeFunctionError, Value, Vm, VmError};

pub fn panic<'a>(vm: &'a Vm<'a>, instruction: Instruction) -> Result<Option<Value>, VmError> {
    let argument_count = instruction.c();
    let message = if argument_count == 0 {
        None
    } else {
        let mut message = String::new();

        for argument_index in 0..argument_count {
            if argument_index != 0 {
                message.push(' ');
            }

            let argument = if let Some(value) = vm.open_register_allow_empty(argument_index)? {
                value
            } else {
                continue;
            };
            let argument_string = argument.display(vm)?;

            message.push_str(&argument_string);
        }

        Some(message)
    };

    Err(VmError::NativeFunction(NativeFunctionError::Panic {
        message,
        position: vm.current_position(),
    }))
}

pub fn to_string<'a>(vm: &'a Vm<'a>, instruction: Instruction) -> Result<Option<Value>, VmError> {
    let argument_count = instruction.c();

    if argument_count != 1 {
        return Err(VmError::NativeFunction(
            NativeFunctionError::ExpectedArgumentCount {
                expected: 1,
                found: argument_count as usize,
                position: vm.current_position(),
            },
        ));
    }

    let mut string = String::new();

    for argument_index in 0..argument_count {
        let argument = if let Some(value) = vm.open_register_allow_empty(argument_index)? {
            value
        } else {
            continue;
        };
        let argument_string = argument.display(vm)?;

        string.push_str(&argument_string);
    }

    Ok(Some(Value::Concrete(ConcreteValue::String(string))))
}

pub fn read_line<'a>(vm: &'a Vm<'a>, instruction: Instruction) -> Result<Option<Value>, VmError> {
    let argument_count = instruction.c();

    if argument_count != 0 {
        return Err(VmError::NativeFunction(
            NativeFunctionError::ExpectedArgumentCount {
                expected: 0,
                found: argument_count as usize,
                position: vm.current_position(),
            },
        ));
    }

    let mut buffer = String::new();

    match io::stdin().read_line(&mut buffer) {
        Ok(_) => {
            buffer = buffer.trim_end_matches('\n').to_string();

            Ok(Some(Value::Concrete(ConcreteValue::String(buffer))))
        }
        Err(error) => Err(VmError::NativeFunction(NativeFunctionError::Io {
            error: error.kind(),
            position: vm.current_position(),
        })),
    }
}

pub fn write<'a>(vm: &'a Vm<'a>, instruction: Instruction) -> Result<Option<Value>, VmError> {
    let to_register = instruction.a();
    let argument_count = instruction.c();
    let mut stdout = stdout();
    let map_err = |io_error: io::Error| {
        VmError::NativeFunction(NativeFunctionError::Io {
            error: io_error.kind(),
            position: vm.current_position(),
        })
    };

    let first_argument = to_register.saturating_sub(argument_count);

    for argument_index in first_argument..to_register {
        let argument = if let Some(value) = vm.open_register_allow_empty(argument_index)? {
            value
        } else {
            continue;
        };
        let argument_string = argument.display(vm)?;

        stdout
            .write_all(argument_string.as_bytes())
            .map_err(map_err)?;
    }

    Ok(None)
}

pub fn write_line<'a>(vm: &'a Vm<'a>, instruction: Instruction) -> Result<Option<Value>, VmError> {
    let to_register = instruction.a();
    let argument_count = instruction.c();
    let mut stdout = stdout();
    let map_err = |io_error: io::Error| {
        VmError::NativeFunction(NativeFunctionError::Io {
            error: io_error.kind(),
            position: vm.current_position(),
        })
    };

    let first_argument = to_register.saturating_sub(argument_count);

    for argument_index in first_argument..to_register {
        let argument = if let Some(value) = vm.open_register_allow_empty(argument_index)? {
            value
        } else {
            continue;
        };
        let argument_string = argument.display(vm)?;

        stdout
            .write_all(argument_string.as_bytes())
            .map_err(map_err)?;
    }

    stdout.write(b"\n").map_err(map_err)?;

    Ok(None)
}

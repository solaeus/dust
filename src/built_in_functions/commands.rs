use std::process::Command;

use crate::{BuiltInFunction, Error, Map, Result, Type, Value};

pub struct Raw;

impl BuiltInFunction for Raw {
    fn name(&self) -> &'static str {
        "raw"
    }

    fn run(&self, arguments: &[Value], _context: &Map) -> Result<Value> {
        Error::expect_argument_amount(self, 2, arguments.len())?;

        let program = arguments.first().unwrap().as_string()?;
        let command_arguments = arguments.get(1).unwrap().as_list()?;
        let mut command = Command::new(program);

        for argument in command_arguments.items().iter() {
            command.arg(argument.as_string()?);
        }

        let output = command.spawn()?.wait_with_output()?.stdout;

        Ok(Value::String(String::from_utf8(output)?))
    }

    fn r#type(&self) -> crate::Type {
        Type::Function {
            parameter_types: vec![Type::String, Type::List(Box::new(Type::String))],
            return_type: Box::new(Type::String),
        }
    }
}

pub struct Sh;

impl BuiltInFunction for Sh {
    fn name(&self) -> &'static str {
        "sh"
    }

    fn run(&self, arguments: &[Value], _context: &Map) -> Result<Value> {
        let command_text = arguments.first().unwrap().as_string()?;
        let mut command = Command::new("sh");

        command.arg("-c");
        command.arg(command_text);

        let extra_command_text = arguments.get(1).unwrap_or_default().as_option()?;

        if let Some(text) = extra_command_text {
            command.args(["--", text.as_string()?]);
        }

        let output = command.spawn()?.wait_with_output()?.stdout;

        Ok(Value::String(String::from_utf8(output)?))
    }

    fn r#type(&self) -> crate::Type {
        Type::Function {
            parameter_types: vec![Type::String, Type::Option(Box::new(Type::String))],
            return_type: Box::new(Type::String),
        }
    }
}

use std::process::Command;

use crate::{BuiltInFunction, Error, Map, Result, Type, Value};

pub struct Sh;

impl BuiltInFunction for Sh {
    fn name(&self) -> &'static str {
        "sh"
    }

    fn run(&self, arguments: &[Value], _context: &Map) -> Result<Value> {
        Error::expect_argument_amount(self, 1, arguments.len())?;

        let command_text = arguments.first().unwrap().as_string()?;
        let mut command = Command::new("sh");

        for word in command_text.split(' ') {
            command.arg(word);
        }

        let output = command.spawn()?.wait_with_output()?.stdout;

        Ok(Value::String(String::from_utf8(output)?))
    }

    fn r#type(&self) -> crate::Type {
        Type::Function {
            parameter_types: vec![Type::String],
            return_type: Box::new(Type::String),
        }
    }
}

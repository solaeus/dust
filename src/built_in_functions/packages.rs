use std::process::Command;

use crate::{BuiltInFunction, Error, Map, Result, Type, Value};

pub struct InstallPackages;

impl BuiltInFunction for InstallPackages {
    fn name(&self) -> &'static str {
        "install_packages"
    }

    fn run(&self, arguments: &[Value], _context: &Map) -> Result<Value> {
        Error::expect_argument_amount(self, 1, arguments.len())?;

        let mut command = Command::new("sudo");
        let argument_list = arguments.first().unwrap().as_list()?;

        command.args(&["dnf", "-y", "install"]);

        for argument in argument_list.items().iter() {
            command.arg(argument.as_string()?);
        }

        command.spawn()?.wait()?;

        Ok(Value::Option(None))
    }

    fn r#type(&self) -> Type {
        Type::Function {
            parameter_types: vec![Type::List(Box::new(Type::String))],
            return_type: Box::new(Type::Empty),
        }
    }
}

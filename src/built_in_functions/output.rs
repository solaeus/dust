use lazy_static::lazy_static;

use crate::{BuiltInFunction, Error, Map, Result, Type, Value};

lazy_static! {
    static ref OUTPUT_MODE: OutputMode = {
        if let Ok(variable) = std::env::var("DUST_OUTPUT_MODE") {
            if variable == "SILENT" {
                OutputMode::Silent
            } else {
                OutputMode::Normal
            }
        } else {
            OutputMode::Normal
        }
    };
}

pub enum OutputMode {
    Normal,
    Silent,
}

pub struct Output;

impl BuiltInFunction for Output {
    fn name(&self) -> &'static str {
        "output"
    }

    fn run(&self, arguments: &[Value], _context: &Map) -> Result<Value> {
        Error::expect_argument_amount(self, 1, arguments.len())?;

        let value = arguments.first().unwrap();

        if let OutputMode::Normal = *OUTPUT_MODE {
            println!("{value}");
        }

        Ok(Value::default())
    }

    fn r#type(&self) -> Type {
        Type::Function {
            parameter_types: vec![Type::Any],
            return_type: Box::new(Type::None),
        }
    }
}

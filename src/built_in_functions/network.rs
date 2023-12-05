use reqwest::blocking::get;

use crate::{BuiltInFunction, Error, Map, Result, Type, Value};

pub struct Download;

impl BuiltInFunction for Download {
    fn name(&self) -> &'static str {
        "download"
    }

    fn run(&self, arguments: &[Value], _context: &Map) -> Result<Value> {
        Error::expect_argument_amount(self, 1, arguments.len())?;

        let url = arguments.first().unwrap().as_string()?;
        let response = get(url)?;

        Ok(Value::String(response.text()?))
    }

    fn r#type(&self) -> Type {
        Type::Function {
            parameter_types: vec![Type::String],
            return_type: Box::new(Type::String),
        }
    }
}

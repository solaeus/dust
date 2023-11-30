use crate::{BuiltInFunction, Error, Map, Result, Value};

pub struct FromJson;

impl BuiltInFunction for FromJson {
    fn name(&self) -> &'static str {
        "from_json"
    }

    fn run(&self, arguments: &[Value], _context: &Map) -> Result<Value> {
        Error::expect_built_in_function_argument_amount(self, 1, arguments.len())?;

        let json_string = arguments.first().unwrap().as_string()?;
        let value = serde_json::from_str(&json_string)?;

        Ok(value)
    }
}

pub struct ToJson;

impl BuiltInFunction for ToJson {
    fn name(&self) -> &'static str {
        "to_json"
    }

    fn run(&self, arguments: &[Value], _context: &Map) -> Result<Value> {
        Error::expect_built_in_function_argument_amount(self, 1, arguments.len())?;

        let value = arguments.first().unwrap();
        let json_string = serde_json::to_string(&value)?;

        Ok(Value::String(json_string))
    }
}

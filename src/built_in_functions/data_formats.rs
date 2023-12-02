use crate::{BuiltInFunction, Error, Map, Result, Type, TypeDefinition, Value};

pub struct FromJson;

impl BuiltInFunction for FromJson {
    fn name(&self) -> &'static str {
        "from_json"
    }

    fn run(&self, arguments: &[Value], _context: &Map) -> Result<Value> {
        Error::expect_argument_amount(self, 1, arguments.len())?;

        let json_string = arguments.first().unwrap().as_string()?;
        let value = serde_json::from_str(&json_string)?;

        Ok(value)
    }

    fn type_definition(&self) -> crate::TypeDefinition {
        TypeDefinition::new(Type::Function {
            parameter_types: vec![Type::String],
            return_type: Box::new(Type::Any),
        })
    }
}

pub struct ToJson;

impl BuiltInFunction for ToJson {
    fn name(&self) -> &'static str {
        "to_json"
    }

    fn run(&self, arguments: &[Value], _context: &Map) -> Result<Value> {
        Error::expect_argument_amount(self, 1, arguments.len())?;

        let value = arguments.first().unwrap();
        let json_string = serde_json::to_string(&value)?;

        Ok(Value::String(json_string))
    }

    fn type_definition(&self) -> crate::TypeDefinition {
        TypeDefinition::new(Type::Function {
            parameter_types: vec![Type::Any],
            return_type: Box::new(Type::String),
        })
    }
}

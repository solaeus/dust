use crate::{BuiltInFunction, Error, Map, Result, Type, Value};

pub struct TypeFunction;

impl BuiltInFunction for TypeFunction {
    fn name(&self) -> &'static str {
        "type"
    }

    fn run(&self, arguments: &[Value], _context: &Map) -> Result<Value> {
        Error::expect_argument_amount(self, 1, arguments.len())?;

        let type_text = arguments.first().unwrap().r#type().to_string();

        Ok(Value::String(type_text))
    }

    fn r#type(&self) -> Type {
        Type::Function {
            parameter_types: vec![Type::Any],
            return_type: Box::new(Type::String),
        }
    }
}

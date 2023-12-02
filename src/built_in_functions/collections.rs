use crate::{BuiltInFunction, Error, Map, Result, Type, TypeDefinition, Value};

pub struct Length;

impl BuiltInFunction for Length {
    fn name(&self) -> &'static str {
        "length"
    }

    fn run(&self, arguments: &[Value], _context: &Map) -> Result<Value> {
        Error::expect_argument_amount(self, 1, arguments.len())?;

        let length = arguments.first().unwrap().as_list()?.items().len();

        Ok(Value::Integer(length as i64))
    }

    fn type_definition(&self) -> TypeDefinition {
        TypeDefinition::new(Type::Function {
            parameter_types: vec![Type::List(Box::new(Type::Any))],
            return_type: Box::new(Type::Integer),
        })
    }
}

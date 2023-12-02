use crate::{BuiltInFunction, Map, Result, Type, TypeDefinition, Value};

pub struct Output;

impl BuiltInFunction for Output {
    fn name(&self) -> &'static str {
        "output"
    }

    fn run(&self, arguments: &[Value], _context: &Map) -> Result<Value> {
        for argument in arguments {
            println!("{argument}");
        }

        Ok(Value::Empty)
    }

    fn type_definition(&self) -> crate::TypeDefinition {
        TypeDefinition::new(Type::Function {
            parameter_types: vec![Type::Any],
            return_type: Box::new(Type::Empty),
        })
    }
}

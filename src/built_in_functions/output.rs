use crate::{BuiltInFunction, Map, Result, Type, Value};

pub struct Output;

impl BuiltInFunction for Output {
    fn name(&self) -> &'static str {
        "output"
    }

    fn run(&self, arguments: &[Value], _context: &Map) -> Result<Value> {
        for argument in arguments {
            println!("{argument}");
        }

        Ok(Value::Option(None))
    }

    fn r#type(&self) -> Type {
        Type::Function {
            parameter_types: vec![Type::Any],
            return_type: Box::new(Type::Empty),
        }
    }
}

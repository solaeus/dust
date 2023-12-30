use crate::{BuiltInFunction, Error, Map, Result, Type, Value};

pub struct Output;

impl BuiltInFunction for Output {
    fn name(&self) -> &'static str {
        "output"
    }

    fn run(&self, arguments: &[Value], _context: &Map) -> Result<Value> {
        Error::expect_argument_amount(self, 1, arguments.len())?;

        let value = arguments.first().unwrap();

        println!("{value}");

        Ok(Value::default())
    }

    fn r#type(&self) -> Type {
        Type::Function {
            parameter_types: vec![Type::Any],
            return_type: Box::new(Type::None),
        }
    }
}

use std::sync::OnceLock;

use crate::{interpret_with_context, BuiltInFunction, Error, Map, Result, Type, Value};

static STD: OnceLock<Map> = OnceLock::new();

pub struct Std;

impl BuiltInFunction for Std {
    fn name(&self) -> &'static str {
        "std"
    }

    fn run(&self, arguments: &[Value], _context: &Map) -> Result<Value> {
        Error::expect_argument_amount(self, 0, arguments.len())?;

        let std_context = STD.get_or_init(|| {
            let std_source = "say_hi = () <none> { output(hi) }";
            let std_context = Map::new();

            interpret_with_context(std_source, std_context.clone()).unwrap();

            std_context
        });

        Ok(Value::Map(std_context.clone()))
    }

    fn r#type(&self) -> Type {
        Type::Function {
            parameter_types: vec![],
            return_type: Box::new(Type::Map),
        }
    }
}

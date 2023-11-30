use crate::{BuiltInFunction, Map, Result, Value};

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
}

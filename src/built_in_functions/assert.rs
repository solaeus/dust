use crate::{BuiltInFunction, Error, Result, Value};

pub struct Assert;

impl BuiltInFunction for Assert {
    fn name(&self) -> &'static str {
        "assert"
    }

    fn run(&self, arguments: &[Value]) -> Result<Value> {
        for argument in arguments {
            if !argument.as_boolean()? {
                return Err(Error::AssertFailed);
            }
        }

        Ok(Value::Empty)
    }
}

use crate::{BuiltInFunction, Error, Map, Result, Type, Value};

pub struct Assert;

impl BuiltInFunction for Assert {
    fn name(&self) -> &'static str {
        "assert"
    }

    fn run(&self, arguments: &[Value], _context: &Map) -> Result<Value> {
        for argument in arguments {
            if !argument.as_boolean()? {
                return Err(Error::AssertFailed);
            }
        }

        Ok(Value::Option(None))
    }

    fn r#type(&self) -> Type {
        Type::Function {
            parameter_types: vec![Type::Any],
            return_type: Box::new(Type::None),
        }
    }
}

pub struct AssertEqual;

impl BuiltInFunction for AssertEqual {
    fn name(&self) -> &'static str {
        "assert_equal"
    }

    fn run(&self, arguments: &[Value], _context: &Map) -> Result<Value> {
        Error::expect_argument_amount(self, 2, arguments.len())?;

        let left = arguments.get(0).unwrap();
        let right = arguments.get(1).unwrap();

        if left == right {
            Ok(Value::Option(None))
        } else {
            Err(Error::AssertEqualFailed {
                expected: left.clone(),
                actual: right.clone(),
            })
        }
    }

    fn r#type(&self) -> Type {
        Type::Function {
            parameter_types: vec![Type::Any, Type::Any],
            return_type: Box::new(Type::Boolean),
        }
    }
}

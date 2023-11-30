use rand::{random, thread_rng, Rng};

use crate::{BuiltInFunction, Error, Map, Result, Value};

pub struct Random;

impl BuiltInFunction for Random {
    fn name(&self) -> &'static str {
        "random"
    }

    fn run(&self, arguments: &[Value], _context: &Map) -> Result<Value> {
        Error::expect_argument_minimum(self, 2, arguments.len())?;

        let random_index = thread_rng().gen_range(0..arguments.len());
        let random_argument = arguments.get(random_index).unwrap();

        Ok(random_argument.clone())
    }
}

pub struct RandomInteger;

impl BuiltInFunction for RandomInteger {
    fn name(&self) -> &'static str {
        "random_integer"
    }

    fn run(&self, arguments: &[Value], _context: &Map) -> Result<Value> {
        Error::expect_argument_amount(self, 0, arguments.len())?;

        Ok(Value::Integer(random()))
    }
}

pub struct RandomFloat;

impl BuiltInFunction for RandomFloat {
    fn name(&self) -> &'static str {
        "random_float"
    }

    fn run(&self, arguments: &[Value], _context: &Map) -> Result<Value> {
        Error::expect_argument_amount(self, 0, arguments.len())?;

        Ok(Value::Float(random()))
    }
}

pub struct RandomBoolean;

impl BuiltInFunction for RandomBoolean {
    fn name(&self) -> &'static str {
        "random_boolean"
    }

    fn run(&self, arguments: &[Value], _context: &Map) -> Result<Value> {
        Error::expect_argument_amount(self, 0, arguments.len())?;

        Ok(Value::Boolean(random()))
    }
}

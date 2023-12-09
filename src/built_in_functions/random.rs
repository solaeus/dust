use rand::{random, thread_rng, Rng};

use crate::{BuiltInFunction, Error, Map, Result, Type, Value};

pub struct Random;

impl BuiltInFunction for Random {
    fn name(&self) -> &'static str {
        "random"
    }

    fn run(&self, arguments: &[Value], _context: &Map) -> Result<Value> {
        Error::expect_argument_amount(self, 1, arguments.len())?;

        let list = arguments.first().unwrap().as_list()?;
        let items = list.items();
        let random_index = thread_rng().gen_range(0..items.len());
        let random_argument = items.get(random_index).unwrap();

        Ok(random_argument.clone())
    }

    fn r#type(&self) -> Type {
        Type::Function {
            parameter_types: vec![Type::List(Box::new(Type::Any))],
            return_type: Box::new(Type::Any),
        }
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

    fn r#type(&self) -> Type {
        Type::Function {
            parameter_types: Vec::with_capacity(0),
            return_type: Box::new(Type::Integer),
        }
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

    fn r#type(&self) -> Type {
        Type::Function {
            parameter_types: Vec::with_capacity(0),
            return_type: Box::new(Type::Float),
        }
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

    fn r#type(&self) -> Type {
        Type::Function {
            parameter_types: Vec::with_capacity(0),
            return_type: Box::new(Type::Boolean),
        }
    }
}
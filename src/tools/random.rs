use std::convert::TryInto;

use rand::{random, thread_rng, Rng};

use crate::{Error, Result, Tool, ToolInfo, Value, ValueType};

pub struct RandomBoolean;

impl Tool for RandomBoolean {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "random_boolean",
            description: "Create a random boolean.",
            group: "random",
            inputs: vec![ValueType::Empty],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let argument = self.check_type(argument)?;

        if let Value::Empty = argument {
            let boolean = rand::thread_rng().gen();

            Ok(Value::Boolean(boolean))
        } else {
            self.fail(argument)
        }
    }
}

pub struct RandomInteger;

impl Tool for RandomInteger {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "random_integer",
            description: "Create a random integer.",
            group: "random",
            inputs: vec![
                ValueType::Empty,
                ValueType::Integer,
                ValueType::ListExact(vec![ValueType::Integer, ValueType::Integer]),
            ],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        match argument {
            Value::Integer(max) => {
                let integer = rand::thread_rng().gen_range(0..*max);

                Ok(Value::Integer(integer))
            }
            Value::List(min_max) => {
                Error::expect_function_argument_amount(self.info().identifier, min_max.len(), 2)?;

                let min = min_max[0].as_int()?;
                let max = min_max[1].as_int()? + 1;
                let integer = rand::thread_rng().gen_range(min..max);

                Ok(Value::Integer(integer))
            }
            Value::Empty => Ok(crate::Value::Integer(random())),
            _ => self.fail(argument),
        }
    }
}

pub struct RandomString;

impl Tool for RandomString {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "random_string",
            description: "Generate a random string.",
            group: "random",
            inputs: vec![ValueType::Empty, ValueType::Integer],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let argument = self.check_type(argument)?;

        if let Value::Integer(length) = argument {
            let length: usize = length.unsigned_abs().try_into().unwrap_or(0);
            let mut random = String::with_capacity(length);

            for _ in 0..length {
                let random_char = thread_rng().gen_range('A'..='z').to_string();

                random.push_str(&random_char);
            }

            return Ok(Value::String(random));
        }

        if let Value::Empty = argument {
            let mut random = String::with_capacity(10);

            for _ in 0..10 {
                let random_char = thread_rng().gen_range('A'..='z').to_string();

                random.push_str(&random_char);
            }

            return Ok(Value::String(random));
        }

        self.fail(argument)
    }
}

pub struct RandomFloat;

impl Tool for RandomFloat {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "random_float",
            description: "Generate a random floating point value between 0 and 1.",
            group: "random",
            inputs: vec![ValueType::Empty],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let argument = self.check_type(argument)?;

        if argument.is_empty() {
            Ok(Value::Float(random()))
        } else {
            self.fail(argument)
        }
    }
}

pub struct Random;

impl Tool for Random {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "random",
            description: "Select a random item from a list.",
            group: "random",
            inputs: vec![ValueType::List],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        let argument = self.check_type(argument)?;

        if let Value::List(list) = argument {
            let random_index = thread_rng().gen_range(0..list.len());
            let random_item = list.get(random_index).unwrap();

            Ok(random_item.clone())
        } else {
            self.fail(argument)
        }
    }
}

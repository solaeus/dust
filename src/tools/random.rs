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
        match argument {
            Value::Empty => {
                let boolean = rand::thread_rng().gen();

                Ok(Value::Boolean(boolean))
            }
            Value::String(_)
            | Value::Float(_)
            | Value::Integer(_)
            | Value::Boolean(_)
            | Value::List(_)
            | Value::Map(_)
            | Value::Table(_)
            | Value::Time(_)
            | Value::Function(_) => Err(Error::TypeCheckFailure {
                tool_info: self.info(),
                argument: argument.clone(),
            }),
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
                let max = min_max[1].as_int()?;
                let integer = rand::thread_rng().gen_range(min..=max);

                Ok(Value::Integer(integer))
            }
            Value::Empty => Ok(crate::Value::Integer(random())),
            _ => Err(Error::TypeCheckFailure {
                tool_info: self.info(),
                argument: argument.clone(),
            }),
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
        match argument {
            Value::Integer(length) => {
                let length: usize = length.unsigned_abs().try_into().unwrap_or(0);
                let mut random = String::with_capacity(length);

                for _ in 0..length {
                    let random_char = thread_rng().gen_range('A'..='z').to_string();

                    random.push_str(&random_char);
                }

                Ok(Value::String(random))
            }
            Value::Empty => {
                let mut random = String::with_capacity(10);

                for _ in 0..10 {
                    let random_char = thread_rng().gen_range('A'..='z').to_string();

                    random.push_str(&random_char);
                }

                Ok(Value::String(random))
            }
            _ => Err(Error::TypeCheckFailure {
                tool_info: self.info(),
                argument: argument.clone(),
            }),
        }
    }
}

pub struct RandomFloat;

impl Tool for RandomFloat {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "random_float",
            description: "Generate a random floating point value between 0 and 1.",
            group: "random",
            inputs: vec![],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        argument.as_empty()?;

        Ok(Value::Float(random()))
    }
}

pub struct Random;

impl Tool for Random {
    fn info(&self) -> ToolInfo<'static> {
        ToolInfo {
            identifier: "random",
            description: "Select a random item from a collection.",
            group: "random",
            inputs: vec![],
        }
    }

    fn run(&self, argument: &Value) -> Result<Value> {
        if let Ok(list) = argument.as_list() {
            let random_index = thread_rng().gen_range(0..list.len());
            let random_item = list.get(random_index).unwrap();

            Ok(random_item.clone())
        } else {
            Err(Error::ExpectedCollection {
                actual: argument.clone(),
            })
        }
    }
}

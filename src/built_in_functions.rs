use std::fs::read_to_string;

use rand::{random, thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::{Error, Map, Result, Type, Value};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum BuiltInFunction {
    AssertEqual,
    FsRead,
    JsonParse,
    Length,
    Output,
    RandomBoolean,
    RandomFloat,
    RandomFrom,
    RandomInteger,
}

impl BuiltInFunction {
    pub fn name(&self) -> &'static str {
        match self {
            BuiltInFunction::AssertEqual => "assert_equal",
            BuiltInFunction::FsRead => "read",
            BuiltInFunction::JsonParse => "parse",
            BuiltInFunction::Length => "length",
            BuiltInFunction::Output => "output",
            BuiltInFunction::RandomBoolean => "boolean",
            BuiltInFunction::RandomFloat => "float",
            BuiltInFunction::RandomFrom => "from",
            BuiltInFunction::RandomInteger => "integer",
        }
    }

    pub fn r#type(&self) -> Type {
        match self {
            BuiltInFunction::AssertEqual => Type::function(vec![Type::Any, Type::Any], Type::None),
            BuiltInFunction::FsRead => Type::function(vec![Type::String], Type::String),
            BuiltInFunction::JsonParse => Type::function(vec![Type::String], Type::Any),
            BuiltInFunction::Length => Type::function(vec![Type::Collection], Type::Integer),
            BuiltInFunction::Output => Type::function(vec![Type::Any], Type::None),
            BuiltInFunction::RandomBoolean => Type::function(vec![], Type::Boolean),
            BuiltInFunction::RandomFloat => Type::function(vec![], Type::Float),
            BuiltInFunction::RandomFrom => Type::function(vec![Type::Collection], Type::Any),
            BuiltInFunction::RandomInteger => Type::function(vec![], Type::Integer),
        }
    }

    pub fn call(&self, arguments: &[Value], _source: &str, _outer_context: &Map) -> Result<Value> {
        match self {
            BuiltInFunction::AssertEqual => {
                Error::expect_argument_amount(self, 2, arguments.len())?;

                let left = arguments.get(0).unwrap();
                let right = arguments.get(1).unwrap();

                Ok(Value::Boolean(left == right))
            }
            BuiltInFunction::FsRead => {
                Error::expect_argument_amount(self, 1, arguments.len())?;

                let path = arguments.first().unwrap().as_string()?;
                let file_content = read_to_string(path)?;

                Ok(Value::String(file_content))
            }
            BuiltInFunction::JsonParse => {
                Error::expect_argument_amount(self, 1, arguments.len())?;

                let string = arguments.first().unwrap().as_string()?;
                let value = serde_json::from_str(&string)?;

                Ok(value)
            }
            BuiltInFunction::Length => {
                Error::expect_argument_amount(self, 1, arguments.len())?;

                let value = arguments.first().unwrap();
                let length = if let Ok(list) = value.as_list() {
                    list.items().len()
                } else if let Ok(map) = value.as_map() {
                    map.variables()?.len()
                } else if let Ok(string) = value.as_string() {
                    string.chars().count()
                } else {
                    return Err(Error::ExpectedCollection {
                        actual: value.clone(),
                    });
                };

                Ok(Value::Integer(length as i64))
            }
            BuiltInFunction::Output => {
                Error::expect_argument_amount(self, 1, arguments.len())?;

                let value = arguments.first().unwrap();

                println!("{value}");

                Ok(Value::none())
            }
            BuiltInFunction::RandomBoolean => {
                Error::expect_argument_amount(self, 0, arguments.len())?;

                Ok(Value::Boolean(random()))
            }
            BuiltInFunction::RandomFloat => {
                Error::expect_argument_amount(self, 0, arguments.len())?;

                Ok(Value::Float(random()))
            }
            BuiltInFunction::RandomFrom => {
                Error::expect_argument_amount(self, 1, arguments.len())?;

                let value = arguments.first().unwrap();

                if let Ok(list) = value.as_list() {
                    let items = list.items();

                    if items.len() == 0 {
                        Ok(Value::none())
                    } else {
                        let random_index = thread_rng().gen_range(0..items.len());
                        let random_value = items.get(random_index).cloned().unwrap_or_default();

                        Ok(random_value)
                    }
                } else {
                    todo!()
                }
            }
            BuiltInFunction::RandomInteger => {
                Error::expect_argument_amount(self, 0, arguments.len())?;

                Ok(Value::Integer(random()))
            }
        }
    }
}

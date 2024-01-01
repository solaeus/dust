use std::{
    fmt::{self, Display, Formatter},
    fs::read_to_string,
};

use rand::random;
use serde::{Deserialize, Serialize};

use crate::{Error, Map, Result, Type, Value};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum BuiltInFunction {
    AssertEqual,
    FsRead,
    Output,
    RandomBoolean,
    Length,
}

impl BuiltInFunction {
    pub fn name(&self) -> &'static str {
        match self {
            BuiltInFunction::AssertEqual => "assert_equal",
            BuiltInFunction::FsRead => "fs_read",
            BuiltInFunction::Output => "output",
            BuiltInFunction::RandomBoolean => "boolean",
            BuiltInFunction::Length => "length",
        }
    }

    pub fn r#type(&self) -> Type {
        match self {
            BuiltInFunction::AssertEqual => Type::function(vec![Type::Any, Type::Any], Type::None),
            BuiltInFunction::FsRead => Type::function(vec![Type::String], Type::String),
            BuiltInFunction::Output => Type::function(vec![Type::Any], Type::None),
            BuiltInFunction::RandomBoolean => Type::function(vec![], Type::Boolean),
            BuiltInFunction::Length => Type::function(vec![Type::Collection], Type::Integer),
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
        }
    }
}

impl Display for BuiltInFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.r#type())
    }
}

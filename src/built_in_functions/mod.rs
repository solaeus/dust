mod string;

use std::{
    fmt::{self, Display, Formatter},
    fs::read_to_string,
};

use rand::{random, thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::{Error, Format, Map, Result, Type, Value};

pub use string::{string_functions, StringFunction};

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
    String(StringFunction),
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
            BuiltInFunction::String(string_function) => string_function.name(),
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
            BuiltInFunction::String(string_function) => string_function.r#type(),
        }
    }

    pub fn call(&self, arguments: &[Value], _source: &str, _outer_context: &Map) -> Result<Value> {
        match self {
            BuiltInFunction::AssertEqual => {
                Error::expect_argument_amount(self.name(), 2, arguments.len())?;

                let left = arguments.get(0).unwrap();
                let right = arguments.get(1).unwrap();

                Ok(Value::Boolean(left == right))
            }
            BuiltInFunction::FsRead => {
                Error::expect_argument_amount(self.name(), 1, arguments.len())?;

                let path = arguments.first().unwrap().as_string()?;
                let file_content = read_to_string(path.as_str())?;

                Ok(Value::string(file_content))
            }
            BuiltInFunction::JsonParse => {
                Error::expect_argument_amount(self.name(), 1, arguments.len())?;

                let string = arguments.first().unwrap().as_string()?;
                let value = serde_json::from_str(&string)?;

                Ok(value)
            }
            BuiltInFunction::Length => {
                Error::expect_argument_amount(self.name(), 1, arguments.len())?;

                let value = arguments.first().unwrap();
                let length = if let Ok(list) = value.as_list() {
                    list.items().len()
                } else if let Ok(map) = value.as_map() {
                    map.variables()?.len()
                } else if let Ok(str) = value.as_string() {
                    str.chars().count()
                } else {
                    return Err(Error::ExpectedCollection {
                        actual: value.clone(),
                    });
                };

                Ok(Value::Integer(length as i64))
            }
            BuiltInFunction::Output => {
                Error::expect_argument_amount(self.name(), 1, arguments.len())?;

                let value = arguments.first().unwrap();

                println!("{value}");

                Ok(Value::none())
            }
            BuiltInFunction::RandomBoolean => {
                Error::expect_argument_amount(self.name(), 0, arguments.len())?;

                Ok(Value::Boolean(random()))
            }
            BuiltInFunction::RandomFloat => {
                Error::expect_argument_amount(self.name(), 0, arguments.len())?;

                Ok(Value::Float(random()))
            }
            BuiltInFunction::RandomFrom => {
                Error::expect_argument_amount(self.name(), 1, arguments.len())?;

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
                Error::expect_argument_amount(self.name(), 0, arguments.len())?;

                Ok(Value::Integer(random()))
            }
            BuiltInFunction::String(string_function) => {
                string_function.call(arguments, _source, _outer_context)
            }
        }
    }
}

impl Format for BuiltInFunction {
    fn format(&self, output: &mut String, indent_level: u8) {
        output.push_str(self.name());
    }
}

impl Display for BuiltInFunction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

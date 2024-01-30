pub mod fs;
pub mod json;
pub mod str;

use std::fmt::{self, Display, Formatter};

use rand::{random, thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::{Error, Format, Map, Result, Type, Value};

use self::{fs::Fs, json::Json, str::StrFunction};

pub trait Callable {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn r#type(&self) -> Type;
    fn call(&self, arguments: &[Value], source: &str, outer_context: &Map) -> Result<Value>;
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum BuiltInFunction {
    AssertEqual,
    Fs(Fs),
    Json(Json),
    Length,
    Output,
    RandomBoolean,
    RandomFloat,
    RandomFrom,
    RandomInteger,
    String(StrFunction),
}

impl Callable for BuiltInFunction {
    fn name(&self) -> &'static str {
        match self {
            BuiltInFunction::AssertEqual => "assert_equal",
            BuiltInFunction::Fs(fs_function) => fs_function.name(),
            BuiltInFunction::Json(json_function) => json_function.name(),
            BuiltInFunction::Length => "length",
            BuiltInFunction::Output => "output",
            BuiltInFunction::RandomBoolean => "boolean",
            BuiltInFunction::RandomFloat => "float",
            BuiltInFunction::RandomFrom => "from",
            BuiltInFunction::RandomInteger => "integer",
            BuiltInFunction::String(string_function) => string_function.name(),
        }
    }

    fn description(&self) -> &'static str {
        match self {
            BuiltInFunction::AssertEqual => "assert_equal",
            BuiltInFunction::Fs(fs_function) => fs_function.description(),
            BuiltInFunction::Json(json_function) => json_function.description(),
            BuiltInFunction::Length => "length",
            BuiltInFunction::Output => "output",
            BuiltInFunction::RandomBoolean => "boolean",
            BuiltInFunction::RandomFloat => "float",
            BuiltInFunction::RandomFrom => "from",
            BuiltInFunction::RandomInteger => "integer",
            BuiltInFunction::String(string_function) => string_function.description(),
        }
    }

    fn r#type(&self) -> Type {
        match self {
            BuiltInFunction::AssertEqual => Type::function(vec![Type::Any, Type::Any], Type::None),
            BuiltInFunction::Fs(fs_function) => fs_function.r#type(),
            BuiltInFunction::Json(json_function) => json_function.r#type(),
            BuiltInFunction::Length => Type::function(vec![Type::Collection], Type::Integer),
            BuiltInFunction::Output => Type::function(vec![Type::Any], Type::None),
            BuiltInFunction::RandomBoolean => Type::function(vec![], Type::Boolean),
            BuiltInFunction::RandomFloat => Type::function(vec![], Type::Float),
            BuiltInFunction::RandomFrom => Type::function(vec![Type::Collection], Type::Any),
            BuiltInFunction::RandomInteger => Type::function(vec![], Type::Integer),
            BuiltInFunction::String(string_function) => string_function.r#type(),
        }
    }

    fn call(&self, arguments: &[Value], _source: &str, _outer_context: &Map) -> Result<Value> {
        match self {
            BuiltInFunction::AssertEqual => {
                Error::expect_argument_amount(self.name(), 2, arguments.len())?;

                let left = arguments.first().unwrap();
                let right = arguments.get(1).unwrap();

                Ok(Value::Boolean(left == right))
            }
            BuiltInFunction::Fs(fs_function) => {
                fs_function.call(arguments, _source, _outer_context)
            }
            BuiltInFunction::Json(json_function) => {
                json_function.call(arguments, _source, _outer_context)
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
    fn format(&self, output: &mut String, _indent_level: u8) {
        output.push_str(self.name());
    }
}

impl Display for BuiltInFunction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

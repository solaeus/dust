use enum_iterator::{all, Sequence};
use serde::{Deserialize, Serialize};

use crate::{Error, List, Map, Result, Type, Value};

pub fn string_functions() -> impl Iterator<Item = StringFunction> {
    all()
}

#[derive(Sequence, Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum StringFunction {
    AsBytes,
    EndsWith,
    Find,
    IsAscii,
    IsEmpty,
    Lines,
    Matches,
    Split,
    SplitAt,
    SplitInclusive,
    SplitN,
    SplitOnce,
    SplitTerminator,
    SplitWhitespace,
    StartsWith,
    StripPrefix,
    ToLowercase,
    ToUppercase,
    Trim,
    TrimEnd,
    TrimEndMatches,
    TrimLeft,
    TrimLeftMatches,
    TrimMatches,
    TrimRight,
    TrimRightMatches,
    TrimStart,
    TrimStartMatches,
}

impl StringFunction {
    pub fn name(&self) -> &'static str {
        match self {
            StringFunction::AsBytes => "as_bytes",
            StringFunction::EndsWith => "ends_with",
            StringFunction::Find => "find",
            StringFunction::IsAscii => "is_ascii",
            StringFunction::IsEmpty => "is_empty",
            StringFunction::Lines => "lines",
            StringFunction::Matches => "matches",
            StringFunction::Split => "split",
            StringFunction::SplitAt => "split_at",
            StringFunction::SplitInclusive => "split_inclusive",
            StringFunction::SplitN => "split_n",
            StringFunction::SplitOnce => "split_once",
            StringFunction::SplitTerminator => "split_terminator",
            StringFunction::SplitWhitespace => "split_whitespace",
            StringFunction::StartsWith => "starts_with",
            StringFunction::StripPrefix => "strip_prefix",
            StringFunction::ToLowercase => "to_lowercase",
            StringFunction::ToUppercase => "to_uppercase",
            StringFunction::Trim => "trim",
            StringFunction::TrimEnd => "trim_end",
            StringFunction::TrimEndMatches => "trim_end_matches",
            StringFunction::TrimLeft => "trim_left",
            StringFunction::TrimLeftMatches => "trim_left_matches",
            StringFunction::TrimMatches => "trim_matches",
            StringFunction::TrimRight => "trim_right",
            StringFunction::TrimRightMatches => "trim_right_matches",
            StringFunction::TrimStart => "trim_start",
            StringFunction::TrimStartMatches => "trim_start_matches",
        }
    }

    pub fn r#type(&self) -> Type {
        match self {
            StringFunction::AsBytes => {
                Type::function(vec![Type::String], Type::list(Type::Integer))
            }
            StringFunction::EndsWith => {
                Type::function(vec![Type::String, Type::String], Type::Boolean)
            }
            StringFunction::Find => Type::function(
                vec![Type::String, Type::String],
                Type::option(Type::Integer),
            ),
            StringFunction::IsAscii => Type::function(vec![Type::String], Type::Boolean),
            StringFunction::IsEmpty => Type::function(vec![Type::String], Type::Boolean),
            StringFunction::Lines => Type::function(vec![Type::String], Type::list(Type::String)),
            StringFunction::Matches => {
                Type::function(vec![Type::String, Type::String], Type::list(Type::String))
            }
            StringFunction::Split => {
                Type::function(vec![Type::String, Type::String], Type::list(Type::String))
            }
            StringFunction::SplitAt => {
                Type::function(vec![Type::String, Type::Integer], Type::list(Type::String))
            }
            StringFunction::SplitInclusive => {
                Type::function(vec![Type::String, Type::String], Type::list(Type::String))
            }
            StringFunction::SplitN => Type::function(
                vec![Type::String, Type::Integer, Type::String],
                Type::list(Type::String),
            ),
            StringFunction::SplitOnce => {
                Type::function(vec![Type::String, Type::String], Type::list(Type::String))
            }
            StringFunction::SplitTerminator => {
                Type::function(vec![Type::String, Type::String], Type::list(Type::String))
            }
            StringFunction::SplitWhitespace => {
                Type::function(vec![Type::String], Type::list(Type::String))
            }
            StringFunction::StartsWith => {
                Type::function(vec![Type::String, Type::String], Type::Boolean)
            }
            StringFunction::StripPrefix => {
                Type::function(vec![Type::String, Type::String], Type::option(Type::String))
            }
            StringFunction::ToLowercase => Type::function(vec![Type::String], Type::String),
            StringFunction::ToUppercase => Type::function(vec![Type::String], Type::String),
            StringFunction::Trim => Type::function(vec![Type::String], Type::String),
            StringFunction::TrimEnd => Type::function(vec![Type::String], Type::String),
            StringFunction::TrimEndMatches => {
                Type::function(vec![Type::String, Type::String], Type::String)
            }
            StringFunction::TrimLeft => Type::function(vec![Type::String], Type::String),
            StringFunction::TrimLeftMatches => {
                Type::function(vec![Type::String, Type::String], Type::String)
            }
            StringFunction::TrimMatches => {
                Type::function(vec![Type::String, Type::String], Type::String)
            }
            StringFunction::TrimRight => Type::function(vec![Type::String], Type::String),
            StringFunction::TrimRightMatches => {
                Type::function(vec![Type::String, Type::String], Type::String)
            }
            StringFunction::TrimStart => Type::function(vec![Type::String], Type::String),
            StringFunction::TrimStartMatches => {
                Type::function(vec![Type::String, Type::String], Type::String)
            }
        }
    }

    pub fn call(&self, arguments: &[Value], _source: &str, _outer_context: &Map) -> Result<Value> {
        match self {
            StringFunction::AsBytes => {
                Error::expect_argument_amount(self.name(), 1, arguments.len())?;

                let string = arguments.first().unwrap().as_string()?;
                let bytes = string
                    .bytes()
                    .map(|byte| Value::Integer(byte as i64))
                    .collect();

                Ok(Value::List(List::with_items(bytes)))
            }
            StringFunction::EndsWith => {
                Error::expect_argument_amount(self.name(), 2, arguments.len())?;

                let string = arguments.get(0).unwrap().as_string()?;
                let pattern = arguments.get(1).unwrap().as_string()?;

                Ok(Value::Boolean(string.ends_with(pattern)))
            }
            StringFunction::Find => {
                Error::expect_argument_amount(self.name(), 2, arguments.len())?;

                let string = arguments.get(0).unwrap().as_string()?;
                let pattern = arguments.get(1).unwrap().as_string()?;
                let find = string
                    .find(pattern)
                    .map(|index| Box::new(Value::Integer(index as i64)));

                Ok(Value::Option(find))
            }
            StringFunction::IsAscii => todo!(),
            StringFunction::IsEmpty => todo!(),
            StringFunction::Lines => todo!(),
            StringFunction::Matches => todo!(),
            StringFunction::Split => todo!(),
            StringFunction::SplitAt => todo!(),
            StringFunction::SplitInclusive => todo!(),
            StringFunction::SplitN => todo!(),
            StringFunction::SplitOnce => todo!(),
            StringFunction::SplitTerminator => todo!(),
            StringFunction::SplitWhitespace => todo!(),
            StringFunction::StartsWith => todo!(),
            StringFunction::StripPrefix => todo!(),
            StringFunction::ToLowercase => todo!(),
            StringFunction::ToUppercase => todo!(),
            StringFunction::Trim => {
                Error::expect_argument_amount(self.name(), 1, arguments.len())?;

                let trimmed = arguments.first().unwrap().as_string()?.trim().to_string();

                Ok(Value::String(trimmed))
            }
            StringFunction::TrimEnd => todo!(),
            StringFunction::TrimEndMatches => todo!(),
            StringFunction::TrimLeft => todo!(),
            StringFunction::TrimLeftMatches => todo!(),
            StringFunction::TrimMatches => todo!(),
            StringFunction::TrimRight => todo!(),
            StringFunction::TrimRightMatches => todo!(),
            StringFunction::TrimStart => todo!(),
            StringFunction::TrimStartMatches => todo!(),
        }
    }
}

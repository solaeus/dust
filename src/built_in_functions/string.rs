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
    TrimMatches,
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
            StringFunction::TrimMatches => "trim_matches",
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
            StringFunction::TrimMatches => {
                Type::function(vec![Type::String, Type::String], Type::String)
            }
            StringFunction::TrimStart => Type::function(vec![Type::String], Type::String),
            StringFunction::TrimStartMatches => {
                Type::function(vec![Type::String, Type::String], Type::String)
            }
        }
    }

    pub fn call(&self, arguments: &[Value], _source: &str, _outer_context: &Map) -> Result<Value> {
        let value = match self {
            StringFunction::AsBytes => {
                Error::expect_argument_amount(self.name(), 1, arguments.len())?;

                let string = arguments.first().unwrap().as_string()?;
                let bytes = string
                    .bytes()
                    .map(|byte| Value::Integer(byte as i64))
                    .collect();

                Value::List(List::with_items(bytes))
            }
            StringFunction::EndsWith => {
                Error::expect_argument_amount(self.name(), 2, arguments.len())?;

                let string = arguments.get(0).unwrap().as_string()?;
                let pattern = arguments.get(1).unwrap().as_string()?;

                Value::Boolean(string.ends_with(pattern))
            }
            StringFunction::Find => {
                Error::expect_argument_amount(self.name(), 2, arguments.len())?;

                let string = arguments.get(0).unwrap().as_string()?;
                let pattern = arguments.get(1).unwrap().as_string()?;
                let find = string
                    .find(pattern)
                    .map(|index| Box::new(Value::Integer(index as i64)));

                Value::Option(find)
            }
            StringFunction::IsAscii => {
                Error::expect_argument_amount(self.name(), 1, arguments.len())?;

                let string = arguments.first().unwrap().as_string()?;

                Value::Boolean(string.is_ascii())
            }
            StringFunction::IsEmpty => {
                Error::expect_argument_amount(self.name(), 1, arguments.len())?;

                let string = arguments.first().unwrap().as_string()?;

                Value::Boolean(string.is_empty())
            }
            StringFunction::Lines => {
                Error::expect_argument_amount(self.name(), 1, arguments.len())?;

                let string = arguments.first().unwrap().as_string()?;
                let lines = string
                    .lines()
                    .map(|line| Value::String(line.to_string()))
                    .collect();

                Value::List(List::with_items(lines))
            }
            StringFunction::Matches => {
                Error::expect_argument_amount(self.name(), 2, arguments.len())?;

                let string = arguments.get(0).unwrap().as_string()?;
                let pattern = arguments.get(1).unwrap().as_string()?;
                let matches = string
                    .matches(pattern)
                    .map(|pattern| Value::String(pattern.to_string()))
                    .collect();

                Value::List(List::with_items(matches))
            }
            StringFunction::Split => {
                Error::expect_argument_amount(self.name(), 2, arguments.len())?;

                let string = arguments.get(0).unwrap().as_string()?;
                let pattern = arguments.get(1).unwrap().as_string()?;
                let sections = string
                    .split(pattern)
                    .map(|section| Value::String(section.to_string()))
                    .collect();

                Value::List(List::with_items(sections))
            }
            StringFunction::SplitAt => {
                Error::expect_argument_amount(self.name(), 2, arguments.len())?;

                let string = arguments.get(0).unwrap().as_string()?;
                let index = arguments.get(1).unwrap().as_integer()?;
                let (left, right) = string.split_at(index as usize);

                Value::List(List::with_items(vec![
                    Value::String(left.to_string()),
                    Value::String(right.to_string()),
                ]))
            }
            StringFunction::SplitInclusive => {
                Error::expect_argument_amount(self.name(), 2, arguments.len())?;

                let string = arguments.get(0).unwrap().as_string()?;
                let pattern = arguments.get(1).unwrap().as_string()?;
                let sections = string
                    .split(pattern)
                    .map(|pattern| Value::String(pattern.to_string()))
                    .collect();

                Value::List(List::with_items(sections))
            }
            StringFunction::SplitN => {
                Error::expect_argument_amount(self.name(), 3, arguments.len())?;

                let string = arguments.get(0).unwrap().as_string()?;
                let count = arguments.get(1).unwrap().as_integer()?;
                let pattern = arguments.get(2).unwrap().as_string()?;
                let sections = string
                    .splitn(count as usize, pattern)
                    .map(|pattern| Value::String(pattern.to_string()))
                    .collect();

                Value::List(List::with_items(sections))
            }
            StringFunction::SplitOnce => {
                Error::expect_argument_amount(self.name(), 2, arguments.len())?;

                let string = arguments.get(0).unwrap().as_string()?;
                let pattern = arguments.get(1).unwrap().as_string()?;
                let sections = string.split_once(pattern).map(|(left, right)| {
                    Value::List(List::with_items(vec![
                        Value::String(left.to_string()),
                        Value::String(right.to_string()),
                    ]))
                });

                Value::option(sections)
            }
            StringFunction::SplitTerminator => {
                Error::expect_argument_amount(self.name(), 2, arguments.len())?;

                let string = arguments.get(0).unwrap().as_string()?;
                let pattern = arguments.get(1).unwrap().as_string()?;
                let sections = string
                    .split_terminator(pattern)
                    .map(|section| Value::String(section.to_string()))
                    .collect();

                Value::List(List::with_items(sections))
            }
            StringFunction::SplitWhitespace => {
                Error::expect_argument_amount(self.name(), 1, arguments.len())?;

                let string = arguments.get(0).unwrap().as_string()?;
                let sections = string
                    .split_whitespace()
                    .map(|section| Value::String(section.to_string()))
                    .collect();

                Value::List(List::with_items(sections))
            }
            StringFunction::StartsWith => {
                Error::expect_argument_amount(self.name(), 2, arguments.len())?;

                let string = arguments.get(0).unwrap().as_string()?;
                let pattern = arguments.get(1).unwrap().as_string()?;

                Value::Boolean(string.starts_with(pattern))
            }
            StringFunction::StripPrefix => {
                Error::expect_argument_amount(self.name(), 2, arguments.len())?;

                let string = arguments.get(0).unwrap().as_string()?;
                let prefix = arguments.get(1).unwrap().as_string()?;
                let stripped = string
                    .strip_prefix(prefix)
                    .map(|remainder| Value::String(remainder.to_string()));

                Value::option(stripped)
            }
            StringFunction::ToLowercase => {
                Error::expect_argument_amount(self.name(), 1, arguments.len())?;

                let string = arguments.get(0).unwrap().as_string()?;
                let lowercase = string.to_lowercase();

                Value::String(lowercase)
            }
            StringFunction::ToUppercase => {
                Error::expect_argument_amount(self.name(), 1, arguments.len())?;

                let string = arguments.get(0).unwrap().as_string()?;
                let uppercase = string.to_uppercase();

                Value::String(uppercase)
            }
            StringFunction::Trim => {
                Error::expect_argument_amount(self.name(), 1, arguments.len())?;

                let trimmed = arguments.first().unwrap().as_string()?.trim().to_string();

                Value::String(trimmed)
            }
            StringFunction::TrimEnd => {
                Error::expect_argument_amount(self.name(), 1, arguments.len())?;

                let trimmed = arguments
                    .first()
                    .unwrap()
                    .as_string()?
                    .trim_end()
                    .to_string();

                Value::String(trimmed)
            }
            StringFunction::TrimEndMatches => {
                Error::expect_argument_amount(self.name(), 2, arguments.len())?;

                let string = arguments.get(0).unwrap().as_string()?;
                let pattern = arguments.get(1).unwrap().as_string()?;
                let trimmed = string.trim_end_matches(pattern).to_string();

                Value::String(trimmed)
            }
            StringFunction::TrimMatches => {
                Error::expect_argument_amount(self.name(), 2, arguments.len())?;

                let string = arguments.get(0).unwrap().as_string()?;
                let pattern = arguments
                    .get(1)
                    .unwrap()
                    .as_string()?
                    .chars()
                    .collect::<Vec<char>>();
                let trimmed = string.trim_matches(pattern.as_slice()).to_string();

                Value::String(trimmed)
            }
            StringFunction::TrimStart => {
                Error::expect_argument_amount(self.name(), 1, arguments.len())?;

                let trimmed = arguments
                    .first()
                    .unwrap()
                    .as_string()?
                    .trim_start()
                    .to_string();

                Value::String(trimmed)
            }
            StringFunction::TrimStartMatches => todo!(),
        };

        Ok(value)
    }
}

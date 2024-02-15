use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};

use crate::{error::RuntimeError, Context, EnumInstance, Identifier, List, Type, Value};

use super::Callable;

pub fn string_functions() -> impl Iterator<Item = StrFunction> {
    enum_iterator::all()
}

#[derive(Sequence, Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum StrFunction {
    AsBytes,
    EndsWith,
    Find,
    Insert,
    IsAscii,
    IsEmpty,
    Lines,
    Matches,
    Parse,
    Remove,
    ReplaceRange,
    Retain,
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
    Truncate,
}

impl Callable for StrFunction {
    fn name(&self) -> &'static str {
        match self {
            StrFunction::AsBytes => "as_bytes",
            StrFunction::EndsWith => "ends_with",
            StrFunction::Find => "find",
            StrFunction::Insert => "insert",
            StrFunction::IsAscii => "is_ascii",
            StrFunction::IsEmpty => "is_empty",
            StrFunction::Lines => "lines",
            StrFunction::Matches => "matches",
            StrFunction::Parse => "parse",
            StrFunction::Remove => "remove",
            StrFunction::ReplaceRange => "replace_range",
            StrFunction::Retain => "retain",
            StrFunction::Split => "split",
            StrFunction::SplitAt => "split_at",
            StrFunction::SplitInclusive => "split_inclusive",
            StrFunction::SplitN => "split_n",
            StrFunction::SplitOnce => "split_once",
            StrFunction::SplitTerminator => "split_terminator",
            StrFunction::SplitWhitespace => "split_whitespace",
            StrFunction::StartsWith => "starts_with",
            StrFunction::StripPrefix => "strip_prefix",
            StrFunction::ToLowercase => "to_lowercase",
            StrFunction::ToUppercase => "to_uppercase",
            StrFunction::Trim => "trim",
            StrFunction::TrimEnd => "trim_end",
            StrFunction::TrimEndMatches => "trim_end_matches",
            StrFunction::TrimMatches => "trim_matches",
            StrFunction::TrimStart => "trim_start",
            StrFunction::TrimStartMatches => "trim_start_matches",
            StrFunction::Truncate => "truncate",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            StrFunction::AsBytes => "TODO",
            StrFunction::EndsWith => "TODO",
            StrFunction::Find => "TODO",
            StrFunction::Insert => "TODO",
            StrFunction::IsAscii => "TODO",
            StrFunction::IsEmpty => "TODO",
            StrFunction::Lines => "TODO",
            StrFunction::Matches => "TODO",
            StrFunction::Parse => "TODO",
            StrFunction::Remove => "TODO",
            StrFunction::ReplaceRange => "TODO",
            StrFunction::Retain => "TODO",
            StrFunction::Split => "TODO",
            StrFunction::SplitAt => "TODO",
            StrFunction::SplitInclusive => "TODO",
            StrFunction::SplitN => "TODO",
            StrFunction::SplitOnce => "TODO",
            StrFunction::SplitTerminator => "TODO",
            StrFunction::SplitWhitespace => "TODO",
            StrFunction::StartsWith => "TODO",
            StrFunction::StripPrefix => "TODO",
            StrFunction::ToLowercase => "TODO",
            StrFunction::ToUppercase => "TODO",
            StrFunction::Trim => "TODO",
            StrFunction::TrimEnd => "TODO",
            StrFunction::TrimEndMatches => "TODO",
            StrFunction::TrimMatches => "TODO",
            StrFunction::TrimStart => "TODO",
            StrFunction::TrimStartMatches => "TODO",
            StrFunction::Truncate => "TODO",
        }
    }

    fn r#type(&self) -> Type {
        match self {
            StrFunction::AsBytes => Type::function(vec![Type::String], Type::list(Type::Integer)),
            StrFunction::EndsWith => {
                Type::function(vec![Type::String, Type::String], Type::Boolean)
            }
            StrFunction::Find => Type::function(
                vec![Type::String, Type::String],
                Type::option(Some(Type::Integer)),
            ),
            StrFunction::Insert => Type::function(
                vec![Type::String, Type::Integer, Type::String],
                Type::String,
            ),
            StrFunction::IsAscii => Type::function(vec![Type::String], Type::Boolean),
            StrFunction::IsEmpty => Type::function(vec![Type::String], Type::Boolean),
            StrFunction::Lines => Type::function(vec![Type::String], Type::list(Type::String)),
            StrFunction::Matches => {
                Type::function(vec![Type::String, Type::String], Type::list(Type::String))
            }
            StrFunction::Parse => Type::function(vec![Type::String], Type::Any),
            StrFunction::Remove => Type::function(
                vec![Type::String, Type::Integer],
                Type::option(Some(Type::String)),
            ),
            StrFunction::ReplaceRange => Type::function(
                vec![Type::String, Type::list(Type::Integer), Type::String],
                Type::String,
            ),
            StrFunction::Retain => Type::function(
                vec![
                    Type::String,
                    Type::function(vec![Type::String], Type::Boolean),
                ],
                Type::String,
            ),
            StrFunction::Split => {
                Type::function(vec![Type::String, Type::String], Type::list(Type::String))
            }
            StrFunction::SplitAt => {
                Type::function(vec![Type::String, Type::Integer], Type::list(Type::String))
            }
            StrFunction::SplitInclusive => {
                Type::function(vec![Type::String, Type::String], Type::list(Type::String))
            }
            StrFunction::SplitN => Type::function(
                vec![Type::String, Type::Integer, Type::String],
                Type::list(Type::String),
            ),
            StrFunction::SplitOnce => {
                Type::function(vec![Type::String, Type::String], Type::list(Type::String))
            }
            StrFunction::SplitTerminator => {
                Type::function(vec![Type::String, Type::String], Type::list(Type::String))
            }
            StrFunction::SplitWhitespace => {
                Type::function(vec![Type::String], Type::list(Type::String))
            }
            StrFunction::StartsWith => {
                Type::function(vec![Type::String, Type::String], Type::Boolean)
            }
            StrFunction::StripPrefix => Type::function(
                vec![Type::String, Type::String],
                Type::option(Some(Type::String)),
            ),
            StrFunction::ToLowercase => Type::function(vec![Type::String], Type::String),
            StrFunction::ToUppercase => Type::function(vec![Type::String], Type::String),
            StrFunction::Truncate => {
                Type::function(vec![Type::String, Type::Integer], Type::String)
            }
            StrFunction::Trim => Type::function(vec![Type::String], Type::String),
            StrFunction::TrimEnd => Type::function(vec![Type::String], Type::String),
            StrFunction::TrimEndMatches => {
                Type::function(vec![Type::String, Type::String], Type::String)
            }
            StrFunction::TrimMatches => {
                Type::function(vec![Type::String, Type::String], Type::String)
            }
            StrFunction::TrimStart => Type::function(vec![Type::String], Type::String),
            StrFunction::TrimStartMatches => {
                Type::function(vec![Type::String, Type::String], Type::String)
            }
        }
    }

    fn call(
        &self,
        arguments: &[Value],
        _source: &str,
        _context: &Context,
    ) -> Result<Value, RuntimeError> {
        let value = match self {
            StrFunction::AsBytes => {
                RuntimeError::expect_argument_amount(self.name(), 1, arguments.len())?;

                let string = arguments.first().unwrap().as_string()?;
                let bytes = string
                    .bytes()
                    .map(|byte| Value::Integer(byte as i64))
                    .collect();

                Value::List(List::with_items(bytes))
            }
            StrFunction::EndsWith => {
                RuntimeError::expect_argument_amount(self.name(), 2, arguments.len())?;

                let string = arguments.first().unwrap().as_string()?;
                let pattern_string = arguments.get(1).unwrap().as_string()?;
                let pattern = pattern_string.as_str();

                Value::Boolean(string.ends_with(pattern))
            }
            StrFunction::Find => {
                RuntimeError::expect_argument_amount(self.name(), 2, arguments.len())?;

                let string = arguments.first().unwrap().as_string()?;
                let pattern_string = arguments.get(1).unwrap().as_string()?;
                let pattern = pattern_string.as_str();
                let find = string
                    .find(pattern)
                    .map(|index| Value::Integer(index as i64));

                if let Some(index) = find {
                    Value::Enum(EnumInstance::new(
                        Identifier::new("Option"),
                        Identifier::new("Some"),
                        Some(index),
                    ))
                } else {
                    Value::Enum(EnumInstance::new(
                        Identifier::new("Option"),
                        Identifier::new("None"),
                        Some(Value::none()),
                    ))
                }
            }
            StrFunction::IsAscii => {
                RuntimeError::expect_argument_amount(self.name(), 1, arguments.len())?;

                let string = arguments.first().unwrap().as_string()?;

                Value::Boolean(string.is_ascii())
            }
            StrFunction::IsEmpty => {
                RuntimeError::expect_argument_amount(self.name(), 1, arguments.len())?;

                let string = arguments.first().unwrap().as_string()?;

                Value::Boolean(string.is_empty())
            }
            StrFunction::Insert => {
                RuntimeError::expect_argument_amount(self.name(), 3, arguments.len())?;

                let mut string = arguments.first().unwrap().as_string()?.clone();
                let index = arguments.get(1).unwrap().as_integer()? as usize;
                let insertion = arguments.get(2).unwrap().as_string()?;

                string.insert_str(index, insertion);

                Value::String(string)
            }
            StrFunction::Lines => {
                RuntimeError::expect_argument_amount(self.name(), 1, arguments.len())?;

                let string = arguments.first().unwrap().as_string()?;
                let lines = string
                    .lines()
                    .map(|line| Value::string(line.to_string()))
                    .collect();

                Value::List(List::with_items(lines))
            }
            StrFunction::Matches => {
                RuntimeError::expect_argument_amount(self.name(), 2, arguments.len())?;

                let string = arguments.first().unwrap().as_string()?;
                let pattern_string = arguments.get(1).unwrap().as_string()?;
                let pattern = pattern_string.as_str();
                let matches = string
                    .matches(pattern)
                    .map(|pattern| Value::string(pattern.to_string()))
                    .collect();

                Value::List(List::with_items(matches))
            }
            StrFunction::Parse => {
                RuntimeError::expect_argument_amount(self.name(), 1, arguments.len())?;

                let string = arguments.first().unwrap().as_string()?;

                if let Ok(integer) = string.parse::<i64>() {
                    Value::Integer(integer)
                } else if let Ok(float) = string.parse::<f64>() {
                    Value::Float(float)
                } else {
                    Value::none()
                }
            }
            StrFunction::Remove => {
                RuntimeError::expect_argument_amount(self.name(), 2, arguments.len())?;

                let string = arguments.first().unwrap().as_string()?;
                let index = arguments.get(1).unwrap().as_integer()? as usize;
                let chars = string.chars().collect::<Vec<char>>();

                if index < chars.len() {
                    let new_string = chars
                        .iter()
                        .map(|char| char.to_string())
                        .collect::<String>();

                    Value::some(Value::string(new_string))
                } else {
                    Value::none()
                }
            }
            StrFunction::ReplaceRange => {
                RuntimeError::expect_argument_amount(self.name(), 3, arguments.len())?;

                let mut string = arguments.first().unwrap().as_string()?.clone();
                let range = arguments.get(1).unwrap().as_list()?.items();
                let start = range[0].as_integer()? as usize;
                let end = range[1].as_integer()? as usize;
                let pattern = arguments.get(2).unwrap().as_string()?;

                string.replace_range(start..end, pattern);

                Value::String(string)
            }
            StrFunction::Retain => {
                RuntimeError::expect_argument_amount(self.name(), 2, arguments.len())?;

                todo!();

                // let mut string = arguments.first().unwrap().as_string()?.clone();
                // let predicate = arguments.get(1).unwrap().as_function()?;

                // string.retain(|char| {
                //     predicate
                //         .call(&[Value::string(char)], _source, _outer_context)
                //         .unwrap()
                //         .as_boolean()
                //         .unwrap()
                // });

                // Value::String(string)
            }
            StrFunction::Split => {
                RuntimeError::expect_argument_amount(self.name(), 2, arguments.len())?;

                let string = arguments.first().unwrap().as_string()?;
                let pattern_string = arguments.get(1).unwrap().as_string()?;
                let pattern = pattern_string.as_str();
                let sections = string
                    .split(pattern)
                    .map(|section| Value::string(section.to_string()))
                    .collect();

                Value::List(List::with_items(sections))
            }
            StrFunction::SplitAt => {
                RuntimeError::expect_argument_amount(self.name(), 2, arguments.len())?;

                let string = arguments.first().unwrap().as_string()?;
                let index = arguments.get(1).unwrap().as_integer()?;
                let (left, right) = string.split_at(index as usize);

                Value::List(List::with_items(vec![
                    Value::string(left.to_string()),
                    Value::string(right.to_string()),
                ]))
            }
            StrFunction::SplitInclusive => {
                RuntimeError::expect_argument_amount(self.name(), 2, arguments.len())?;

                let string = arguments.first().unwrap().as_string()?;
                let pattern_string = arguments.get(1).unwrap().as_string()?;
                let pattern = pattern_string.as_str();
                let sections = string
                    .split(pattern)
                    .map(|pattern| Value::string(pattern.to_string()))
                    .collect();

                Value::List(List::with_items(sections))
            }
            StrFunction::SplitN => {
                RuntimeError::expect_argument_amount(self.name(), 3, arguments.len())?;

                let string = arguments.first().unwrap().as_string()?;
                let count = arguments.get(1).unwrap().as_integer()?;
                let pattern_string = arguments.get(2).unwrap().as_string()?;
                let pattern = pattern_string.as_str();
                let sections = string
                    .splitn(count as usize, pattern)
                    .map(|pattern| Value::string(pattern.to_string()))
                    .collect();

                Value::List(List::with_items(sections))
            }
            StrFunction::SplitOnce => {
                RuntimeError::expect_argument_amount(self.name(), 2, arguments.len())?;

                let string = arguments.first().unwrap().as_string()?;
                let pattern_string = arguments.get(1).unwrap().as_string()?;
                let pattern = pattern_string.as_str();
                let sections = string.split_once(pattern).map(|(left, right)| {
                    Value::List(List::with_items(vec![
                        Value::string(left.to_string()),
                        Value::string(right.to_string()),
                    ]))
                });

                if let Some(sections) = sections {
                    Value::some(sections)
                } else {
                    Value::none()
                }
            }
            StrFunction::SplitTerminator => {
                RuntimeError::expect_argument_amount(self.name(), 2, arguments.len())?;

                let string = arguments.first().unwrap().as_string()?;
                let pattern_string = arguments.get(1).unwrap().as_string()?;
                let pattern = pattern_string.as_str();
                let sections = string
                    .split_terminator(pattern)
                    .map(|section| Value::string(section.to_string()))
                    .collect();

                Value::List(List::with_items(sections))
            }
            StrFunction::SplitWhitespace => {
                RuntimeError::expect_argument_amount(self.name(), 1, arguments.len())?;

                let string = arguments.first().unwrap().as_string()?;
                let sections = string
                    .split_whitespace()
                    .map(|section| Value::string(section.to_string()))
                    .collect();

                Value::List(List::with_items(sections))
            }
            StrFunction::StartsWith => {
                RuntimeError::expect_argument_amount(self.name(), 2, arguments.len())?;

                let string = arguments.first().unwrap().as_string()?;
                let pattern_string = arguments.get(1).unwrap().as_string()?;
                let pattern = pattern_string.as_str();

                Value::Boolean(string.starts_with(pattern))
            }
            StrFunction::StripPrefix => {
                RuntimeError::expect_argument_amount(self.name(), 2, arguments.len())?;

                let string = arguments.first().unwrap().as_string()?;
                let prefix_string = arguments.get(1).unwrap().as_string()?;
                let prefix = prefix_string.as_str();
                let stripped = string
                    .strip_prefix(prefix)
                    .map(|remainder| Value::string(remainder.to_string()));

                if let Some(value) = stripped {
                    Value::Enum(EnumInstance::new(
                        Identifier::new("Option"),
                        Identifier::new("Some"),
                        Some(value),
                    ))
                } else {
                    Value::none()
                }
            }
            StrFunction::ToLowercase => {
                RuntimeError::expect_argument_amount(self.name(), 1, arguments.len())?;

                let string = arguments.first().unwrap().as_string()?;
                let lowercase = string.to_lowercase();

                Value::string(lowercase)
            }
            StrFunction::ToUppercase => {
                RuntimeError::expect_argument_amount(self.name(), 1, arguments.len())?;

                let string = arguments.first().unwrap().as_string()?;
                let uppercase = string.to_uppercase();

                Value::string(uppercase)
            }
            StrFunction::Trim => {
                RuntimeError::expect_argument_amount(self.name(), 1, arguments.len())?;

                let trimmed = arguments.first().unwrap().as_string()?.trim().to_string();

                Value::string(trimmed)
            }
            StrFunction::TrimEnd => {
                RuntimeError::expect_argument_amount(self.name(), 1, arguments.len())?;

                let trimmed = arguments
                    .first()
                    .unwrap()
                    .as_string()?
                    .trim_end()
                    .to_string();

                Value::string(trimmed)
            }
            StrFunction::TrimEndMatches => {
                RuntimeError::expect_argument_amount(self.name(), 2, arguments.len())?;

                let string = arguments.first().unwrap().as_string()?;
                let pattern_string = arguments.get(1).unwrap().as_string()?;
                let pattern = pattern_string.as_str();
                let trimmed = string.trim_end_matches(pattern).to_string();

                Value::string(trimmed)
            }
            StrFunction::TrimMatches => {
                RuntimeError::expect_argument_amount(self.name(), 2, arguments.len())?;

                let string = arguments.first().unwrap().as_string()?;
                let pattern = arguments
                    .get(1)
                    .unwrap()
                    .as_string()?
                    .chars()
                    .collect::<Vec<char>>();
                let trimmed = string.trim_matches(pattern.as_slice()).to_string();

                Value::string(trimmed)
            }
            StrFunction::TrimStart => {
                RuntimeError::expect_argument_amount(self.name(), 1, arguments.len())?;

                let trimmed = arguments
                    .first()
                    .unwrap()
                    .as_string()?
                    .trim_start()
                    .to_string();

                Value::string(trimmed)
            }
            StrFunction::TrimStartMatches => {
                RuntimeError::expect_argument_amount(self.name(), 2, arguments.len())?;

                let string = arguments.first().unwrap().as_string()?;
                let pattern = arguments
                    .get(1)
                    .unwrap()
                    .as_string()?
                    .chars()
                    .collect::<Vec<char>>();
                let trimmed = string.trim_start_matches(pattern.as_slice()).to_string();

                Value::string(trimmed)
            }
            StrFunction::Truncate => {
                RuntimeError::expect_argument_amount(self.name(), 2, arguments.len())?;

                let input_string = arguments.first().unwrap().as_string()?;
                let new_length = arguments.get(1).unwrap().as_integer()? as usize;

                let new_string = input_string
                    .chars()
                    .take(new_length)
                    .map(|char| char.to_string())
                    .collect();

                Value::String(new_string)
            }
        };

        Ok(value)
    }
}

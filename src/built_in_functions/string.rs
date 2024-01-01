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
            StringFunction::EndsWith => todo!(),
            StringFunction::Find => todo!(),
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
            StringFunction::Trim => todo!(),
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

    pub fn r#type(&self) -> Type {
        match self {
            StringFunction::AsBytes => {
                Type::function(vec![Type::String], Type::list_of(Type::Integer))
            }
            StringFunction::EndsWith => todo!(),
            StringFunction::Find => todo!(),
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
            StringFunction::Trim => todo!(),
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
            StringFunction::EndsWith => todo!(),
            StringFunction::Find => todo!(),
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
            StringFunction::Trim => todo!(),
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

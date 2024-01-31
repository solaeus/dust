use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};

use crate::{error::RuntimeError, Error, Map, Type, Value};

use super::Callable;

pub fn json_functions() -> impl Iterator<Item = Json> {
    enum_iterator::all()
}

#[derive(Sequence, Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Json {
    Create,
    CreatePretty,
    Parse,
}

impl Callable for Json {
    fn name(&self) -> &'static str {
        match self {
            Json::Create => "create",
            Json::CreatePretty => "create_pretty",
            Json::Parse => "parse",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Json::Create => "Convert a value to a JSON string.",
            Json::CreatePretty => "Convert a value to a formatted JSON string.",
            Json::Parse => "Convert JSON to a value",
        }
    }

    fn r#type(&self) -> Type {
        match self {
            Json::Create => Type::function(vec![Type::Any], Type::String),
            Json::CreatePretty => Type::function(vec![Type::Any], Type::String),
            Json::Parse => Type::function(vec![Type::String], Type::Any),
        }
    }

    fn call(
        &self,
        arguments: &[Value],
        _source: &str,
        _outer_context: &Map,
    ) -> Result<Value, RuntimeError> {
        match self {
            Json::Create => {
                Error::expect_argument_amount(self.name(), 1, arguments.len())?;

                let value = arguments.first().unwrap();
                let json_string = serde_json::to_string(value)?;

                Ok(Value::String(json_string))
            }
            Json::CreatePretty => {
                Error::expect_argument_amount(self.name(), 1, arguments.len())?;

                let value = arguments.first().unwrap();
                let json_string = serde_json::to_string_pretty(value)?;

                Ok(Value::String(json_string))
            }
            Json::Parse => {
                Error::expect_argument_amount(self.name(), 1, arguments.len())?;

                let json_string = arguments.first().unwrap().as_string()?;
                let value = serde_json::from_str(json_string)?;

                Ok(value)
            }
        }
    }
}

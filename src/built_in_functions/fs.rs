use std::fs::read_to_string;

use enum_iterator::{all, Sequence};
use serde::{Deserialize, Serialize};

use crate::{error::RuntimeError, Map, Type, Value};

use super::Callable;

pub fn fs_functions() -> impl Iterator<Item = Fs> {
    all()
}

#[derive(Sequence, Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Fs {
    ReadFile,
}

impl Callable for Fs {
    fn name(&self) -> &'static str {
        match self {
            Fs::ReadFile => "read_file",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Fs::ReadFile => "Read the contents of a file to a string.",
        }
    }

    fn r#type(&self) -> Type {
        match self {
            Fs::ReadFile => Type::function(vec![Type::String], Type::String),
        }
    }

    fn call(
        &self,
        arguments: &[Value],
        _source: &str,
        _outer_context: &Map,
    ) -> Result<Value, RuntimeError> {
        match self {
            Fs::ReadFile => {
                RuntimeError::expect_argument_amount(self.name(), 1, arguments.len())?;

                let path = arguments.first().unwrap().as_string()?;
                let file_content = read_to_string(path.as_str())?;

                Ok(Value::string(file_content))
            }
        }
    }
}

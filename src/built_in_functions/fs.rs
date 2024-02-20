use std::{fs::File, io::Read};

use enum_iterator::{all, Sequence};
use serde::{Deserialize, Serialize};

use crate::{error::RuntimeError, Context, Type, Value};

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
        _outer_context: &Context,
    ) -> Result<Value, RuntimeError> {
        match self {
            Fs::ReadFile => {
                RuntimeError::expect_argument_amount(self.name(), 1, arguments.len())?;

                let path = arguments.first().unwrap().as_string()?;
                let mut file = File::open(path.as_str())?;
                let file_size = file.metadata()?.len() as usize;
                let mut file_content = String::with_capacity(file_size);

                file.read_to_string(&mut file_content)?;

                Ok(Value::string(file_content))
            }
        }
    }
}

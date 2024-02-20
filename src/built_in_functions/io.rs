use enum_iterator::{all, Sequence};
use serde::{Deserialize, Serialize};

use crate::{error::RuntimeError, Context, Type, Value};

use super::Callable;

pub fn all_io_functions() -> impl Iterator<Item = Io> {
    all()
}

#[derive(Sequence, Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Io {
    Stdin,
}

impl Callable for Io {
    fn name(&self) -> &'static str {
        match self {
            Io::Stdin => "stdin",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Io::Stdin => "Read input from stdin.",
        }
    }

    fn r#type(&self) -> crate::Type {
        match self {
            Io::Stdin => Type::Function {
                parameter_types: vec![],
                return_type: Box::new(Type::String),
            },
        }
    }

    fn call(
        &self,
        _arguments: &[Value],
        _source: &str,
        _context: &Context,
    ) -> Result<Value, RuntimeError> {
        match self {
            Io::Stdin => {
                let mut input = String::new();
                let stdin = std::io::stdin();

                stdin.read_line(&mut input)?;

                Ok(Value::string(input))
            }
        }
    }
}

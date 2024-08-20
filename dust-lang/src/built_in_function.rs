//! Integrated functions that can be called from Dust code.
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    io::{self, stdin, stdout, Write},
};

use serde::{Deserialize, Serialize};

use crate::{Identifier, Type, Value};

/// Integrated function that can be called from Dust code.
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BuiltInFunction {
    // String tools
    ToString,

    // Integer and float tools
    IsEven,
    IsOdd,

    // I/O
    ReadLine,
    WriteLine,
}

impl BuiltInFunction {
    pub fn name(&self) -> &'static str {
        match self {
            BuiltInFunction::IsEven => "is_even",
            BuiltInFunction::IsOdd => "is_odd",
            BuiltInFunction::ReadLine => "read_line",
            BuiltInFunction::ToString { .. } => "to_string",
            BuiltInFunction::WriteLine => "write_line",
        }
    }

    pub fn type_parameters(&self) -> Option<Vec<Identifier>> {
        match self {
            BuiltInFunction::ToString { .. } => None,
            BuiltInFunction::IsEven => None,
            BuiltInFunction::IsOdd => None,
            BuiltInFunction::ReadLine => None,
            BuiltInFunction::WriteLine => None,
        }
    }

    pub fn value_parameters(&self) -> Option<Vec<(Identifier, Type)>> {
        match self {
            BuiltInFunction::ToString { .. } => Some(vec![("value".into(), Type::Any)]),
            BuiltInFunction::IsEven => Some(vec![("value".into(), Type::Number)]),
            BuiltInFunction::IsOdd => Some(vec![("value".into(), Type::Number)]),
            BuiltInFunction::ReadLine => None,
            BuiltInFunction::WriteLine => Some(vec![("value".into(), Type::Any)]),
        }
    }

    pub fn return_type(&self) -> Option<Type> {
        match self {
            BuiltInFunction::ToString { .. } => Some(Type::String { length: None }),
            BuiltInFunction::IsEven => Some(Type::Boolean),
            BuiltInFunction::IsOdd => Some(Type::Boolean),
            BuiltInFunction::ReadLine => Some(Type::String { length: None }),
            BuiltInFunction::WriteLine => None,
        }
    }

    pub fn call(
        &self,
        _type_arguments: Option<Vec<Type>>,
        value_arguments: Option<Vec<Value>>,
    ) -> Result<Option<Value>, BuiltInFunctionError> {
        match (self.value_parameters(), &value_arguments) {
            (Some(value_parameters), Some(value_arguments)) => {
                if value_parameters.len() != value_arguments.len() {
                    return Err(BuiltInFunctionError::WrongNumberOfValueArguments);
                }
            }
            (Some(_), None) | (None, Some(_)) => {
                return Err(BuiltInFunctionError::WrongNumberOfValueArguments);
            }
            (None, None) => {}
        }

        match self {
            BuiltInFunction::ToString => {
                Ok(Some(Value::String(value_arguments.unwrap()[0].to_string())))
            }
            BuiltInFunction::IsEven => {
                if let Value::Integer(integer) = value_arguments.unwrap()[0] {
                    Ok(Some(Value::Boolean(integer % 2 == 0)))
                } else {
                    Err(BuiltInFunctionError::ExpectedInteger)
                }
            }
            BuiltInFunction::IsOdd => {
                if let Value::Integer(integer) = value_arguments.unwrap()[0] {
                    Ok(Some(Value::Boolean(integer % 2 != 0)))
                } else {
                    Err(BuiltInFunctionError::ExpectedInteger)
                }
            }
            BuiltInFunction::ReadLine => {
                let mut input = String::new();

                stdin().read_line(&mut input)?;

                Ok(Some(Value::string(input.trim_end_matches('\n'))))
            }
            BuiltInFunction::WriteLine => {
                if let Value::String(string) = &value_arguments.unwrap()[0] {
                    let mut stdout = stdout();

                    stdout.write_all(string.as_bytes())?;
                    stdout.write_all(b"\n")?;

                    Ok(None)
                } else {
                    Err(BuiltInFunctionError::ExpectedString)
                }
            }
        }
    }
}

impl Display for BuiltInFunction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BuiltInFunctionError {
    Io(io::ErrorKind),

    ExpectedString,
    ExpectedList,
    ExpectedInteger,

    WrongNumberOfValueArguments,
}

impl From<io::Error> for BuiltInFunctionError {
    fn from(error: io::Error) -> Self {
        Self::Io(error.kind())
    }
}

impl Error for BuiltInFunctionError {}

impl Display for BuiltInFunctionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            BuiltInFunctionError::Io(error_kind) => write!(f, "I/O error: {}", error_kind),
            BuiltInFunctionError::ExpectedInteger => write!(f, "Expected an integer"),
            BuiltInFunctionError::ExpectedString => write!(f, "Expected a string"),
            BuiltInFunctionError::ExpectedList => write!(f, "Expected a list"),
            BuiltInFunctionError::WrongNumberOfValueArguments => {
                write!(f, "Wrong number of value arguments")
            }
        }
    }
}

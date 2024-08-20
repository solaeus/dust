//! Integrated functions that can be called from Dust code.
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    io::{self, stdin},
};

use serde::{Deserialize, Serialize};

use crate::{Identifier, Type, Value};

/// Integrated function that can be called from Dust code.
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BuiltInFunction {
    // String tools
    ToString { argument: Box<Value> },

    // Integer and float tools
    IsEven,
    IsOdd,

    // List tools
    Length,

    // I/O
    ReadLine,
    WriteLine,
}

impl BuiltInFunction {
    pub fn name(&self) -> &'static str {
        match self {
            BuiltInFunction::IsEven => "is_even",
            BuiltInFunction::IsOdd => "is_odd",
            BuiltInFunction::Length => "length",
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
            BuiltInFunction::Length => None,
            BuiltInFunction::ReadLine => None,
            BuiltInFunction::WriteLine => None,
        }
    }

    pub fn value_parameters(&self) -> Option<Vec<(Identifier, Type)>> {
        match self {
            BuiltInFunction::ToString { .. } => Some(vec![("value".into(), Type::Any)]),
            BuiltInFunction::IsEven => Some(vec![("value".into(), Type::Number)]),
            BuiltInFunction::IsOdd => Some(vec![("value".into(), Type::Number)]),
            BuiltInFunction::Length => Some(vec![(
                "value".into(),
                Type::ListOf {
                    item_type: Box::new(Type::Any),
                },
            )]),
            BuiltInFunction::ReadLine => None,
            BuiltInFunction::WriteLine => Some(vec![("output".into(), Type::Any)]),
        }
    }

    pub fn return_type(&self) -> Option<Type> {
        match self {
            BuiltInFunction::ToString { .. } => Some(Type::String),
            BuiltInFunction::IsEven => Some(Type::Boolean),
            BuiltInFunction::IsOdd => Some(Type::Boolean),
            BuiltInFunction::Length => Some(Type::Number),
            BuiltInFunction::ReadLine => Some(Type::String),
            BuiltInFunction::WriteLine => None,
        }
    }

    pub fn call(
        &self,
        _type_arguments: Option<Vec<Type>>,
        value_arguments: Option<Vec<Value>>,
    ) -> Result<Option<Value>, BuiltInFunctionError> {
        match self {
            BuiltInFunction::ToString { argument } => Ok(Some(Value::string(argument))),
            BuiltInFunction::IsEven => {
                if let Some(value_arguments) = value_arguments {
                    if value_arguments.len() == 1 {
                        if let Some(integer) = value_arguments[0].as_integer() {
                            Ok(Some(Value::Boolean(integer % 2 == 0)))
                        } else {
                            Err(BuiltInFunctionError::ExpectedInteger)
                        }
                    } else {
                        Err(BuiltInFunctionError::WrongNumberOfValueArguments)
                    }
                } else {
                    Err(BuiltInFunctionError::WrongNumberOfValueArguments)
                }
            }
            BuiltInFunction::IsOdd => {
                if let Some(value_arguments) = value_arguments {
                    if value_arguments.len() == 1 {
                        if let Some(integer) = value_arguments[0].as_integer() {
                            Ok(Some(Value::Boolean(integer % 2 != 0)))
                        } else {
                            Err(BuiltInFunctionError::ExpectedInteger)
                        }
                    } else {
                        Err(BuiltInFunctionError::WrongNumberOfValueArguments)
                    }
                } else {
                    Err(BuiltInFunctionError::WrongNumberOfValueArguments)
                }
            }
            BuiltInFunction::Length => {
                if let Some(value_arguments) = value_arguments {
                    if value_arguments.len() == 1 {
                        if let Value::List(list) = &value_arguments[0] {
                            Ok(Some(Value::Integer(list.len() as i64)))
                        } else {
                            Err(BuiltInFunctionError::ExpectedInteger)
                        }
                    } else {
                        Err(BuiltInFunctionError::WrongNumberOfValueArguments)
                    }
                } else {
                    Err(BuiltInFunctionError::WrongNumberOfValueArguments)
                }
            }
            BuiltInFunction::ReadLine => {
                if value_arguments.is_none() {
                    let mut input = String::new();

                    stdin().read_line(&mut input)?;

                    Ok(Some(Value::string(input.trim_end_matches('\n'))))
                } else {
                    Err(BuiltInFunctionError::WrongNumberOfValueArguments)
                }
            }
            BuiltInFunction::WriteLine => {
                if let Some(value_arguments) = value_arguments {
                    if value_arguments.len() == 1 {
                        println!("{}", value_arguments[0]);

                        Ok(None)
                    } else {
                        Err(BuiltInFunctionError::WrongNumberOfValueArguments)
                    }
                } else {
                    Err(BuiltInFunctionError::WrongNumberOfValueArguments)
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
    ExpectedInteger,
    Io(io::ErrorKind),
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
            BuiltInFunctionError::ExpectedInteger => write!(f, "Expected an integer"),
            BuiltInFunctionError::Io(error_kind) => write!(f, "I/O error: {}", error_kind),
            BuiltInFunctionError::WrongNumberOfValueArguments => {
                write!(f, "Wrong number of value arguments")
            }
        }
    }
}

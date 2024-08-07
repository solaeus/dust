use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{Type, Value};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BuiltInFunction {
    IsEven,
    IsOdd,
    Length,
}

impl BuiltInFunction {
    pub fn name(&self) -> &'static str {
        match self {
            BuiltInFunction::IsEven => "is_even",
            BuiltInFunction::IsOdd => "is_odd",
            BuiltInFunction::Length => "length",
        }
    }

    pub fn call(
        &self,
        _type_arguments: Option<Vec<Type>>,
        value_arguments: Option<Vec<Value>>,
    ) -> Result<Value, BuiltInFunctionError> {
        match self {
            BuiltInFunction::IsEven => {
                if let Some(value_arguments) = value_arguments {
                    if value_arguments.len() == 1 {
                        if let Some(integer) = value_arguments[0].as_integer() {
                            Ok(Value::boolean(integer % 2 == 0))
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
                            Ok(Value::boolean(integer % 2 != 0))
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
                        if let Some(list) = value_arguments[0].as_list() {
                            Ok(Value::integer(list.len() as i64))
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
        }
    }

    pub fn expected_type(&self) -> Option<Type> {
        match self {
            BuiltInFunction::IsEven => Some(Type::Boolean),
            BuiltInFunction::IsOdd => Some(Type::Boolean),
            BuiltInFunction::Length => Some(Type::Integer),
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
    WrongNumberOfValueArguments,
}

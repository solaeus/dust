use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::OperandType;

#[derive(Clone, Default, Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Type {
    #[default]
    None,

    Boolean,
    Byte,
    Character,
    Float,
    Integer,

    String,
    Array(Box<Type>, usize),
    List(Box<Type>),
    Map,

    Function(Box<FunctionType>),
    FunctionSelf,
}

impl Type {
    pub fn array(element_type: Type, length: usize) -> Self {
        Type::Array(Box::new(element_type), length)
    }

    pub fn list(element_type: Type) -> Self {
        Type::List(Box::new(element_type))
    }

    pub fn function<T: Into<Vec<Type>>, U: Into<Vec<Type>>>(
        type_parameters: T,
        value_parameters: U,
        return_type: Type,
    ) -> Self {
        Type::Function(Box::new(FunctionType {
            type_parameters: type_parameters.into(),
            value_parameters: value_parameters.into(),
            return_type,
        }))
    }

    pub fn as_operand_type(&self) -> OperandType {
        match self {
            Type::None => OperandType::NONE,
            Type::Boolean => OperandType::BOOLEAN,
            Type::Byte => OperandType::BYTE,
            Type::Character => OperandType::CHARACTER,
            Type::Float => OperandType::FLOAT,
            Type::Integer => OperandType::INTEGER,
            Type::String => OperandType::STRING,
            Type::Array(item_type, _) => match item_type.as_ref() {
                Type::Boolean => OperandType::ARRAY_BOOLEAN,
                Type::Byte => OperandType::ARRAY_BYTE,
                Type::Character => OperandType::ARRAY_CHARACTER,
                Type::Float => OperandType::ARRAY_FLOAT,
                Type::Integer => OperandType::ARRAY_INTEGER,
                Type::String => OperandType::ARRAY_STRING,
                Type::Map => OperandType::ARRAY_MAP,
                Type::Function(_) | Type::FunctionSelf => OperandType::ARRAY_FUNCTION,
                Type::Array(_, _) => OperandType::ARRAY_ARRAY,
                Type::List(_) => OperandType::ARRAY_LIST,
                Type::None => {
                    panic!("An array's item type must be known, even if it is empty")
                }
            },
            Type::List(item_type) => match item_type.as_ref() {
                Type::Boolean => OperandType::LIST_BOOLEAN,
                Type::Byte => OperandType::LIST_BYTE,
                Type::Character => OperandType::LIST_CHARACTER,
                Type::Float => OperandType::LIST_FLOAT,
                Type::Integer => OperandType::LIST_INTEGER,
                Type::String => OperandType::LIST_STRING,
                Type::Map => OperandType::LIST_MAP,
                Type::Function(_) | Type::FunctionSelf => OperandType::LIST_FUNCTION,
                Type::Array(_, _) => OperandType::LIST_ARRAY,
                Type::List(_) => OperandType::LIST_LIST,
                Type::None => panic!("A list's item type must be known, even if it is empty"),
            },
            Type::Map => OperandType::MAP,
            Type::Function(_) | Type::FunctionSelf => OperandType::FUNCTION,
        }
    }

    /// Checks that the type is compatible with another type.
    pub fn check(&self, other: &Type) -> Result<(), TypeConflict> {
        match (self, other) {
            (Type::Boolean, Type::Boolean)
            | (Type::Byte, Type::Byte)
            | (Type::Character, Type::Character)
            | (Type::Float, Type::Float)
            | (Type::Integer, Type::Integer)
            | (Type::None, Type::None)
            | (Type::String, Type::String) => return Ok(()),
            (Type::List(left_type), Type::List(right_type)) => {
                if left_type != right_type {
                    return Err(TypeConflict {
                        actual: other.clone(),
                        expected: self.clone(),
                    });
                }

                return Ok(());
            }
            (Type::Function(left_function_type), Type::Function(right_function_type)) => {
                if left_function_type != right_function_type {
                    return Err(TypeConflict {
                        actual: other.clone(),
                        expected: self.clone(),
                    });
                }

                return Ok(());
            }
            _ => {}
        }

        Err(TypeConflict {
            actual: other.clone(),
            expected: self.clone(),
        })
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Type::Boolean => write!(f, "bool"),
            Type::Byte => write!(f, "byte"),
            Type::Character => write!(f, "char"),
            Type::Float => write!(f, "float"),
            Type::Function(function_type) => write!(f, "{function_type}"),
            Type::Integer => write!(f, "int"),
            Type::List(item_type) => write!(f, "List<{item_type}>"),
            Type::Map => write!(f, "map"),
            Type::None => write!(f, "none"),
            Type::FunctionSelf => write!(f, "self"),
            Type::String => write!(f, "str"),
            Type::Array(item_type, length) => write!(f, "[{item_type}; {length}]"),
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FunctionType {
    pub type_parameters: Vec<Type>,
    pub value_parameters: Vec<Type>,
    pub return_type: Type,
}

impl FunctionType {
    pub fn new<T: Into<Vec<Type>>, U: Into<Vec<Type>>>(
        type_parameters: T,
        value_parameters: U,
        return_type: Type,
    ) -> Self {
        FunctionType {
            type_parameters: type_parameters.into(),
            value_parameters: value_parameters.into(),
            return_type,
        }
    }
}

impl Default for FunctionType {
    fn default() -> Self {
        FunctionType {
            type_parameters: Vec::new(),
            value_parameters: Vec::new(),
            return_type: Type::None,
        }
    }
}

impl Display for FunctionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "fn")?;

        if !self.type_parameters.is_empty() {
            write!(f, "<")?;

            for (index, type_parameter) in self.type_parameters.iter().enumerate() {
                if index > 0 {
                    write!(f, ", ")?;
                }

                write!(f, "{type_parameter}")?;
            }

            write!(f, ">")?;
        }

        write!(f, "(")?;

        if !self.value_parameters.is_empty() {
            for (index, r#type) in self.value_parameters.iter().enumerate() {
                if index > 0 {
                    write!(f, ", ")?;
                }

                write!(f, "{type}")?;
            }
        }

        write!(f, ")")?;

        if self.return_type != Type::None {
            write!(f, " -> {}", self.return_type)?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TypeConflict {
    pub expected: Type,
    pub actual: Type,
}

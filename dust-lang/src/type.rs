//! Value types and conflict handling.
use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};
use tracing::error;

use crate::instruction::OperandType;

/// Description of a kind of value.
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
    Range(Box<Type>),

    Function(Box<FunctionType>),
    FunctionSelf,
}

impl Type {
    pub fn function<T: Into<Vec<u16>>, U: Into<Vec<Type>>>(
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

    pub fn list(item_type: Type) -> Self {
        Type::List(Box::new(item_type))
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
            Type::List(item_type) => match item_type.as_ref() {
                Type::Boolean => OperandType::LIST_BOOLEAN,
                Type::Byte => OperandType::LIST_BYTE,
                Type::Character => OperandType::LIST_CHARACTER,
                Type::Float => OperandType::LIST_FLOAT,
                Type::Integer => OperandType::LIST_INTEGER,
                Type::String => OperandType::LIST_STRING,
                Type::List(_) | Type::Range(_) => OperandType::LIST_LIST,
                Type::Map => OperandType::LIST_MAP,
                Type::Function(_) | Type::FunctionSelf => OperandType::LIST_FUNCTION,
                Type::Array(_, _) => OperandType::LIST_ARRAY,
                Type::None => panic!("A list's item type must be known, even if it is empty"),
            },
            Type::Map => OperandType::MAP,
            Type::Range(range_type) => match range_type.as_ref() {
                Type::Byte => OperandType::LIST_BYTE,
                Type::Character => OperandType::LIST_CHARACTER,
                Type::Float => OperandType::LIST_FLOAT,
                Type::Integer => OperandType::LIST_INTEGER,
                _ => {
                    unreachable!("A range must have a numeric or character type, got: {range_type}")
                }
            },
            Type::Function(_) | Type::FunctionSelf => OperandType::FUNCTION,
            Type::Array(item_type, _) => match item_type.as_ref() {
                Type::Boolean => OperandType::ARRAY_BOOLEAN,
                Type::Byte => OperandType::ARRAY_BYTE,
                Type::Character => OperandType::ARRAY_CHARACTER,
                Type::Float => OperandType::ARRAY_FLOAT,
                Type::Integer => OperandType::ARRAY_INTEGER,
                Type::String => OperandType::ARRAY_STRING,
                Type::List(_) | Type::Range(_) => OperandType::ARRAY_LIST,
                Type::Map => OperandType::ARRAY_MAP,
                Type::Function(_) | Type::FunctionSelf => OperandType::ARRAY_FUNCTION,
                Type::Array(_, _) => OperandType::ARRAY_ARRAY,
                Type::None => {
                    panic!("An array's item type must be known, even if it is empty")
                }
            },
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
            (Type::Range(left_type), Type::Range(right_type)) => {
                if left_type == right_type {
                    return Ok(());
                }
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
            Type::Range(r#type) => write!(f, "Range<{type}>"),
            Type::FunctionSelf => write!(f, "self"),
            Type::String => write!(f, "str"),
            Type::Array(item_type, length) => write!(f, "[{item_type}; {length}]"),
        }
    }
}

/// An opaque representation of a value's type that does not hold of a type's details.
///
/// For primitive types (i.e. `bool`, `byte`, `char`, `float`, `int`, `str`, `[]` and `fn`) the
/// TypeKind is identitcal to the [`Type`]. But for `Generic` and all the compound types, none of
/// the type details are available. Therefore a `TypeKind` can represent a list but cannot convey
/// that it is a list of integers. This makes `TypeKind` much smaller (1 byte v.s. 32 bytes), which
/// is useful for performance.
#[derive(
    Clone, Copy, Default, Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize,
)]
pub enum ConcreteType {
    #[default]
    None,

    Boolean,
    Byte,
    Character,
    Float,
    Integer,
    String,

    Array,
    List,
    Map,
    Range,

    Function,
    FunctionSelf,
}

impl ConcreteType {
    pub fn write_invalid(&self, f: &mut Formatter) -> fmt::Result {
        error!(
            "Invalid type used: {:?}, writing \"INVALID\" instead.",
            self
        );
        write!(f, "INVALID")
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FunctionType {
    pub type_parameters: Vec<u16>,
    pub value_parameters: Vec<Type>,
    pub return_type: Type,
}

impl FunctionType {
    pub fn new<T: Into<Vec<u16>>, U: Into<Vec<Type>>>(
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

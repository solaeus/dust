use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::instruction::OperandType;

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
    List(Box<Type>),

    Function(Box<FunctionType>),
}

impl Type {
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

    pub fn as_element_type(&self) -> Option<&Type> {
        match self {
            Type::List(item_type) => Some(item_type.as_ref()),
            _ => None,
        }
    }

    pub fn into_function_type(self) -> Option<FunctionType> {
        match self {
            Type::Function(function_type) => Some(*function_type),
            _ => None,
        }
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
                Type::Function(_) => OperandType::LIST_FUNCTION,
                Type::List(_) => OperandType::LIST_LIST,
                Type::None => panic!("A list's item type must be known, even if it is empty"),
            },
            Type::Function(_) => OperandType::FUNCTION,
        }
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
            Type::None => write!(f, "none"),
            Type::String => write!(f, "str"),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

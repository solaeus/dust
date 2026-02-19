use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::small_type::SmallType;

#[derive(Clone, Default, Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DustType {
    #[default]
    None,
    Boolean,
    Byte,
    Character,
    Float,
    Integer,
    String,
    List(Box<DustType>),
    Function(Box<DustFunctionType>),
    Struct(Box<DustStructType>),
}

impl DustType {
    pub fn list(element_type: DustType) -> Self {
        DustType::List(Box::new(element_type))
    }

    pub fn function<T: Into<Vec<String>>, U: Into<Vec<DustType>>>(
        type_parameters: T,
        value_parameters: U,
        return_type: DustType,
    ) -> Self {
        DustType::Function(Box::new(DustFunctionType {
            type_parameters: type_parameters.into(),
            value_parameters: value_parameters.into(),
            return_type,
        }))
    }

    pub fn as_element_type(&self) -> Option<&DustType> {
        match self {
            DustType::List(item_type) => Some(item_type.as_ref()),
            _ => None,
        }
    }

    pub fn into_function_type(self) -> Option<DustFunctionType> {
        match self {
            DustType::Function(function_type) => Some(*function_type),
            _ => None,
        }
    }

    pub fn as_operand_type(&self) -> SmallType {
        match self {
            DustType::None => SmallType::NONE,
            DustType::Boolean => SmallType::BOOLEAN,
            DustType::Byte => SmallType::BYTE,
            DustType::Character => SmallType::CHARACTER,
            DustType::Float => SmallType::FLOAT,
            DustType::Integer => SmallType::INTEGER,
            DustType::String => SmallType::STRING,
            DustType::List(item_type) => match item_type.as_ref() {
                DustType::Boolean => SmallType::LIST_BOOLEAN,
                DustType::Byte => SmallType::LIST_BYTE,
                DustType::Character => SmallType::LIST_CHARACTER,
                DustType::Float => SmallType::LIST_FLOAT,
                DustType::Integer => SmallType::LIST_INTEGER,
                DustType::String => SmallType::LIST_STRING,
                DustType::Function(_) => SmallType::LIST_FUNCTION,
                DustType::List(_) => SmallType::LIST_LIST,
                DustType::Struct { .. } => SmallType::LIST_STRUCT,
                DustType::None => panic!("A list's item type must be known, even if it is empty"),
            },
            DustType::Struct { .. } => SmallType::STRUCT,
            DustType::Function(_) => SmallType::FUNCTION,
        }
    }
}

impl Display for DustType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            DustType::None => write!(f, "none"),
            DustType::Boolean => write!(f, "bool"),
            DustType::Byte => write!(f, "byte"),
            DustType::Character => write!(f, "char"),
            DustType::Float => write!(f, "float"),
            DustType::Function(function_type) => write!(f, "{function_type}"),
            DustType::Integer => write!(f, "int"),
            DustType::List(item_type) => write!(f, "[{item_type}]"),
            DustType::Struct(struct_type) => write!(f, "{struct_type}"),
            DustType::String => write!(f, "str"),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DustFunctionType {
    pub type_parameters: Vec<String>,
    pub value_parameters: Vec<DustType>,
    pub return_type: DustType,
}

impl DustFunctionType {
    pub fn new<T: Into<Vec<String>>, U: Into<Vec<DustType>>>(
        type_parameters: T,
        value_parameters: U,
        return_type: DustType,
    ) -> Self {
        DustFunctionType {
            type_parameters: type_parameters.into(),
            value_parameters: value_parameters.into(),
            return_type,
        }
    }
}

impl Display for DustFunctionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "fn")?;

        if !self.type_parameters.is_empty() {
            write!(f, "<")?;

            for (index, type_parameter_name) in self.type_parameters.iter().enumerate() {
                if index > 0 {
                    write!(f, ", ")?;
                }

                write!(f, "{type_parameter_name}")?;
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

        write!(f, ") -> {}", self.return_type)
    }
}

#[derive(Clone, Default, Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DustStructType {
    pub name: String,
    pub fields: Vec<(String, DustType)>,
}

impl Display for DustStructType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DustType {
    #[default]
    Unit,
    Boolean,
    I8,
    I16,
    I32,
    I64,
    I128,
    ISize,
    U8,
    U16,
    U32,
    U64,
    U128,
    USize,
    F32,
    F64,
    Character,
    Tuple(Vec<DustType>),
    Array(Box<DustType>, usize),
    Slice(Box<DustType>),
    Function(Box<DustFunctionType>),
    Struct(Box<DustStructType>),
    Enum(Box<DustEnumType>),
}

impl DustType {
    pub fn into_function_type(self) -> Option<DustFunctionType> {
        match self {
            DustType::Function(function_type) => Some(*function_type),
            _ => None,
        }
    }
}

impl Display for DustType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            DustType::Unit => write!(f, "()"),
            DustType::Boolean => write!(f, "bool"),
            DustType::Character => write!(f, "char"),
            DustType::I8 => write!(f, "i8"),
            DustType::I16 => write!(f, "i16"),
            DustType::I32 => write!(f, "i32"),
            DustType::I64 => write!(f, "i64"),
            DustType::I128 => write!(f, "i128"),
            DustType::ISize => write!(f, "isize"),
            DustType::U8 => write!(f, "u8"),
            DustType::U16 => write!(f, "u16"),
            DustType::U32 => write!(f, "u32"),
            DustType::U64 => write!(f, "u64"),
            DustType::U128 => write!(f, "u128"),
            DustType::USize => write!(f, "usize"),
            DustType::F32 => write!(f, "f32"),
            DustType::F64 => write!(f, "f64"),
            DustType::Tuple(item_type) => {
                write!(f, "(")?;

                for (index, r#type) in item_type.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{type}")?;
                }

                write!(f, ")")
            }
            DustType::Array(item_type, size) => write!(f, "[{item_type}; {size}]"),
            DustType::Slice(item_type) => write!(f, "[{item_type}]"),
            DustType::Function(function_type) => write!(f, "{function_type}"),
            DustType::Struct(struct_type) => write!(f, "{struct_type}"),
            DustType::Enum(enum_type) => write!(f, "{enum_type}"),
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

#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DustStructType {
    pub name: String,
    pub value_type: DustStructValueType,
}

impl Display for DustStructType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "struct {}{}", self.name, self.value_type)
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DustEnumType {
    pub name: String,
    pub variants: Vec<(String, DustStructValueType)>,
}

impl Display for DustEnumType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "enum {} {{", self.name)?;

        for (index, (variant_name, variant_value_type)) in self.variants.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }

            write!(f, "{variant_name}{variant_value_type}")?;
        }

        write!(f, "}}")
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DustStructValueType {
    Unit,
    Tuple(Vec<DustType>),
    Struct(Vec<(String, DustType)>),
}

impl Display for DustStructValueType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            DustStructValueType::Unit => Ok(()),
            DustStructValueType::Tuple(types) => {
                write!(f, "(")?;

                for (index, r#type) in types.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{type}")?;
                }

                write!(f, ")")
            }
            DustStructValueType::Struct(fields) => {
                if fields.len() == 1 {
                    let (field_name, field_type) = &fields[0];

                    return write!(f, "{{ {field_name}: {field_type} }}");
                }

                writeln!(f, "{{")?;

                for (field_name, field_type) in fields {
                    writeln!(f, "{field_name}: {field_type},")?;
                }

                writeln!(f, "}}")
            }
        }
    }
}

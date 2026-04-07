use std::fmt::{self, Display, Formatter};

/// External representation of a Dust value.
///
/// This is used to represent values going into and out of the VM. It is not used as a
/// representation of values in the compiler or the VM.
pub enum DustValue {
    Boolean(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    ISize(isize),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    USize(usize),
    F32(f32),
    F64(f64),
    Character(char),
    Tuple(Vec<DustValue>),
    Struct(Box<DustStruct>),
    EnumVariant(Box<DustEnumVariant>),
}

impl Display for DustValue {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            DustValue::Boolean(boolean) => write!(f, "{boolean}"),
            DustValue::I8(integer) => write!(f, "{integer}"),
            DustValue::I16(integer) => write!(f, "{integer}"),
            DustValue::I32(integer) => write!(f, "{integer}"),
            DustValue::I64(integer) => write!(f, "{integer}"),
            DustValue::I128(integer) => write!(f, "{integer}"),
            DustValue::ISize(integer) => write!(f, "{integer}"),
            DustValue::U8(integer) => write!(f, "{integer}"),
            DustValue::U16(integer) => write!(f, "{integer}"),
            DustValue::U32(integer) => write!(f, "{integer}"),
            DustValue::U64(integer) => write!(f, "{integer}"),
            DustValue::U128(integer) => write!(f, "{integer}"),
            DustValue::USize(integer) => write!(f, "{integer}"),
            DustValue::F32(float) => write!(f, "{float}"),
            DustValue::F64(float) => {
                if float % 1.0 == 0.0 {
                    write!(f, "{float:.1}")
                } else {
                    write!(f, "{float:.}")
                }
            }
            DustValue::Tuple(items) => {
                write!(f, "(")?;

                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{item}")?;
                }

                write!(f, ")")
            }
            DustValue::Character(character) => write!(f, "'{character}'"),
            DustValue::Struct(instance) => write!(f, "{instance}"),
            DustValue::EnumVariant(variant) => write!(f, "{variant}"),
        }
    }
}

pub struct DustStruct {
    pub struct_name: String,
    pub value: DustStructValue,
}

impl Display for DustStruct {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let DustStruct { struct_name, value } = self;

        write!(f, "{struct_name}{value}")
    }
}

pub struct DustEnumVariant {
    pub enum_name: String,
    pub variant_name: String,
    pub value: DustStructValue,
}

impl Display for DustEnumVariant {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let DustEnumVariant {
            enum_name,
            variant_name,
            value,
        } = self;

        write!(f, "{enum_name}::{variant_name}{value}")
    }
}

pub enum DustStructValue {
    Unit,
    Tuple(Vec<DustValue>),
    Struct(Vec<(String, DustValue)>),
}

impl Display for DustStructValue {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            DustStructValue::Unit => Ok(()),
            DustStructValue::Tuple(values) => {
                write!(f, " (")?;

                for (index, value) in values.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{value}")?;
                }

                write!(f, ")")
            }
            DustStructValue::Struct(fields) => {
                if fields.len() == 1 {
                    let (field_name, value) = &fields[0];

                    return write!(f, " {{ {field_name}: {value} }}");
                }

                writeln!(f, " {{")?;

                for (field_name, value) in fields {
                    writeln!(f, "{field_name}: {value},")?;
                }

                writeln!(f, "}}")
            }
        }
    }
}

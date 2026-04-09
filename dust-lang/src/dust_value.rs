//! A public interface for working with Dust values.

use std::{
    any::Any,
    fmt::{self, Debug, Display, Formatter},
    panic::catch_unwind,
};

/// A public interface for working with Dust values.
///
/// `DustValue`s are passed to and returned by the VM to allow for safe interaction with Rust code.
/// This type is *never* used as an internal representation of values, it is only for representing
/// Rust values in Dust and vice versa.
///
/// # Function Values
///
/// # Unit Values
///
/// Instances of the zero-sized "unit" type (`()`) are generally be represented as
/// `Option::<DustValue>::None` because they are not runtime values, they are just compile-time
/// constructs of the type system. In the very rare case that one wants to pass a unit value to the
/// VM, use an empty tuple (`DustValue::Tuple(Vec::new())`).
pub enum DustValue<E> {
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
    Tuple(Vec<DustValue<E>>),
    Struct(Box<DustStruct<E>>),
    EnumVariant(Box<DustEnumVariant<E>>),
    Function(DustFunction<E>),
}

impl<E> Display for DustValue<E> {
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
            DustValue::Function(DustFunction { name, .. }) => write!(
                f,
                "fn {name}<E>(arguments: &[DustValue]) -> Result<Option<DustValue>, E>"
            ),
        }
    }
}

pub struct DustStruct<E> {
    pub struct_name: String,
    pub value: DustStructValue<E>,
}

impl<E> Display for DustStruct<E> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let DustStruct { struct_name, value } = self;

        write!(f, "{struct_name}{value}")
    }
}

pub struct DustEnumVariant<E> {
    pub enum_name: String,
    pub variant_name: String,
    pub value: DustStructValue<E>,
}

impl<E> Display for DustEnumVariant<E> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let DustEnumVariant {
            enum_name,
            variant_name,
            value,
        } = self;

        write!(f, "{enum_name}::{variant_name}{value}")
    }
}

pub enum DustStructValue<E> {
    Unit,
    Tuple(Vec<DustValue<E>>),
    Struct(Vec<(String, DustValue<E>)>),
}

impl<E> Display for DustStructValue<E> {
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

pub struct DustFunction<E> {
    pub name: String,
    logic: fn(&[DustValue<E>]) -> Result<Option<DustValue<E>>, E>,
}

impl<E: Debug> DustFunction<E> {
    pub fn new(
        name: String,
        logic: fn(&[DustValue<E>]) -> Result<Option<DustValue<E>>, E>,
    ) -> Self {
        Self { name, logic }
    }

    pub fn call(
        &self,
        arguments: &[DustValue<E>],
    ) -> Result<Option<DustValue<E>>, DustFunctionError<E>> {
        catch_unwind(|| (self.logic)(arguments))
            .map_err(DustFunctionError::Panic)
            .and_then(|result| result.map_err(DustFunctionError::Custom))
    }
}

enum DustFunctionError<E> {
    Custom(E),
    Panic(Box<dyn Any + Send + 'static>),
}

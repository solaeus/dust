//! Value types and conflict handling.
use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

use serde::{Deserialize, Serialize};

/// Description of a kind of value.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Type {
    Any,
    Boolean,
    Byte,
    Character,
    Enum(EnumType),
    Float,
    Function(FunctionType),
    Generic {
        identifier_index: u8,
        concrete_type: Option<Box<Type>>,
    },
    Integer,
    List {
        item_type: Box<Type>,
        length: usize,
    },
    ListEmpty,
    ListOf {
        item_type: Box<Type>,
    },
    Map {
        pairs: HashMap<u8, Type>,
    },
    None,
    Number,
    Range {
        r#type: Box<Type>,
    },
    SelfChunk,
    String {
        length: Option<usize>,
    },
    Struct(StructType),
    Tuple {
        fields: Option<Vec<Type>>,
    },
}

impl Type {
    /// Returns a concrete type, either the type itself or the concrete type of a generic type.
    pub fn concrete_type(&self) -> &Type {
        if let Type::Generic {
            concrete_type: Some(concrete_type),
            ..
        } = self
        {
            concrete_type.concrete_type()
        } else {
            self
        }
    }

    /// Checks that the type is compatible with another type.
    pub fn check(&self, other: &Type) -> Result<(), TypeConflict> {
        match (self.concrete_type(), other.concrete_type()) {
            (Type::Any, _)
            | (_, Type::Any)
            | (Type::Boolean, Type::Boolean)
            | (Type::Byte, Type::Byte)
            | (Type::Character, Type::Character)
            | (Type::Float, Type::Float)
            | (Type::Integer, Type::Integer)
            | (Type::None, Type::None)
            | (Type::String { .. }, Type::String { .. }) => return Ok(()),
            (
                Type::Generic {
                    concrete_type: left,
                    ..
                },
                Type::Generic {
                    concrete_type: right,
                    ..
                },
            ) => match (left, right) {
                (Some(left), Some(right)) => {
                    if left.check(right).is_ok() {
                        return Ok(());
                    }
                }
                (None, None) => {
                    return Ok(());
                }
                _ => {}
            },
            (Type::Generic { concrete_type, .. }, other)
            | (other, Type::Generic { concrete_type, .. }) => {
                if let Some(concrete_type) = concrete_type {
                    if other == concrete_type.as_ref() {
                        return Ok(());
                    }
                }
            }
            (Type::Struct(left_struct_type), Type::Struct(right_struct_type)) => {
                if left_struct_type == right_struct_type {
                    return Ok(());
                }
            }
            (
                Type::List {
                    item_type: left_type,
                    length: left_length,
                },
                Type::List {
                    item_type: right_type,
                    length: right_length,
                },
            ) => {
                if left_length != right_length {
                    return Err(TypeConflict {
                        actual: other.clone(),
                        expected: self.clone(),
                    });
                }

                if left_type.check(right_type).is_err() {
                    return Err(TypeConflict {
                        actual: other.clone(),
                        expected: self.clone(),
                    });
                }

                return Ok(());
            }
            (
                Type::ListOf {
                    item_type: left_type,
                },
                Type::ListOf {
                    item_type: right_type,
                },
            ) => {
                if left_type.check(right_type).is_err() {
                    return Err(TypeConflict {
                        actual: other.clone(),
                        expected: self.clone(),
                    });
                }
            }
            (
                Type::List {
                    item_type: list_item_type,
                    ..
                },
                Type::ListOf {
                    item_type: list_of_item_type,
                },
            )
            | (
                Type::ListOf {
                    item_type: list_of_item_type,
                },
                Type::List {
                    item_type: list_item_type,
                    ..
                },
            ) => {
                // TODO: This is a hack, remove it.
                if let Type::Any = **list_of_item_type {
                    return Ok(());
                }

                if list_item_type.check(list_of_item_type).is_err() {
                    return Err(TypeConflict {
                        actual: other.clone(),
                        expected: self.clone(),
                    });
                }
            }
            (
                Type::Function(FunctionType {
                    type_parameters: left_type_parameters,
                    value_parameters: left_value_parameters,
                    return_type: left_return,
                }),
                Type::Function(FunctionType {
                    type_parameters: right_type_parameters,
                    value_parameters: right_value_parameters,
                    return_type: right_return,
                }),
            ) => {
                if left_return != right_return
                    || left_type_parameters != right_type_parameters
                    || left_value_parameters != right_value_parameters
                {
                    return Err(TypeConflict {
                        actual: other.clone(),
                        expected: self.clone(),
                    });
                }

                return Ok(());
            }
            (Type::Range { r#type: left_type }, Type::Range { r#type: right_type }) => {
                if left_type == right_type {
                    return Ok(());
                }
            }
            (Type::Number, Type::Number | Type::Integer | Type::Float)
            | (Type::Integer | Type::Float, Type::Number) => {
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
            Type::Any => write!(f, "any"),
            Type::Boolean => write!(f, "bool"),
            Type::Byte => write!(f, "byte"),
            Type::Character => write!(f, "char"),
            Type::Enum(EnumType { name, .. }) => write!(f, "{name}"),
            Type::Float => write!(f, "float"),
            Type::Function(function_type) => write!(f, "{function_type}"),
            Type::Generic { concrete_type, .. } => {
                match concrete_type.clone().map(|r#box| *r#box) {
                    Some(Type::Generic {
                        identifier_index: identifier,
                        ..
                    }) => write!(f, "{identifier}"),
                    Some(concrete_type) => write!(f, "implied to be {concrete_type}"),
                    None => write!(f, "unknown"),
                }
            }
            Type::Integer => write!(f, "int"),
            Type::List { item_type, length } => write!(f, "[{item_type}; {length}]"),
            Type::ListEmpty => write!(f, "[]"),
            Type::ListOf { item_type } => write!(f, "[{item_type}]"),
            Type::Map { pairs } => {
                write!(f, "map ")?;

                write!(f, "{{")?;

                for (index, (key, value)) in pairs.iter().enumerate() {
                    write!(f, "{key}: {value}")?;

                    if index != pairs.len() - 1 {
                        write!(f, ", ")?;
                    }
                }

                write!(f, "}}")
            }
            Type::None => write!(f, "none"),
            Type::Number => write!(f, "num"),
            Type::Range { r#type } => write!(f, "{type} range"),
            Type::SelfChunk => write!(f, "self"),
            Type::String { .. } => write!(f, "str"),
            Type::Struct(struct_type) => write!(f, "{struct_type}"),
            Type::Tuple { fields } => {
                if let Some(fields) = fields {
                    write!(f, "(")?;

                    for (index, r#type) in fields.iter().enumerate() {
                        write!(f, "{type}")?;

                        if index != fields.len() - 1 {
                            write!(f, ", ")?;
                        }
                    }

                    write!(f, ")")
                } else {
                    write!(f, "tuple")
                }
            }
        }
    }
}

impl PartialOrd for Type {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Type {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Type::Any, Type::Any) => Ordering::Equal,
            (Type::Any, _) => Ordering::Greater,
            (Type::Boolean, Type::Boolean) => Ordering::Equal,
            (Type::Boolean, _) => Ordering::Greater,
            (Type::Byte, Type::Byte) => Ordering::Equal,
            (Type::Byte, _) => Ordering::Greater,
            (Type::Character, Type::Character) => Ordering::Equal,
            (Type::Character, _) => Ordering::Greater,
            (Type::Enum(left_enum), Type::Enum(right_enum)) => left_enum.cmp(right_enum),
            (Type::Enum(_), _) => Ordering::Greater,
            (Type::Float, Type::Float) => Ordering::Equal,
            (Type::Float, _) => Ordering::Greater,
            (Type::Function(left_function), Type::Function(right_function)) => {
                left_function.cmp(right_function)
            }
            (Type::Function(_), _) => Ordering::Greater,
            (Type::Generic { .. }, Type::Generic { .. }) => Ordering::Equal,
            (Type::Generic { .. }, _) => Ordering::Greater,
            (Type::Integer, Type::Integer) => Ordering::Equal,
            (Type::Integer, _) => Ordering::Greater,
            (
                Type::List {
                    item_type: left_item_type,
                    length: left_length,
                },
                Type::List {
                    item_type: right_item_type,
                    length: right_length,
                },
            ) => {
                if left_length == right_length {
                    left_item_type.cmp(right_item_type)
                } else {
                    left_length.cmp(right_length)
                }
            }
            (Type::List { .. }, _) => Ordering::Greater,
            (Type::ListEmpty, Type::ListEmpty) => Ordering::Equal,
            (Type::ListEmpty, _) => Ordering::Greater,
            (
                Type::ListOf {
                    item_type: left_item_type,
                },
                Type::ListOf {
                    item_type: right_item_type,
                },
            ) => left_item_type.cmp(right_item_type),
            (Type::ListOf { .. }, _) => Ordering::Greater,
            (Type::Map { pairs: left_pairs }, Type::Map { pairs: right_pairs }) => {
                left_pairs.iter().cmp(right_pairs.iter())
            }
            (Type::Map { .. }, _) => Ordering::Greater,
            (Type::None, Type::None) => Ordering::Equal,
            (Type::None, _) => Ordering::Greater,
            (Type::Number, Type::Number) => Ordering::Equal,
            (Type::Number, _) => Ordering::Greater,
            (Type::Range { r#type: left_type }, Type::Range { r#type: right_type }) => {
                left_type.cmp(right_type)
            }
            (Type::Range { .. }, _) => Ordering::Greater,
            (Type::SelfChunk, Type::SelfChunk) => Ordering::Equal,
            (Type::SelfChunk, _) => Ordering::Greater,
            (Type::String { length: left }, Type::String { length: right }) => left.cmp(right),
            (Type::String { .. }, _) => Ordering::Greater,
            (Type::Struct(left_struct), Type::Struct(right_struct)) => {
                left_struct.cmp(right_struct)
            }
            (Type::Struct(_), _) => Ordering::Greater,

            (Type::Tuple { fields: left }, Type::Tuple { fields: right }) => left.cmp(right),
            (Type::Tuple { .. }, _) => Ordering::Greater,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FunctionType {
    pub type_parameters: Option<Vec<u8>>,
    pub value_parameters: Option<Vec<(u8, Type)>>,
    pub return_type: Box<Type>,
}

impl Display for FunctionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "fn ")?;

        if let Some(type_parameters) = &self.type_parameters {
            write!(f, "<")?;

            for (index, type_parameter) in type_parameters.iter().enumerate() {
                if index > 0 {
                    write!(f, ", ")?;
                }

                write!(f, "{type_parameter}")?;
            }

            write!(f, ">")?;
        }

        write!(f, "(")?;

        if let Some(value_parameters) = &self.value_parameters {
            for (index, (identifier, r#type)) in value_parameters.iter().enumerate() {
                if index > 0 {
                    write!(f, ", ")?;
                }

                write!(f, "{identifier}: {type}")?;
            }
        }

        write!(f, ")")?;

        if *self.return_type != Type::None {
            write!(f, " -> {}", self.return_type)?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum StructType {
    Unit { name: u8 },
    Tuple { name: u8, fields: Vec<Type> },
    Fields { name: u8, fields: HashMap<u8, Type> },
}

impl StructType {
    pub fn name(&self) -> u8 {
        match self {
            StructType::Unit { name } => *name,
            StructType::Tuple { name, .. } => *name,
            StructType::Fields { name, .. } => *name,
        }
    }
}

impl Display for StructType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            StructType::Unit { name } => write!(f, "{name}"),
            StructType::Tuple { name, fields } => {
                write!(f, "{name}(")?;

                for (index, field) in fields.iter().enumerate() {
                    write!(f, "{field}")?;

                    if index != fields.len() - 1 {
                        write!(f, ", ")?;
                    }
                }

                write!(f, ")")
            }
            StructType::Fields { name, fields } => {
                write!(f, "{name} {{")?;

                for (index, (identifier, r#type)) in fields.iter().enumerate() {
                    write!(f, "{identifier}: {type}")?;

                    if index != fields.len() - 1 {
                        write!(f, ", ")?;
                    }
                }

                write!(f, "}}")
            }
        }
    }
}

impl PartialOrd for StructType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StructType {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (StructType::Unit { name: left_name }, StructType::Unit { name: right_name }) => {
                left_name.cmp(right_name)
            }
            (StructType::Unit { .. }, _) => Ordering::Greater,
            (
                StructType::Tuple {
                    name: left_name,
                    fields: left_fields,
                },
                StructType::Tuple {
                    name: right_name,
                    fields: right_fields,
                },
            ) => {
                let name_cmp = left_name.cmp(right_name);

                if name_cmp == Ordering::Equal {
                    left_fields.cmp(right_fields)
                } else {
                    name_cmp
                }
            }
            (StructType::Tuple { .. }, _) => Ordering::Greater,
            (
                StructType::Fields {
                    name: left_name,
                    fields: left_fields,
                },
                StructType::Fields {
                    name: right_name,
                    fields: right_fields,
                },
            ) => {
                let name_cmp = left_name.cmp(right_name);

                if name_cmp == Ordering::Equal {
                    let len_cmp = left_fields.len().cmp(&right_fields.len());

                    if len_cmp == Ordering::Equal {
                        left_fields.iter().cmp(right_fields.iter())
                    } else {
                        len_cmp
                    }
                } else {
                    name_cmp
                }
            }
            (StructType::Fields { .. }, _) => Ordering::Greater,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EnumType {
    pub name: u8,
    pub variants: Vec<StructType>,
}

impl Display for EnumType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let EnumType { name, variants } = self;

        write!(f, "enum {name} {{ ")?;

        for (index, variant) in variants.iter().enumerate() {
            write!(f, "{variant}")?;

            if index != self.variants.len() - 1 {
                write!(f, ", ")?;
            }
        }

        write!(f, " }}")
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TypeConflict {
    pub expected: Type,
    pub actual: Type,
}

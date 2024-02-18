use crate::{Type, TypeDefinition, Value};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum ValueData {
    Value(Value),
    TypeHint(Type),
    TypeDefinition(TypeDefinition),
}

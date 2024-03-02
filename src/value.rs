use std::{
    cmp::Ordering,
    collections::BTreeMap,
    ops::Range,
    sync::{Arc, OnceLock},
};

use crate::{abstract_tree::Identifier, error::ValidationError};

pub static NONE: OnceLock<Value> = OnceLock::new();

#[derive(Clone, Debug, PartialEq)]
pub struct Value(Arc<ValueInner>);

impl Value {
    pub fn inner(&self) -> &Arc<ValueInner> {
        &self.0
    }

    pub fn none() -> Self {
        NONE.get_or_init(|| {
            Value(Arc::new(ValueInner::Enum(
                Identifier::new("Option"),
                Identifier::new("None"),
            )))
        })
        .clone()
    }

    pub fn boolean(boolean: bool) -> Self {
        Value(Arc::new(ValueInner::Boolean(boolean)))
    }

    pub fn float(float: f64) -> Self {
        Value(Arc::new(ValueInner::Float(float)))
    }

    pub fn integer(integer: i64) -> Self {
        Value(Arc::new(ValueInner::Integer(integer)))
    }

    pub fn list(list: Vec<Value>) -> Self {
        Value(Arc::new(ValueInner::List(list)))
    }

    pub fn map(map: BTreeMap<Identifier, Value>) -> Self {
        Value(Arc::new(ValueInner::Map(map)))
    }

    pub fn range(range: Range<i64>) -> Self {
        Value(Arc::new(ValueInner::Range(range)))
    }

    pub fn string<T: ToString>(string: T) -> Self {
        Value(Arc::new(ValueInner::String(string.to_string())))
    }

    pub fn r#enum(name: Identifier, variant: Identifier) -> Self {
        Value(Arc::new(ValueInner::Enum(name, variant)))
    }

    pub fn as_boolean(&self) -> Result<bool, ValidationError> {
        if let ValueInner::Boolean(boolean) = self.0.as_ref() {
            return Ok(*boolean);
        }

        Err(ValidationError::ExpectedBoolean)
    }
}

impl Eq for Value {}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.as_ref().cmp(other.0.as_ref())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValueInner {
    Boolean(bool),
    Float(f64),
    Integer(i64),
    List(Vec<Value>),
    Map(BTreeMap<Identifier, Value>),
    Range(Range<i64>),
    String(String),
    Enum(Identifier, Identifier),
}

impl Eq for ValueInner {}

impl PartialOrd for ValueInner {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ValueInner {
    fn cmp(&self, other: &Self) -> Ordering {
        use ValueInner::*;

        match (self, other) {
            (Boolean(left), Boolean(right)) => left.cmp(right),
            (Boolean(_), _) => Ordering::Greater,
            (Float(left), Float(right)) => left.total_cmp(right),
            (Float(_), _) => Ordering::Greater,
            (Integer(left), Integer(right)) => left.cmp(right),
            (Integer(_), _) => Ordering::Greater,
            (List(left), List(right)) => left.cmp(right),
            (List(_), _) => Ordering::Greater,
            (Map(left), Map(right)) => left.cmp(right),
            (Map(_), _) => Ordering::Greater,
            (Range(left), Range(right)) => {
                let start_cmp = left.start.cmp(&right.start);

                if start_cmp.is_eq() {
                    left.end.cmp(&right.end)
                } else {
                    start_cmp
                }
            }
            (Range(_), _) => Ordering::Greater,
            (String(left), String(right)) => left.cmp(right),
            (String(_), _) => Ordering::Greater,
            (Enum(name_left, variant_left), Enum(name_right, variant_right)) => {
                let name_cmp = name_left.cmp(name_right);

                if name_cmp.is_eq() {
                    variant_left.cmp(variant_right)
                } else {
                    name_cmp
                }
            }
            (Enum(..), _) => Ordering::Greater,
        }
    }
}

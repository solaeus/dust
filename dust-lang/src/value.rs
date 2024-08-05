use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    ops::Range,
    sync::Arc,
};

use chumsky::container::Container;
use serde::{
    de::Visitor,
    ser::{SerializeMap, SerializeSeq, SerializeTuple},
    Deserialize, Deserializer, Serialize,
};

use crate::{identifier::Identifier, Type};

#[derive(Clone, Debug, PartialEq)]
pub struct Value(Arc<ValueInner>);

impl Value {
    pub fn inner(&self) -> &Arc<ValueInner> {
        &self.0
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

    pub fn string<T: ToString>(to_string: T) -> Self {
        Value(Arc::new(ValueInner::String(to_string.to_string())))
    }

    pub fn r#type(&self) -> Type {
        self.0.r#type()
    }

    pub fn as_boolean(&self) -> Option<bool> {
        if let ValueInner::Boolean(boolean) = self.0.as_ref() {
            Some(*boolean)
        } else {
            None
        }
    }

    pub fn as_list(&self) -> Option<&Vec<Value>> {
        if let ValueInner::List(list) = self.inner().as_ref() {
            Some(list)
        } else {
            None
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        if let ValueInner::Integer(integer) = self.inner().as_ref() {
            Some(*integer)
        } else {
            None
        }
    }

    pub fn add(&self, other: &Value) -> Result<Value, ValueError> {
        match (self.inner().as_ref(), other.inner().as_ref()) {
            (ValueInner::Float(left), ValueInner::Float(right)) => Ok(Value::float(left + right)),
            (ValueInner::Integer(left), ValueInner::Integer(right)) => {
                Ok(Value::integer(left + right))
            }
            _ => Err(ValueError::CannotAdd(self.clone(), other.clone())),
        }
    }

    pub fn property_access(&self, property: &Identifier) -> Result<Value, ValueError> {
        match self.inner().as_ref() {
            ValueInner::Map(map) => {
                if let Some(value) = map.get(property) {
                    Ok(value.clone())
                } else {
                    Err(ValueError::PropertyNotFound {
                        value: self.clone(),
                        property: property.clone(),
                    })
                }
            }
            ValueInner::Integer(integer) => match property.as_str() {
                "is_even" => Ok(Value::boolean(integer % 2 == 0)),
                "to_string" => Ok(Value::string(integer.to_string())),
                _ => Err(ValueError::PropertyNotFound {
                    value: self.clone(),
                    property: property.clone(),
                }),
            },
            ValueInner::List(values) => match property.as_str() {
                "length" => Ok(Value::integer(values.len() as i64)),
                _ => Err(ValueError::PropertyNotFound {
                    value: self.clone(),
                    property: property.clone(),
                }),
            },
            _ => todo!(),
        }
    }

    pub fn list_access(&self, index: i64) -> Result<Value, ValueError> {
        match self.inner().as_ref() {
            ValueInner::List(list) => {
                if index < 0 {
                    return Err(ValueError::IndexOutOfBounds {
                        value: self.clone(),
                        index,
                    });
                }

                if let Some(value) = list.get(index as usize) {
                    Ok(value.clone())
                } else {
                    Err(ValueError::IndexOutOfBounds {
                        value: self.clone(),
                        index,
                    })
                }
            }
            _ => Err(ValueError::ExpectedList(self.clone())),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self.inner().as_ref() {
            ValueInner::Boolean(boolean) => write!(f, "{boolean}"),
            ValueInner::Float(float) => {
                write!(f, "{float}")?;

                if &float.floor() == float {
                    write!(f, ".0")?;
                }

                Ok(())
            }
            ValueInner::Integer(integer) => write!(f, "{integer}"),
            ValueInner::List(list) => {
                write!(f, "[")?;

                for (index, value) in list.iter().enumerate() {
                    if index == list.len() - 1 {
                        write!(f, "{}", value)?;
                    } else {
                        write!(f, "{}, ", value)?;
                    }
                }

                write!(f, "]")
            }
            ValueInner::Map(map) => {
                write!(f, "{{ ")?;

                for (index, (key, value)) in map.iter().enumerate() {
                    write!(f, "{key} = {value}")?;

                    if index != map.len() - 1 {
                        write!(f, ", ")?;
                    }
                }

                write!(f, " }}")
            }
            ValueInner::Range(range) => write!(f, "{}..{}", range.start, range.end),
            ValueInner::String(string) => write!(f, "{string}"),
        }
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

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.0.as_ref() {
            ValueInner::Boolean(boolean) => serializer.serialize_bool(*boolean),
            ValueInner::Float(float) => serializer.serialize_f64(*float),
            ValueInner::Integer(integer) => serializer.serialize_i64(*integer),
            ValueInner::List(list) => {
                let mut list_ser = serializer.serialize_seq(Some(list.len()))?;

                for item in list {
                    list_ser.serialize_element(&item)?;
                }

                list_ser.end()
            }
            ValueInner::Map(map) => {
                let mut map_ser = serializer.serialize_map(Some(map.len()))?;

                for (identifier, value) in map {
                    map_ser.serialize_entry(identifier, value)?;
                }

                map_ser.end()
            }
            ValueInner::Range(range) => {
                let mut tuple_ser = serializer.serialize_tuple(2)?;

                tuple_ser.serialize_element(&range.start)?;
                tuple_ser.serialize_element(&range.end)?;

                tuple_ser.end()
            }
            ValueInner::String(string) => serializer.serialize_str(string),
        }
    }
}

struct ValueVisitor;

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .write_str("a boolean, float, function, integer, list, map, range, string or structure")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::boolean(v))
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i64(v as i64)
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i64(v as i64)
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_i64(v as i64)
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::integer(v))
    }

    fn visit_i128<E>(self, _: i128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        todo!()
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_u64(v as u64)
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_u64(v as u64)
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_u64(v as u64)
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::integer(v as i64))
    }

    fn visit_u128<E>(self, _: u128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        todo!()
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_f64(v as f64)
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::float(v))
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(v.encode_utf8(&mut [0u8; 4]))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::string(v))
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(v)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(&v)
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Err(serde::de::Error::invalid_type(
            serde::de::Unexpected::Bytes(v),
            &self,
        ))
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_bytes(v)
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_bytes(&v)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Err(serde::de::Error::invalid_type(
            serde::de::Unexpected::Option,
            &self,
        ))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let _ = deserializer;
        Err(serde::de::Error::invalid_type(
            serde::de::Unexpected::Option,
            &self,
        ))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Err(serde::de::Error::invalid_type(
            serde::de::Unexpected::Unit,
            &self,
        ))
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let _ = deserializer;
        Err(serde::de::Error::invalid_type(
            serde::de::Unexpected::NewtypeStruct,
            &self,
        ))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut list = Vec::with_capacity(seq.size_hint().unwrap_or(10));

        while let Some(element) = seq.next_element()? {
            list.push(element);
        }

        Ok(Value::list(list))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut btree = BTreeMap::with_capacity(map.size_hint().unwrap_or(10));

        while let Some((key, value)) = map.next_entry()? {
            btree.insert(key, value);
        }

        Ok(Value::map(btree))
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        let _ = data;
        Err(serde::de::Error::invalid_type(
            serde::de::Unexpected::Enum,
            &self,
        ))
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor)
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
}

impl ValueInner {
    pub fn r#type(&self) -> Type {
        match self {
            ValueInner::Boolean(_) => Type::Boolean,
            ValueInner::Float(_) => Type::Float,
            ValueInner::Integer(_) => Type::Integer,
            ValueInner::List(values) => {
                let item_type = values.first().unwrap().r#type();

                Type::List {
                    length: values.len(),
                    item_type: Box::new(item_type),
                }
            }
            ValueInner::Map(value_map) => {
                let mut type_map = BTreeMap::with_capacity(value_map.len());

                for (identifier, value) in value_map {
                    let r#type = value.r#type();

                    type_map.insert(identifier.clone(), r#type);
                }

                Type::Map(type_map)
            }
            ValueInner::Range(_) => Type::Range,
            ValueInner::String(_) => Type::String,
        }
    }
}

pub trait ValueProperties<'a> {}

pub struct IntegerProperties<'a>(&'a Value);

impl<'a> IntegerProperties<'a> {
    pub fn is_even(&self) -> bool {
        self.0.as_integer().unwrap() % 2 == 0
    }
}

impl<'a> ValueProperties<'a> for IntegerProperties<'a> {}

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
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValueError {
    CannotAdd(Value, Value),
    PropertyNotFound { value: Value, property: Identifier },
    IndexOutOfBounds { value: Value, index: i64 },
    ExpectedList(Value),
}

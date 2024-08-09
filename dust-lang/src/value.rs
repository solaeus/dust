//! Dust value representation
use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashMap},
    error::Error,
    fmt::{self, Display, Formatter},
    ops::Range,
    sync::Arc,
};

use serde::{
    de::Visitor,
    ser::{SerializeMap, SerializeSeq, SerializeTuple},
    Deserialize, Deserializer, Serialize,
};

use crate::{identifier::Identifier, AbstractSyntaxTree, Type, Vm, VmError};

/// Dust value representation
///
/// Each type of value has a corresponding constructor, here are some simple examples:
///
/// ```
/// # use dust_lang::Value;
/// let boolean = Value::boolean(true);
/// let float = Value::float(3.14);
/// let integer = Value::integer(42);
/// let string = Value::string("Hello, world!");
/// ```
///
/// Values can be combined into more complex values:
///
/// ```
/// # use dust_lang::Value;
/// let list = Value::list(vec![
///     Value::integer(1),
///     Value::integer(2),
///     Value::integer(3),
/// ]);
/// ```
///
/// Values have a type, which can be retrieved using the `type` method:
///
/// ```
/// # use std::collections::HashMap;
/// # use dust_lang::{Type, Value};
/// let variables = HashMap::new();
/// let value = Value::integer(42);
///
/// assert_eq!(value.r#type(&variables), Type::Integer);
/// ```
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

    pub fn function(function: Function) -> Self {
        Value(Arc::new(ValueInner::Function(function)))
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

    pub fn r#type(&self, variables: &HashMap<Identifier, Value>) -> Type {
        self.0.r#type(variables)
    }

    pub fn as_boolean(&self) -> Option<bool> {
        if let ValueInner::Boolean(boolean) = self.0.as_ref() {
            Some(*boolean)
        } else {
            None
        }
    }

    pub fn as_function(&self) -> Option<&Function> {
        if let ValueInner::Function(function) = self.0.as_ref() {
            Some(function)
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
                Ok(Value::integer(left.saturating_add(*right)))
            }
            (ValueInner::String(left), ValueInner::String(right)) => {
                Ok(Value::string(left.to_string() + right))
            }
            _ => Err(ValueError::CannotAdd(self.clone(), other.clone())),
        }
    }

    pub fn subtract(&self, other: &Value) -> Result<Value, ValueError> {
        match (self.inner().as_ref(), other.inner().as_ref()) {
            (ValueInner::Float(left), ValueInner::Float(right)) => Ok(Value::float(left - right)),
            (ValueInner::Integer(left), ValueInner::Integer(right)) => {
                Ok(Value::integer(left.saturating_sub(*right)))
            }
            _ => Err(ValueError::CannotSubtract(self.clone(), other.clone())),
        }
    }

    pub fn multiply(&self, other: &Value) -> Result<Value, ValueError> {
        match (self.inner().as_ref(), other.inner().as_ref()) {
            (ValueInner::Float(left), ValueInner::Float(right)) => Ok(Value::float(left * right)),
            (ValueInner::Integer(left), ValueInner::Integer(right)) => {
                Ok(Value::integer(left * right))
            }
            _ => Err(ValueError::CannotMultiply(self.clone(), other.clone())),
        }
    }

    pub fn less_than(&self, other: &Value) -> Result<Value, ValueError> {
        match (self.inner().as_ref(), other.inner().as_ref()) {
            (ValueInner::Float(left), ValueInner::Float(right)) => Ok(Value::boolean(left < right)),
            (ValueInner::Integer(left), ValueInner::Integer(right)) => {
                Ok(Value::boolean(left < right))
            }
            _ => Err(ValueError::CannotLessThan(self.clone(), other.clone())),
        }
    }

    pub fn less_than_or_equal(&self, other: &Value) -> Result<Value, ValueError> {
        match (self.inner().as_ref(), other.inner().as_ref()) {
            (ValueInner::Float(left), ValueInner::Float(right)) => {
                Ok(Value::boolean(left <= right))
            }
            (ValueInner::Integer(left), ValueInner::Integer(right)) => {
                Ok(Value::boolean(left <= right))
            }
            _ => Err(ValueError::CannotLessThanOrEqual(
                self.clone(),
                other.clone(),
            )),
        }
    }

    pub fn greater_than(&self, other: &Value) -> Result<Value, ValueError> {
        match (self.inner().as_ref(), other.inner().as_ref()) {
            (ValueInner::Float(left), ValueInner::Float(right)) => Ok(Value::boolean(left > right)),
            (ValueInner::Integer(left), ValueInner::Integer(right)) => {
                Ok(Value::boolean(left > right))
            }
            _ => Err(ValueError::CannotGreaterThan(self.clone(), other.clone())),
        }
    }

    pub fn greater_than_or_equal(&self, other: &Value) -> Result<Value, ValueError> {
        match (self.inner().as_ref(), other.inner().as_ref()) {
            (ValueInner::Float(left), ValueInner::Float(right)) => {
                Ok(Value::boolean(left >= right))
            }
            (ValueInner::Integer(left), ValueInner::Integer(right)) => {
                Ok(Value::boolean(left >= right))
            }
            _ => Err(ValueError::CannotGreaterThanOrEqual(
                self.clone(),
                other.clone(),
            )),
        }
    }

    pub fn and(&self, other: &Value) -> Result<Value, ValueError> {
        match (self.inner().as_ref(), other.inner().as_ref()) {
            (ValueInner::Boolean(left), ValueInner::Boolean(right)) => {
                Ok(Value::boolean(*left && *right))
            }
            _ => Err(ValueError::CannotAnd(self.clone(), other.clone())),
        }
    }

    pub fn or(&self, other: &Value) -> Result<Value, ValueError> {
        match (self.inner().as_ref(), other.inner().as_ref()) {
            (ValueInner::Boolean(left), ValueInner::Boolean(right)) => {
                Ok(Value::boolean(*left || *right))
            }
            _ => Err(ValueError::CannotOr(self.clone(), other.clone())),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self.inner().as_ref() {
            ValueInner::Boolean(boolean) => write!(f, "{boolean}"),
            ValueInner::Float(float) => {
                if float == &f64::INFINITY {
                    return write!(f, "Infinity");
                }

                if float == &f64::NEG_INFINITY {
                    return write!(f, "-Infinity");
                }

                write!(f, "{float}")?;

                if &float.floor() == float {
                    write!(f, ".0")?;
                }

                Ok(())
            }
            ValueInner::Function(function) => write!(f, "{function}"),
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
            ValueInner::Function(function) => function.serialize(serializer),
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
        let mut btree = BTreeMap::new();

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
    Function(Function),
    Integer(i64),
    List(Vec<Value>),
    Map(BTreeMap<Identifier, Value>),
    Range(Range<i64>),
    String(String),
}

impl ValueInner {
    pub fn r#type(&self, variables: &HashMap<Identifier, Value>) -> Type {
        match self {
            ValueInner::Boolean(_) => Type::Boolean,
            ValueInner::Float(_) => Type::Float,
            ValueInner::Function(function) => Type::Function {
                type_parameters: function.type_parameters.clone(),
                value_parameters: function.value_parameters.clone(),
                return_type: function.return_type(variables).map(Box::new),
            },
            ValueInner::Integer(_) => Type::Integer,
            ValueInner::List(values) => {
                let item_type = values.first().unwrap().r#type(variables);

                Type::List {
                    length: values.len(),
                    item_type: Box::new(item_type),
                }
            }
            ValueInner::Map(value_map) => {
                let mut type_map = BTreeMap::new();

                for (identifier, value) in value_map {
                    let r#type = value.r#type(variables);

                    type_map.insert(identifier.clone(), r#type);
                }

                Type::Map(type_map)
            }
            ValueInner::Range(_) => Type::Range,
            ValueInner::String(_) => Type::String,
        }
    }
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
            (Function(left), Function(right)) => left.cmp(right),
            (Function(_), _) => Ordering::Greater,
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

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Function {
    pub name: Identifier,
    pub type_parameters: Option<Vec<Type>>,
    pub value_parameters: Option<Vec<(Identifier, Type)>>,
    pub body: AbstractSyntaxTree,
}

impl Function {
    pub fn call(
        self,
        _type_arguments: Option<Vec<Type>>,
        value_arguments: Option<Vec<Value>>,
        variables: &HashMap<Identifier, Value>,
    ) -> Result<Option<Value>, VmError> {
        let mut new_variables = variables.clone();

        if let (Some(value_parameters), Some(value_arguments)) =
            (self.value_parameters, value_arguments)
        {
            for ((identifier, _), value) in value_parameters.into_iter().zip(value_arguments) {
                new_variables.insert(identifier, value);
            }
        }

        let mut vm = Vm::new(self.body);

        vm.run(&mut new_variables)
    }

    pub fn return_type(&self, variables: &HashMap<Identifier, Value>) -> Option<Type> {
        self.body
            .nodes
            .iter()
            .last()
            .unwrap()
            .inner
            .expected_type(variables)
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.name)?;

        if let Some(type_parameters) = &self.type_parameters {
            write!(f, "<")?;

            for (index, type_parameter) in type_parameters.iter().enumerate() {
                if index > 0 {
                    write!(f, ", ")?;
                }

                write!(f, "{}", type_parameter)?;
            }

            write!(f, ">")?;
        }

        write!(f, "(")?;

        if let Some(value_paramers) = &self.value_parameters {
            for (index, (identifier, r#type)) in value_paramers.iter().enumerate() {
                if index > 0 {
                    write!(f, ", ")?;
                }

                write!(f, "{identifier}: {type}")?;
            }
        }

        write!(f, ") {{")?;

        for statement in &self.body.nodes {
            write!(f, "{}", statement)?;
        }

        write!(f, "}}")
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ValueError {
    CannotAdd(Value, Value),
    CannotAnd(Value, Value),
    CannotGreaterThan(Value, Value),
    CannotGreaterThanOrEqual(Value, Value),
    CannotLessThan(Value, Value),
    CannotLessThanOrEqual(Value, Value),
    CannotMultiply(Value, Value),
    CannotSubtract(Value, Value),
    CannotOr(Value, Value),
    ExpectedList(Value),
    IndexOutOfBounds { value: Value, index: i64 },
}

impl Error for ValueError {}

impl Display for ValueError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ValueError::CannotAdd(left, right) => write!(f, "Cannot add {} and {}", left, right),
            ValueError::CannotAnd(left, right) => write!(
                f,
                "Cannot use logical and operation on {} and {}",
                left, right
            ),
            ValueError::CannotMultiply(left, right) => {
                write!(f, "Cannot multiply {} and {}", left, right)
            }
            ValueError::CannotSubtract(left, right) => {
                write!(f, "Cannot subtract {} and {}", left, right)
            }
            ValueError::CannotLessThan(left, right)
            | ValueError::CannotLessThanOrEqual(left, right)
            | ValueError::CannotGreaterThan(left, right)
            | ValueError::CannotGreaterThanOrEqual(left, right) => {
                write!(f, "Cannot compare {} and {}", left, right)
            }
            ValueError::CannotOr(left, right) => {
                write!(
                    f,
                    "Cannot use logical or operation on {} and {}",
                    left, right
                )
            }
            ValueError::IndexOutOfBounds { value, index } => {
                write!(f, "{} does not have an index of {}", value, index)
            }
            ValueError::ExpectedList(value) => write!(f, "{} is not a list", value),
        }
    }
}

use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    ops::Range,
    sync::Arc,
};

use chumsky::container::Container;
use log::{debug, trace};
use serde::{
    de::Visitor,
    ser::{SerializeMap, SerializeSeq, SerializeStruct, SerializeTuple},
    Deserialize, Deserializer, Serialize,
};

use crate::{
    abstract_tree::{
        AbstractNode, Block, BuiltInFunction, Evaluation, SourcePosition, Type,
        WithPosition,
    },
    context::Context,
    error::{RuntimeError, ValidationError},
    identifier::Identifier,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Value(Arc<ValueInner>);

impl Value {
    pub fn inner(&self) -> &Arc<ValueInner> {
        &self.0
    }

    pub fn boolean(boolean: bool) -> Self {
        Value(Arc::new(ValueInner::Boolean(boolean)))
    }

    pub fn built_in_function(function: BuiltInFunction) -> Self {
        Value(Arc::new(ValueInner::BuiltInFunction(function)))
    }

    pub fn enum_instance(
        type_name: Identifier,
        variant: Identifier,
        content: Option<Vec<Value>>,
    ) -> Self {
        Value(Arc::new(ValueInner::EnumInstance {
            type_name,
            variant,
            content,
        }))
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

    pub fn function(
        type_parameters: Option<Vec<Identifier>>,
        value_parameters: Option<Vec<(Identifier, Type)>>,
        return_type: Option<Type>,
        body: Block,
    ) -> Self {
        Value(Arc::new(ValueInner::Function(Function::new(
            type_parameters,
            value_parameters,
            return_type,
            body,
        ))))
    }

    pub fn structure(name: WithPosition<Identifier>, fields: Vec<(Identifier, Value)>) -> Self {
        Value(Arc::new(ValueInner::Structure { name, fields }))
    }

    pub fn r#type(&self, context: &Context) -> Result<Type, ValidationError> {
        self.0.r#type(context)
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
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self.inner().as_ref() {
            ValueInner::Boolean(boolean) => write!(f, "{boolean}"),
            ValueInner::EnumInstance {
                type_name,
                variant,
                content,
            } => {
                if let Some(values) = content {
                    write!(f, "{type_name}::{variant}(")?;

                    for value in values {
                        write!(f, "{value}")?;
                    }

                    write!(f, ")")
                } else {
                    write!(f, "{type_name}::{variant}")
                }
            }
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

                for (index, value) in list.into_iter().enumerate() {
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

                for (index, (key, value)) in map.into_iter().enumerate() {
                    write!(f, "{key} = {value}")?;

                    if index != map.len() - 1 {
                        write!(f, ", ")?;
                    }
                }

                write!(f, " }}")
            }
            ValueInner::Range(range) => write!(f, "{}..{}", range.start, range.end),
            ValueInner::String(string) => write!(f, "{string}"),
            ValueInner::Function(Function {
                type_parameters,
                value_parameters,
                return_type,
                body,
                ..
            }) => {
                write!(f, "fn ")?;

                if let Some(type_parameters) = type_parameters {
                    write!(f, "<")?;

                    for (index, identifier) in type_parameters.into_iter().enumerate() {
                        if index == type_parameters.len() - 1 {
                            write!(f, "{}", identifier)?;
                        } else {
                            write!(f, "{} ", identifier)?;
                        }
                    }

                    write!(f, ">")?;
                }

                write!(f, "(")?;

                if let Some(value_parameters) = value_parameters {
                    for (identifier, r#type) in value_parameters {
                        write!(f, "{identifier}: {}", r#type)?;
                    }
                }

                write!(f, ")")?;

                if let Some(return_type) = return_type {
                    write!(f, " -> {return_type}")?
                }

                write!(f, " {body}")
            }
            ValueInner::Structure { name, fields } => {
                write!(f, "{}\n{{", name.node)?;

                for (key, value) in fields {
                    writeln!(f, "{key} = {value},")?;
                }

                write!(f, "}}")
            }
            ValueInner::BuiltInFunction(built_in_function) => write!(f, "{built_in_function}"),
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
            ValueInner::EnumInstance {
                type_name,
                variant,
                content,
            } => {
                let mut struct_ser = serializer.serialize_struct("EnumInstance", 3)?;

                struct_ser.serialize_field("type_name", type_name)?;
                struct_ser.serialize_field("variant", variant)?;
                struct_ser.serialize_field("content", content)?;

                struct_ser.end()
            }
            ValueInner::Float(float) => serializer.serialize_f64(*float),
            ValueInner::Function(Function {
                type_parameters,
                value_parameters,
                return_type,
                body,
                ..
            }) => {
                let mut struct_ser = serializer.serialize_struct("Function", 4)?;

                struct_ser.serialize_field("type_parameters", type_parameters)?;
                struct_ser.serialize_field("value_parameters", value_parameters)?;
                struct_ser.serialize_field("return_type", return_type)?;
                struct_ser.serialize_field("body", body)?;

                struct_ser.end()
            }
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
            ValueInner::Structure { name, fields } => {
                let mut struct_ser = serializer.serialize_struct("Structure", 2)?;

                struct_ser.serialize_field("name", name)?;
                struct_ser.serialize_field("fields", fields)?;

                struct_ser.end()
            }
            ValueInner::BuiltInFunction(_) => todo!(),
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
    BuiltInFunction(BuiltInFunction),
    EnumInstance {
        type_name: Identifier,
        variant: Identifier,
        content: Option<Vec<Value>>,
    },
    Float(f64),
    Function(Function),
    Integer(i64),
    List(Vec<Value>),
    Map(BTreeMap<Identifier, Value>),
    Range(Range<i64>),
    String(String),
    Structure {
        name: WithPosition<Identifier>,
        fields: Vec<(Identifier, Value)>,
    },
}

impl ValueInner {
    pub fn r#type(&self, context: &Context) -> Result<Type, ValidationError> {
        let r#type = match self {
            ValueInner::Boolean(_) => Type::Boolean,
            ValueInner::EnumInstance { type_name, .. } => {
                if let Some(r#type) = context.get_type(type_name)? {
                    r#type
                } else {
                    return Err(ValidationError::EnumDefinitionNotFound {
                        identifier: type_name.clone(),
                        position: None,
                    });
                }
            }
            ValueInner::Float(_) => Type::Float,
            ValueInner::Integer(_) => Type::Integer,
            ValueInner::List(values) => {
                let item_type = values.first().unwrap().r#type(context)?;

                Type::List {
                    length: values.len(),
                    item_type: Box::new(item_type),
                }
            }
            ValueInner::Map(value_map) => {
                let mut type_map = BTreeMap::with_capacity(value_map.len());

                for (identifier, value) in value_map {
                    let r#type = value.r#type(context)?;

                    type_map.insert(identifier.clone(), r#type);
                }

                Type::Map(type_map)
            }
            ValueInner::Range(_) => Type::Range,
            ValueInner::String(_) => Type::String,
            ValueInner::Function(function) => {
                let return_type = function.return_type.clone().map(|r#type| Box::new(r#type));

                Type::Function {
                    type_parameters: function.type_parameters().clone(),
                    value_parameters: function.value_parameters().clone(),
                    return_type,
                }
            }
            ValueInner::Structure { name, .. } => {
                if let Some(r#type) = context.get_type(&name.node)? {
                    r#type
                } else {
                    return Err(ValidationError::VariableNotFound {
                        identifier: name.node.clone(),
                        position: name.position,
                    });
                }
            }
            ValueInner::BuiltInFunction(function) => function.r#type(),
        };

        Ok(r#type)
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
            (
                EnumInstance {
                    type_name: left_name,
                    variant: left_variant,
                    content: left_content,
                },
                EnumInstance {
                    type_name: right_name,
                    variant: right_variant,
                    content: right_content,
                },
            ) => {
                let name_cmp = left_name.cmp(right_name);

                if name_cmp.is_eq() {
                    let variant_cmp = left_variant.cmp(right_variant);

                    if variant_cmp.is_eq() {
                        left_content.cmp(right_content)
                    } else {
                        variant_cmp
                    }
                } else {
                    name_cmp
                }
            }
            (EnumInstance { .. }, _) => Ordering::Greater,
            (Function(left), Function(right)) => left.cmp(right),
            (Function(_), _) => Ordering::Greater,
            (
                Structure {
                    name: left_name,
                    fields: left_fields,
                },
                Structure {
                    name: right_name,
                    fields: right_fields,
                },
            ) => {
                let name_cmp = left_name.cmp(right_name);

                if name_cmp.is_eq() {
                    left_fields.cmp(right_fields)
                } else {
                    name_cmp
                }
            }
            (Structure { .. }, _) => Ordering::Greater,
            (BuiltInFunction(left), BuiltInFunction(right)) => left.cmp(right),
            (BuiltInFunction(_), _) => Ordering::Greater,
        }
    }
}

#[derive(Debug)]
pub struct Function {
    type_parameters: Option<Vec<Identifier>>,
    value_parameters: Option<Vec<(Identifier, Type)>>,
    return_type: Option<Type>,
    body: Block,
    context: Context,
}

impl Function {
    pub fn new(
        type_parameters: Option<Vec<Identifier>>,
        value_parameters: Option<Vec<(Identifier, Type)>>,
        return_type: Option<Type>,
        body: Block,
    ) -> Self {
        Self {
            type_parameters,
            value_parameters,
            return_type,
            body,
            context: Context::new(),
        }
    }

    pub fn type_parameters(&self) -> &Option<Vec<Identifier>> {
        &self.type_parameters
    }

    pub fn value_parameters(&self) -> &Option<Vec<(Identifier, Type)>> {
        &self.value_parameters
    }

    pub fn body(&self) -> &Block {
        &self.body
    }

    pub fn call(
        self,
        outer_context: Option<&Context>,
        type_arguments: Option<Vec<Type>>,
        value_arguments: Option<Vec<Value>>,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        trace!("Setting function call variables");

        if let Some(outer_context) = outer_context {
            if &self.context == outer_context {
                log::debug!("Recursion detected");
            }

            self.context.inherit_variables_from(outer_context)?;
        }

        if let (Some(type_parameters), Some(type_arguments)) =
            (self.type_parameters, type_arguments)
        {
            for (identifier, r#type) in type_parameters.into_iter().zip(type_arguments.into_iter())
            {
                self.context
                    .set_type(identifier.clone(), r#type, SourcePosition(0, usize::MAX))?;
            }
        }

        if let (Some(value_parameters), Some(value_arguments)) =
            (self.value_parameters, value_arguments)
        {
            for ((identifier, _), value) in value_parameters
                .into_iter()
                .zip(value_arguments.into_iter())
            {
                self.context
                    .set_value(identifier.clone(), value, SourcePosition(0, usize::MAX))?;
            }
        }

        debug!("Calling function");

        self.body
            .evaluate(&self.context, false, SourcePosition(0, usize::MAX))
    }
}

impl Clone for Function {
    fn clone(&self) -> Self {
        Function {
            type_parameters: self.type_parameters.clone(),
            value_parameters: self.value_parameters.clone(),
            return_type: self.return_type.clone(),
            body: self.body.clone(),
            context: Context::new(),
        }
    }
}

impl Eq for Function {}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.type_parameters == other.type_parameters
            && self.value_parameters == other.value_parameters
            && self.return_type == other.return_type
            && self.body == other.body
    }
}

impl PartialOrd for Function {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Function {
    fn cmp(&self, other: &Self) -> Ordering {
        todo!()
    }
}

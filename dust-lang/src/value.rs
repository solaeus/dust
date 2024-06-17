use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    ops::Range,
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use crate::{
    abstract_tree::{AbstractNode, Block, Evaluation, Type, WithPosition},
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

    pub fn float(float: f64) -> Self {
        Value(Arc::new(ValueInner::Float(float)))
    }

    pub fn integer(integer: i64) -> Self {
        Value(Arc::new(ValueInner::Integer(integer)))
    }

    pub fn list(list: Vec<WithPosition<Value>>) -> Self {
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
        value_parameters: Vec<(Identifier, Type)>,
        return_type: Type,
        body: Block,
    ) -> Self {
        Value(Arc::new(ValueInner::Function(Function {
            type_parameters,
            value_parameters,
            return_type,
            body,
        })))
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

    pub fn as_list(&self) -> Option<&Vec<WithPosition<Value>>> {
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
            ValueInner::Float(float) => write!(f, "{float}"),
            ValueInner::Integer(integer) => write!(f, "{integer}"),
            ValueInner::List(list) => {
                write!(f, "[")?;

                for (index, value) in list.into_iter().enumerate() {
                    if index == list.len() - 1 {
                        write!(f, "{}", value.node)?;
                    } else {
                        write!(f, "{}, ", value.node)?;
                    }
                }

                write!(f, "]")
            }
            ValueInner::Map(map) => {
                write!(f, "[")?;

                for (key, value) in map {
                    writeln!(f, "{key} = {value},")?;
                }

                write!(f, "]")
            }
            ValueInner::Range(_) => todo!(),
            ValueInner::String(string) => write!(f, "{string}"),
            ValueInner::Function(Function {
                type_parameters,
                value_parameters: parameters,
                return_type,
                body,
            }) => {
                if let Some(type_parameters) = type_parameters {
                    write!(f, "(")?;

                    for (index, r#type) in type_parameters.into_iter().enumerate() {
                        if index == type_parameters.len() - 1 {
                            write!(f, "{}", r#type)?;
                        } else {
                            write!(f, "{} ", r#type)?;
                        }
                    }

                    write!(f, ")")?;
                }

                write!(f, "(")?;

                for (identifier, r#type) in parameters {
                    write!(f, "{identifier}: {}", r#type)?;
                }

                write!(f, "): {} {:?}", return_type, body)
            }
            ValueInner::Structure { name, fields } => {
                write!(f, "{}\n{{", name.node)?;

                for (key, value) in fields {
                    writeln!(f, "{key} = {value},")?;
                }

                write!(f, "}}")
            }
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
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ValueInner {
    Boolean(bool),
    Float(f64),
    Function(Function),
    Integer(i64),
    List(Vec<WithPosition<Value>>),
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
            ValueInner::Float(_) => Type::Float,
            ValueInner::Integer(_) => Type::Integer,
            ValueInner::List(values) => {
                let item_type = values.first().unwrap().node.r#type(context)?;

                Type::List {
                    length: values.len(),
                    item_type: Box::new(item_type),
                }
            }
            ValueInner::Map(_) => Type::Map,
            ValueInner::Range(_) => Type::Range,
            ValueInner::String(_) => Type::String,
            ValueInner::Function(function) => Type::Function {
                type_parameters: None,
                value_parameters: function.value_parameters.clone(),
                return_type: Box::new(function.return_type.clone()),
            },
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
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Function {
    type_parameters: Option<Vec<Identifier>>,
    value_parameters: Vec<(Identifier, Type)>,
    return_type: Type,
    body: Block,
}

impl Function {
    pub fn type_parameters(&self) -> &Option<Vec<Identifier>> {
        &self.type_parameters
    }

    pub fn call(
        self,
        arguments: Vec<Value>,
        context: &mut Context,
        clear_variables: bool,
    ) -> Result<Evaluation, RuntimeError> {
        for ((identifier, _), value) in self.value_parameters.into_iter().zip(arguments.into_iter())
        {
            context.set_value(identifier.clone(), value)?;
        }

        self.body.evaluate(context, clear_variables)
    }
}

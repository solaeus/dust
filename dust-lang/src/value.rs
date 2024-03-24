use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    ops::Range,
    sync::Arc,
};

use stanza::{
    renderer::{console::Console, Renderer},
    style::{HAlign, MinWidth, Styles},
    table::Table,
};

use crate::{
    abstract_tree::{AbstractNode, Action, Block, Identifier, Type, WithPosition},
    built_in_functions::BuiltInFunction,
    context::Context,
    error::{RuntimeError, ValidationError},
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
        type_arguments: Vec<WithPosition<Type>>,
        parameters: Vec<(Identifier, WithPosition<Type>)>,
        return_type: WithPosition<Type>,
        body: WithPosition<Block>,
    ) -> Self {
        Value(Arc::new(ValueInner::Function(Function::Parsed(
            ParsedFunction {
                type_arguments,
                parameters,
                return_type,
                body,
            },
        ))))
    }

    pub fn structure(name: Identifier, fields: Vec<(Identifier, Value)>) -> Self {
        Value(Arc::new(ValueInner::Structure { name, fields }))
    }

    pub fn built_in_function(function: BuiltInFunction) -> Self {
        Value(Arc::new(ValueInner::Function(Function::BuiltIn(function))))
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
        fn create_table() -> Table {
            Table::with_styles(Styles::default().with(HAlign::Centred).with(MinWidth(3)))
        }

        match self.inner().as_ref() {
            ValueInner::Boolean(boolean) => write!(f, "{boolean}"),
            ValueInner::Float(float) => write!(f, "{float}"),
            ValueInner::Integer(integer) => write!(f, "{integer}"),
            ValueInner::List(list) => {
                let mut table = create_table();

                for value in list {
                    table = table.with_row([value.to_string()]);
                }

                write!(f, "{}", Console::default().render(&table))
            }
            ValueInner::Map(map) => {
                let mut table = create_table();

                for (identifier, value) in map {
                    table = table.with_row([identifier.as_str(), &value.to_string()]);
                }

                write!(f, "{}", Console::default().render(&table))
            }
            ValueInner::Range(_) => todo!(),
            ValueInner::String(string) => write!(f, "{string}"),
            ValueInner::Function(Function::Parsed(ParsedFunction {
                type_arguments,
                parameters,
                return_type,
                body,
            })) => {
                if !type_arguments.is_empty() {
                    write!(f, "(")?;

                    for (index, r#type) in type_arguments.into_iter().enumerate() {
                        if index == type_arguments.len() - 1 {
                            write!(f, "{}", r#type.node)?;
                        } else {
                            write!(f, "{} ", r#type.node)?;
                        }
                    }

                    write!(f, ")")?;
                }

                write!(f, "(")?;

                for (identifier, r#type) in parameters {
                    write!(f, "{identifier}: {}", r#type.node)?;
                }

                write!(f, "): {} {:?}", return_type.node, body.node)
            }
            ValueInner::Function(Function::BuiltIn(built_in_function)) => {
                write!(f, "{built_in_function}")
            }
            ValueInner::Structure { name, fields } => {
                let mut table = create_table();

                for (identifier, value) in fields {
                    table = table.with_row([identifier.as_str(), &value.to_string()]);
                }

                write!(f, "{name}\n{}", Console::default().render(&table))
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
    Structure {
        name: Identifier,
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
                let mut types = Vec::with_capacity(values.len());

                for value in values {
                    types.push(value.r#type(context)?);
                }

                Type::ListExact(types)
            }
            ValueInner::Map(_) => Type::Map,
            ValueInner::Range(_) => Type::Range,
            ValueInner::String(_) => Type::String,
            ValueInner::Function(function) => match function {
                Function::Parsed(parsed_function) => Type::Function {
                    parameter_types: parsed_function
                        .parameters
                        .iter()
                        .map(|(_, r#type)| r#type.node.clone())
                        .collect(),
                    return_type: Box::new(parsed_function.return_type.node.clone()),
                },
                Function::BuiltIn(built_in_function) => {
                    built_in_function.clone().as_value().r#type(context)?
                }
            },
            ValueInner::Structure { name, .. } => {
                if let Some(r#type) = context.get_type(name)? {
                    r#type
                } else {
                    return Err(ValidationError::VariableNotFound(name.clone()));
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

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Function {
    Parsed(ParsedFunction),
    BuiltIn(BuiltInFunction),
}

impl Function {
    pub fn call(self, arguments: Vec<Value>, context: Context) -> Result<Action, RuntimeError> {
        let action = match self {
            Function::Parsed(ParsedFunction {
                parameters, body, ..
            }) => {
                for ((identifier, _), value) in parameters.into_iter().zip(arguments.into_iter()) {
                    context.set_value(identifier.clone(), value)?;
                }

                body.node.run(&context)?
            }
            Function::BuiltIn(built_in_function) => built_in_function.call(arguments, &context)?,
        };

        Ok(action)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct ParsedFunction {
    type_arguments: Vec<WithPosition<Type>>,
    parameters: Vec<(Identifier, WithPosition<Type>)>,
    return_type: WithPosition<Type>,
    body: WithPosition<Block>,
}

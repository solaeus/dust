use std::{cmp::Ordering, collections::BTreeMap, ops::Range};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    Value,
};

use super::{AbstractTree, Action, Expression, Identifier, Statement, Type};

#[derive(Clone, Debug, PartialEq)]
pub enum ValueNode {
    Boolean(bool),
    Float(f64),
    Integer(i64),
    List(Vec<Expression>),
    Map(Vec<(Identifier, Option<Type>, Expression)>),
    Range(Range<i64>),
    String(String),
    Enum(Identifier, Identifier),
    Function {
        parameters: Vec<(Identifier, Type)>,
        return_type: Type,
        body: Box<Statement>,
    },
}

impl AbstractTree for ValueNode {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        let r#type = match self {
            ValueNode::Boolean(_) => Type::Boolean,
            ValueNode::Float(_) => Type::Float,
            ValueNode::Integer(_) => Type::Integer,
            ValueNode::List(items) => {
                let mut item_types = Vec::with_capacity(items.len());

                for expression in items {
                    item_types.push(expression.expected_type(_context)?);
                }

                Type::ListExact(item_types)
            }
            ValueNode::Map(_) => Type::Map,
            ValueNode::Range(_) => Type::Range,
            ValueNode::String(_) => Type::String,
            ValueNode::Enum(name, _) => Type::Custom(name.clone()),
            ValueNode::Function {
                parameters,
                return_type,
                ..
            } => Type::Function {
                parameter_types: parameters
                    .into_iter()
                    .map(|(_, r#type)| r#type)
                    .cloned()
                    .collect(),
                return_type: Box::new(return_type.clone()),
            },
        };

        Ok(r#type)
    }

    fn validate(&self, context: &Context) -> Result<(), ValidationError> {
        if let ValueNode::Map(map_assignments) = self {
            for (_identifier, r#type, expression) in map_assignments {
                if let Some(expected_type) = r#type {
                    let actual_type = expression.expected_type(context)?;

                    expected_type.check(&actual_type)?;
                }
            }
        }

        Ok(())
    }

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        let value = match self {
            ValueNode::Boolean(boolean) => Value::boolean(boolean),
            ValueNode::Float(float) => Value::float(float),
            ValueNode::Integer(integer) => Value::integer(integer),
            ValueNode::List(expression_list) => {
                let mut value_list = Vec::with_capacity(expression_list.len());

                for expression in expression_list {
                    let value = expression.run(_context)?.as_return_value()?;

                    value_list.push(value);
                }

                Value::list(value_list)
            }
            ValueNode::Map(property_list) => {
                let mut property_map = BTreeMap::new();

                for (identifier, _type, expression) in property_list {
                    let value = expression.run(_context)?.as_return_value()?;

                    property_map.insert(identifier, value);
                }

                Value::map(property_map)
            }
            ValueNode::Range(range) => Value::range(range),
            ValueNode::String(string) => Value::string(string),
            ValueNode::Enum(name, variant) => {
                if name.as_str() == "Option" && variant.as_str() == "None" {
                    Value::none()
                } else {
                    Value::r#enum(name, variant)
                }
            }
            ValueNode::Function {
                parameters,
                return_type,
                body,
            } => Value::function(parameters, return_type, *body),
        };

        Ok(Action::Return(value))
    }
}

impl Eq for ValueNode {}

impl PartialOrd for ValueNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ValueNode {
    fn cmp(&self, other: &Self) -> Ordering {
        use ValueNode::*;

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
            (Enum(left_name, left_variant), Enum(right_name, right_variant)) => {
                let name_cmp = left_name.cmp(right_name);

                if name_cmp.is_eq() {
                    left_variant.cmp(right_variant)
                } else {
                    name_cmp
                }
            }
            (Enum(_, _), _) => Ordering::Greater,
            (
                Function {
                    parameters: left_parameters,
                    return_type: left_return,
                    body: left_body,
                },
                Function {
                    parameters: right_parameters,
                    return_type: right_return,
                    body: right_body,
                },
            ) => {
                let parameter_cmp = left_parameters.cmp(right_parameters);

                if parameter_cmp.is_eq() {
                    let return_cmp = left_return.cmp(right_return);

                    if return_cmp.is_eq() {
                        left_body.cmp(right_body)
                    } else {
                        return_cmp
                    }
                } else {
                    parameter_cmp
                }
            }
            (Function { .. }, _) => Ordering::Greater,
        }
    }
}

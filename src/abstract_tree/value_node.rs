use std::{cmp::Ordering, collections::BTreeMap, ops::Range};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    Value,
};

use super::{AbstractTree, Expression, Identifier, Type};

#[derive(Clone, Debug, PartialEq)]
pub enum ValueNode<'src> {
    Boolean(bool),
    Float(f64),
    Integer(i64),
    List(Vec<Expression<'src>>),
    Map(Vec<(Identifier, Expression<'src>)>),
    Range(Range<i64>),
    String(&'src str),
    Enum(Identifier, Identifier),
}

impl<'src> AbstractTree for ValueNode<'src> {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        let r#type = match self {
            ValueNode::Boolean(_) => Type::Boolean,
            ValueNode::Float(_) => Type::Float,
            ValueNode::Integer(_) => Type::Integer,
            ValueNode::List(_) => Type::List,
            ValueNode::Map(_) => Type::Map,
            ValueNode::Range(_) => Type::Range,
            ValueNode::String(_) => Type::String,
            ValueNode::Enum(name, _) => Type::Custom(name.clone()),
        };

        Ok(r#type)
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        todo!()
    }

    fn run(self, _context: &Context) -> Result<Value, RuntimeError> {
        let value = match self {
            ValueNode::Boolean(boolean) => Value::boolean(boolean),
            ValueNode::Float(float) => Value::float(float),
            ValueNode::Integer(integer) => Value::integer(integer),
            ValueNode::List(expression_list) => {
                let mut value_list = Vec::with_capacity(expression_list.len());

                for expression in expression_list {
                    let value = expression.run(_context)?;

                    value_list.push(value);
                }

                Value::list(value_list)
            }
            ValueNode::Map(property_list) => {
                let mut property_map = BTreeMap::new();

                for (identifier, expression) in property_list {
                    let value = expression.run(_context)?;

                    property_map.insert(identifier, value);
                }

                Value::map(property_map)
            }
            ValueNode::Range(range) => Value::range(range),
            ValueNode::String(string) => Value::string(string),
            ValueNode::Enum(name, variant) => Value::r#enum(name, variant),
        };

        Ok(value)
    }
}

impl<'src> Eq for ValueNode<'src> {}

impl<'src> PartialOrd for ValueNode<'src> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'src> Ord for ValueNode<'src> {
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
        }
    }
}

use std::{cmp::Ordering, collections::BTreeMap, ops::Range};

use crate::{context::Context, error::RuntimeError, Value};

use super::{AbstractTree, Expression, Identifier};

#[derive(Clone, Debug, PartialEq)]
pub enum ValueNode<'src> {
    Boolean(bool),
    Float(f64),
    Integer(i64),
    List(Vec<Expression<'src>>),
    Map(BTreeMap<Identifier, Expression<'src>>),
    Range(Range<i64>),
    String(&'src str),
    Enum(Identifier, Identifier),
}

impl<'src> AbstractTree for ValueNode<'src> {
    fn run(self, _context: &Context) -> Result<Value, RuntimeError> {
        let value = match self {
            ValueNode::Boolean(boolean) => Value::boolean(boolean),
            ValueNode::Float(float) => todo!(),
            ValueNode::Integer(integer) => todo!(),
            ValueNode::List(expression_list) => todo!(),
            ValueNode::Map(property_list) => todo!(),
            ValueNode::Range(range) => todo!(),
            ValueNode::String(string) => todo!(),
            ValueNode::Enum(name, variant) => todo!(),
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

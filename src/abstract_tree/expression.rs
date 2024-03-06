use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    Value,
};

use super::{AbstractTree, Identifier, Logic, Type, ValueNode};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Expression<'src> {
    Identifier(Identifier),
    Logic(Box<Logic<'src>>),
    Value(ValueNode<'src>),
}

impl<'src> AbstractTree for Expression<'src> {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        match self {
            Expression::Identifier(identifier) => identifier.expected_type(_context),
            Expression::Logic(logic) => logic.expected_type(_context),
            Expression::Value(value_node) => value_node.expected_type(_context),
        }
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        match self {
            Expression::Identifier(identifier) => identifier.validate(_context),
            Expression::Logic(logic) => logic.validate(_context),
            Expression::Value(value_node) => value_node.validate(_context),
        }
    }

    fn run(self, context: &Context) -> Result<Value, RuntimeError> {
        match self {
            Expression::Identifier(identifier) => identifier.run(context),
            Expression::Logic(logic) => logic.run(context),
            Expression::Value(value_node) => value_node.run(context),
        }
    }
}

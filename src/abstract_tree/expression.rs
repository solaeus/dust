use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractTree, Action, Identifier, Index, Logic, Math, Type, ValueNode};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Expression {
    Identifier(Identifier),
    Index(Box<Index>),
    Logic(Box<Logic>),
    Math(Box<Math>),
    Value(ValueNode),
}

impl AbstractTree for Expression {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        match self {
            Expression::Identifier(identifier) => identifier.expected_type(_context),
            Expression::Index(index) => index.expected_type(_context),
            Expression::Logic(logic) => logic.expected_type(_context),
            Expression::Math(math) => math.expected_type(_context),
            Expression::Value(value_node) => value_node.expected_type(_context),
        }
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        match self {
            Expression::Identifier(identifier) => identifier.validate(_context),
            Expression::Index(index) => index.validate(_context),
            Expression::Logic(logic) => logic.validate(_context),
            Expression::Math(math) => math.validate(_context),
            Expression::Value(value_node) => value_node.validate(_context),
        }
    }

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        match self {
            Expression::Identifier(identifier) => identifier.run(_context),
            Expression::Index(index) => index.run(_context),
            Expression::Logic(logic) => logic.run(_context),
            Expression::Math(math) => math.run(_context),
            Expression::Value(value_node) => value_node.run(_context),
        }
    }
}

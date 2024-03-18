use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{
    AbstractTree, Action, FunctionCall, Identifier, ListIndex, Logic, MapIndex, Math, Type,
    ValueNode,
};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Expression {
    FunctionCall(FunctionCall),
    Identifier(Identifier),
    MapIndex(Box<MapIndex>),
    ListIndex(Box<ListIndex>),
    Logic(Box<Logic>),
    Math(Box<Math>),
    Value(ValueNode),
}

impl AbstractTree for Expression {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        match self {
            Expression::FunctionCall(function_call) => function_call.expected_type(_context),
            Expression::Identifier(identifier) => identifier.expected_type(_context),
            Expression::MapIndex(index) => index.expected_type(_context),
            Expression::ListIndex(_) => todo!(),
            Expression::Logic(logic) => logic.expected_type(_context),
            Expression::Math(math) => math.expected_type(_context),
            Expression::Value(value_node) => value_node.expected_type(_context),
        }
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        match self {
            Expression::FunctionCall(function_call) => function_call.validate(_context),
            Expression::Identifier(identifier) => identifier.validate(_context),
            Expression::MapIndex(index) => index.validate(_context),
            Expression::ListIndex(_) => todo!(),
            Expression::Logic(logic) => logic.validate(_context),
            Expression::Math(math) => math.validate(_context),
            Expression::Value(value_node) => value_node.validate(_context),
        }
    }

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        match self {
            Expression::FunctionCall(function_call) => function_call.run(_context),
            Expression::Identifier(identifier) => identifier.run(_context),
            Expression::MapIndex(index) => index.run(_context),
            Expression::ListIndex(_) => todo!(),
            Expression::Logic(logic) => logic.run(_context),
            Expression::Math(math) => math.run(_context),
            Expression::Value(value_node) => value_node.run(_context),
        }
    }
}

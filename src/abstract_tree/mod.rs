pub mod assignment;
pub mod block;
pub mod expression;
pub mod identifier;
pub mod if_else;
pub mod index;
pub mod logic;
pub mod r#loop;
pub mod math;
pub mod statement;
pub mod r#type;
pub mod value_node;

pub use self::{
    assignment::{Assignment, AssignmentOperator},
    block::Block,
    expression::Expression,
    identifier::Identifier,
    if_else::IfElse,
    index::Index,
    logic::Logic,
    math::Math,
    r#loop::Loop,
    r#type::Type,
    statement::Statement,
    value_node::ValueNode,
};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    Value,
};

pub trait AbstractTree {
    fn expected_type(&self, context: &Context) -> Result<Type, ValidationError>;
    fn validate(&self, context: &Context) -> Result<(), ValidationError>;
    fn run(self, context: &Context) -> Result<Action, RuntimeError>;
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub enum Action {
    Break(Value),
    Return(Value),
    None,
}

impl Action {
    pub fn as_return_value(self) -> Result<Value, ValidationError> {
        if let Action::Return(value) = self {
            Ok(value)
        } else {
            Err(ValidationError::InterpreterExpectedReturn)
        }
    }
}

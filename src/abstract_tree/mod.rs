pub mod assignment;
pub mod block;
pub mod expression;
pub mod function_call;
pub mod identifier;
pub mod if_else;
pub mod index;
pub mod logic;
pub mod r#loop;
pub mod math;
pub mod statement;
pub mod r#type;
pub mod value_node;
pub mod r#while;

pub use self::{
    assignment::{Assignment, AssignmentOperator},
    block::Block,
    expression::Expression,
    function_call::FunctionCall,
    identifier::Identifier,
    if_else::IfElse,
    index::Index,
    logic::Logic,
    math::Math,
    r#loop::Loop,
    r#type::Type,
    r#while::While,
    statement::Statement,
    value_node::ValueNode,
};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    Value,
};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Positioned<T> {
    pub node: T,
    pub position: (usize, usize),
}

pub trait AbstractTree: Sized {
    fn expected_type(&self, context: &Context) -> Result<Type, ValidationError>;
    fn validate(&self, context: &Context) -> Result<(), ValidationError>;
    fn run(self, context: &Context) -> Result<Action, RuntimeError>;

    fn positioned(self, position: (usize, usize)) -> Positioned<Self> {
        Positioned {
            node: self,
            position,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub enum Action {
    Return(Value),
    Break,
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

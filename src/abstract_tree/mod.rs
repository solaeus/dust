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

use chumsky::span::{SimpleSpan, Span};

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Item<A: AbstractTree>(pub A, pub SimpleSpan);

impl<A> Ord for Item<A>
where
    A: AbstractTree + Eq + PartialEq,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let start_cmp = self.1.start().cmp(&other.1.start());

        if start_cmp.is_eq() {
            self.1.end().cmp(&other.1.end())
        } else {
            start_cmp
        }
    }
}

impl<A: AbstractTree> PartialOrd for Item<A>
where
    A: AbstractTree + Eq + PartialEq,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<A: AbstractTree> AbstractTree for Item<A> {
    fn expected_type(&self, context: &Context) -> Result<Type, ValidationError> {
        todo!()
    }

    fn validate(&self, context: &Context) -> Result<(), ValidationError> {
        todo!()
    }

    fn run(self, context: &Context) -> Result<Action, RuntimeError> {
        todo!()
    }
}

pub trait AbstractTree {
    fn expected_type(&self, context: &Context) -> Result<Type, ValidationError>;
    fn validate(&self, context: &Context) -> Result<(), ValidationError>;
    fn run(self, context: &Context) -> Result<Action, RuntimeError>;
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

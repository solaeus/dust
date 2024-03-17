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

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct WithPosition<T> {
    pub node: T,
    pub position: SourcePosition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct SourcePosition(pub usize, pub usize);

impl From<SimpleSpan> for SourcePosition {
    fn from(span: SimpleSpan) -> Self {
        SourcePosition(span.start(), span.end())
    }
}

impl From<(usize, usize)> for SourcePosition {
    fn from((start, end): (usize, usize)) -> Self {
        SourcePosition(start, end)
    }
}

pub trait AbstractTree: Sized {
    fn expected_type(&self, context: &Context) -> Result<Type, ValidationError>;
    fn validate(&self, context: &Context) -> Result<(), ValidationError>;
    fn run(self, context: &Context) -> Result<Action, RuntimeError>;

    fn with_position<T: Into<SourcePosition>>(self, span: T) -> WithPosition<Self> {
        WithPosition {
            node: self,
            position: span.into(),
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

pub mod assignment;
pub mod block;
pub mod expression;
pub mod identifier;
pub mod logic;
pub mod r#loop;
pub mod statement;
pub mod r#type;
pub mod value_node;

pub use self::{
    assignment::Assignment, block::Block, expression::Expression, identifier::Identifier,
    logic::Logic, r#loop::Loop, r#type::Type, statement::Statement, value_node::ValueNode,
};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    Value,
};

pub trait AbstractTree {
    fn expected_type(&self, context: &Context) -> Result<Type, ValidationError>;
    fn validate(&self, context: &Context) -> Result<(), ValidationError>;
    fn run(self, context: &Context) -> Result<Value, RuntimeError>;
}

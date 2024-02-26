pub mod assignment;
pub mod block;
pub mod expression;
pub mod identifier;
pub mod logic;
pub mod r#loop;
pub mod statement;
pub mod value_node;

pub use self::{
    assignment::Assignment, block::Block, expression::Expression, identifier::Identifier,
    logic::Logic, r#loop::Loop, statement::Statement, value_node::ValueNode,
};

use crate::{context::Context, error::RuntimeError, Value};

pub trait AbstractTree {
    fn run(self, context: &Context) -> Result<Value, RuntimeError>;
}

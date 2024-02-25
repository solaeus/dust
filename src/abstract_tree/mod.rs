pub mod assignment;
pub mod block;
pub mod identifier;
pub mod logic;
pub mod r#loop;
pub mod statement;
pub mod value;

pub use self::{
    assignment::Assignment, block::Block, identifier::Identifier, logic::Logic, r#loop::Loop,
    statement::Statement, value::Value,
};

use crate::{context::Context, error::RuntimeError};

pub trait AbstractTree {
    fn run(self, context: &Context) -> Result<Value, RuntimeError>;
}

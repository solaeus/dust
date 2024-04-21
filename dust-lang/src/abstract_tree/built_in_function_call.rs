use std::{
    fmt::{self, Display, Formatter},
    io::stdin,
    thread,
    time::Duration,
};

use crate::{
    abstract_tree::{Action, Type},
    context::Context,
    error::RuntimeError,
};

use super::{AbstractNode, Expression, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum BuiltInFunctionCall {
    ReadLine,
    Sleep(Expression),
    WriteLine(Expression),
}

impl AbstractNode for BuiltInFunctionCall {
    fn expected_type(&self, context: &Context) -> Result<Type, crate::error::ValidationError> {
        todo!()
    }

    fn validate(&self, context: &Context) -> Result<(), crate::error::ValidationError> {
        todo!()
    }

    fn run(self, context: &Context) -> Result<Action, RuntimeError> {
        todo!()
    }
}

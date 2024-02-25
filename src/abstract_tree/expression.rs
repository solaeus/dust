use crate::{context::Context, error::RuntimeError};

use super::{AbstractTree, Identifier, Logic, Value};

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Identifier(Identifier),
    Logic(Box<Logic>),
    Value(Value),
}

impl AbstractTree for Expression {
    fn run(self, context: &Context) -> Result<Value, RuntimeError> {
        match self {
            Expression::Identifier(identifier) => identifier.run(context),
            Expression::Logic(logic) => logic.run(context),
            Expression::Value(value) => value.run(context),
        }
    }
}

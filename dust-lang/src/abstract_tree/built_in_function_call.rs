use std::{io::stdin, thread, time::Duration};

use crate::{
    abstract_tree::{Action, Type},
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
    Value,
};

use super::{AbstractNode, Expression};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum BuiltInFunctionCall {
    ReadLine,
    Sleep(Expression),
    WriteLine(Expression),
}

impl AbstractNode for BuiltInFunctionCall {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        match self {
            BuiltInFunctionCall::ReadLine => Ok(Type::String),
            BuiltInFunctionCall::Sleep(_) => Ok(Type::None),
            BuiltInFunctionCall::WriteLine(_) => Ok(Type::None),
        }
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        Ok(())
    }

    fn run(self, context: &Context) -> Result<Action, RuntimeError> {
        match self {
            BuiltInFunctionCall::ReadLine => {
                let mut buffer = String::new();

                stdin().read_line(&mut buffer)?;

                Ok(Action::Return(Value::string(buffer)))
            }
            BuiltInFunctionCall::Sleep(expression) => {
                let expression_run = expression.clone().run(context)?;
                let expression_value = if let Action::Return(value) = expression_run {
                    value
                } else {
                    return Err(RuntimeError::ValidationFailure(
                        ValidationError::InterpreterExpectedReturn(expression.position()),
                    ));
                };

                if let ValueInner::Integer(milliseconds) = expression_value.inner().as_ref() {
                    thread::sleep(Duration::from_millis(*milliseconds as u64));

                    Ok(Action::None)
                } else {
                    panic!("Expected an integer.");
                }
            }
            BuiltInFunctionCall::WriteLine(_) => todo!(),
        }
    }
}

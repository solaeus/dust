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

    fn validate(&self, _context: &Context, _manage_memory: bool) -> Result<(), ValidationError> {
        match self {
            BuiltInFunctionCall::ReadLine => Ok(()),
            BuiltInFunctionCall::Sleep(expression) => expression.validate(_context, _manage_memory),
            BuiltInFunctionCall::WriteLine(expression) => {
                expression.validate(_context, _manage_memory)
            }
        }
    }

    fn run(self, context: &mut Context, _manage_memory: bool) -> Result<Action, RuntimeError> {
        match self {
            BuiltInFunctionCall::ReadLine => {
                let mut buffer = String::new();

                stdin().read_line(&mut buffer)?;

                Ok(Action::Return(Value::string(buffer)))
            }
            BuiltInFunctionCall::Sleep(expression) => {
                let action = expression.clone().run(context, _manage_memory)?;
                let value = if let Action::Return(value) = action {
                    value
                } else {
                    return Err(RuntimeError::ValidationFailure(
                        ValidationError::InterpreterExpectedReturn(expression.position()),
                    ));
                };

                if let ValueInner::Integer(milliseconds) = value.inner().as_ref() {
                    thread::sleep(Duration::from_millis(*milliseconds as u64));
                }

                Ok(Action::None)
            }
            BuiltInFunctionCall::WriteLine(expression) => {
                let action = expression.clone().run(context, _manage_memory)?;
                let value = if let Action::Return(value) = action {
                    value
                } else {
                    return Err(RuntimeError::ValidationFailure(
                        ValidationError::InterpreterExpectedReturn(expression.position()),
                    ));
                };

                if let ValueInner::String(output) = value.inner().as_ref() {
                    println!("{output}");
                }

                Ok(Action::None)
            }
        }
    }
}

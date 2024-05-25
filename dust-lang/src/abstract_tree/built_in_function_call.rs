use std::{
    fs::read_to_string,
    io::{stdin, stdout, Write},
    thread,
    time::Duration,
};

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
    ReadFile(Expression),
    ReadLine,
    Sleep(Expression),
    WriteLine(Expression),
}

impl AbstractNode for BuiltInFunctionCall {
    fn expected_type(&self, _context: &mut Context) -> Result<Type, ValidationError> {
        match self {
            BuiltInFunctionCall::ReadFile(_) => Ok(Type::String),
            BuiltInFunctionCall::ReadLine => Ok(Type::String),
            BuiltInFunctionCall::Sleep(_) => Ok(Type::None),
            BuiltInFunctionCall::WriteLine(_) => Ok(Type::None),
        }
    }

    fn validate(
        &self,
        _context: &mut Context,
        _manage_memory: bool,
    ) -> Result<(), ValidationError> {
        match self {
            BuiltInFunctionCall::ReadFile(expression) => {
                expression.validate(_context, _manage_memory)
            }
            BuiltInFunctionCall::ReadLine => Ok(()),
            BuiltInFunctionCall::Sleep(expression) => expression.validate(_context, _manage_memory),
            BuiltInFunctionCall::WriteLine(expression) => {
                expression.validate(_context, _manage_memory)
            }
        }
    }

    fn run(self, context: &mut Context, _manage_memory: bool) -> Result<Action, RuntimeError> {
        match self {
            BuiltInFunctionCall::ReadFile(expression) => {
                let action = expression.clone().run(context, _manage_memory)?;
                let value = if let Action::Return(value) = action {
                    value
                } else {
                    return Err(RuntimeError::ValidationFailure(
                        ValidationError::InterpreterExpectedReturn(expression.position()),
                    ));
                };

                let file_contents = if let ValueInner::String(path) = value.inner().as_ref() {
                    read_to_string(path)?
                } else {
                    String::with_capacity(0)
                };

                Ok(Action::Return(Value::string(file_contents)))
            }
            BuiltInFunctionCall::ReadLine => {
                let mut buffer = String::new();

                stdin().read_line(&mut buffer)?;

                Ok(Action::Return(Value::string(
                    buffer.strip_suffix('\n').unwrap_or(&buffer),
                )))
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
                    let mut stdout = stdout();

                    stdout.write_all(output.as_bytes())?;
                    stdout.write(b"\n")?;
                    stdout.flush()?;
                }

                Ok(Action::None)
            }
        }
    }
}

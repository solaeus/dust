use std::{
    fs::read_to_string,
    io::{stdin, stdout, Write},
    thread,
    time::Duration,
};

use serde::{Deserialize, Serialize};

use crate::{
    abstract_tree::{Action, Type},
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
    Value,
};

use super::{AbstractNode, ExpectedType, ValueExpression, WithPosition};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BuiltInFunctionCall {
    JsonParse(WithPosition<Type>, ValueExpression),
    Length(ValueExpression),
    ReadFile(ValueExpression),
    ReadLine,
    Sleep(ValueExpression),
    WriteLine(ValueExpression),
}

impl AbstractNode for BuiltInFunctionCall {
    fn validate(
        &self,
        _context: &mut Context,
        _manage_memory: bool,
    ) -> Result<(), ValidationError> {
        match self {
            BuiltInFunctionCall::JsonParse(_, expression) => {
                expression.validate(_context, _manage_memory)
            }
            BuiltInFunctionCall::Length(expression) => {
                expression.validate(_context, _manage_memory)
            }
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
            BuiltInFunctionCall::JsonParse(_type, expression) => {
                let action = expression.clone().run(context, _manage_memory)?;
                let value = if let Action::Return(value) = action {
                    value
                } else {
                    return Err(RuntimeError::ValidationFailure(
                        ValidationError::InterpreterExpectedReturn(expression.position()),
                    ));
                };

                if let ValueInner::String(string) = value.inner().as_ref() {
                    let deserialized = serde_json::from_str(string)?;

                    Ok(Action::Return(deserialized))
                } else {
                    Err(RuntimeError::ValidationFailure(
                        ValidationError::ExpectedString {
                            actual: value.r#type(context)?,
                            position: expression.position(),
                        },
                    ))
                }
            }
            BuiltInFunctionCall::Length(expression) => {
                let action = expression.clone().run(context, _manage_memory)?;
                let value = if let Action::Return(value) = action {
                    value
                } else {
                    return Err(RuntimeError::ValidationFailure(
                        ValidationError::InterpreterExpectedReturn(expression.position()),
                    ));
                };
                let length = if let ValueInner::List(list) = value.inner().as_ref() {
                    list.len() as i64
                } else {
                    0
                };

                Ok(Action::Return(Value::integer(length)))
            }
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

impl ExpectedType for BuiltInFunctionCall {
    fn expected_type(&self, _context: &mut Context) -> Result<Type, ValidationError> {
        match self {
            BuiltInFunctionCall::JsonParse(r#type, _) => Ok(r#type.node.clone()),
            BuiltInFunctionCall::Length(_) => Ok(Type::Integer),
            BuiltInFunctionCall::ReadFile(_) => Ok(Type::String),
            BuiltInFunctionCall::ReadLine => Ok(Type::String),
            BuiltInFunctionCall::Sleep(_) => Ok(Type::None),
            BuiltInFunctionCall::WriteLine(_) => Ok(Type::None),
        }
    }
}

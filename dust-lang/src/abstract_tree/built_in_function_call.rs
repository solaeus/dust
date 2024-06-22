use std::{
    fs::read_to_string,
    io::{stdin, stdout, Write},
    thread,
    time::Duration,
};

use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    value::ValueInner,
    Value,
};

use super::{AbstractNode, Evaluation, Expression, Type, TypeConstructor};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BuiltInFunctionCall {
    JsonParse(TypeConstructor, Expression),
    Length(Expression),
    ReadFile(Expression),
    ReadLine,
    Sleep(Expression),
    WriteLine(Expression),
}
impl AbstractNode for BuiltInFunctionCall {
    fn define_types(&self, context: &Context) -> Result<(), ValidationError> {
        Ok(())
    }

    fn validate(&self, _context: &Context, _manage_memory: bool) -> Result<(), ValidationError> {
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

    fn evaluate(
        self,
        context: &Context,
        _manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        match self {
            BuiltInFunctionCall::JsonParse(_type, expression) => {
                let action = expression.clone().evaluate(context, _manage_memory)?;
                let value = if let Evaluation::Return(value) = action {
                    value
                } else {
                    return Err(RuntimeError::ValidationFailure(
                        ValidationError::InterpreterExpectedReturn(expression.position()),
                    ));
                };

                if let ValueInner::String(string) = value.inner().as_ref() {
                    let deserialized = serde_json::from_str(string)?;

                    Ok(Some(Evaluation::Return(deserialized)))
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
                let action = expression.clone().evaluate(context, _manage_memory)?;
                let value = if let Evaluation::Return(value) = action {
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

                Ok(Some(Evaluation::Return(Value::integer(length))))
            }
            BuiltInFunctionCall::ReadFile(expression) => {
                let action = expression.clone().evaluate(context, _manage_memory)?;
                let value = if let Evaluation::Return(value) = action {
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

                Ok(Some(Evaluation::Return(Value::string(file_contents))))
            }
            BuiltInFunctionCall::ReadLine => {
                let mut buffer = String::new();

                stdin().read_line(&mut buffer)?;

                Ok(Some(Evaluation::Return(Value::string(
                    buffer.strip_suffix('\n').unwrap_or(&buffer),
                ))))
            }
            BuiltInFunctionCall::Sleep(expression) => {
                let action = expression.clone().evaluate(context, _manage_memory)?;
                let value = if let Evaluation::Return(value) = action {
                    value
                } else {
                    return Err(RuntimeError::ValidationFailure(
                        ValidationError::InterpreterExpectedReturn(expression.position()),
                    ));
                };

                if let ValueInner::Integer(milliseconds) = value.inner().as_ref() {
                    thread::sleep(Duration::from_millis(*milliseconds as u64));
                }

                Ok(Evaluation::Void)
            }
            BuiltInFunctionCall::WriteLine(expression) => {
                let action = expression.clone().evaluate(context, _manage_memory)?;
                let value = if let Evaluation::Return(value) = action {
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

                Ok(Evaluation::Void)
            }
        }
    }

    fn expected_type(&self, context: &Context) -> Result<Option<Type>, ValidationError> {
        match self {
            BuiltInFunctionCall::JsonParse(r#type, _) => {
                Ok(Some(r#type.clone().construct(&context)?))
            }
            BuiltInFunctionCall::Length(_) => Ok(Some(Type::Integer)),
            BuiltInFunctionCall::ReadFile(_) => Ok(Some(Type::String)),
            BuiltInFunctionCall::ReadLine => Ok(Some(Type::String)),
            BuiltInFunctionCall::Sleep(_) => Ok(None),
            BuiltInFunctionCall::WriteLine(_) => Ok(None),
        }
    }
}

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
    fn define_types(&self, _context: &Context) -> Result<(), ValidationError> {
        match self {
            BuiltInFunctionCall::JsonParse(_, expression) => expression.define_types(_context),
            BuiltInFunctionCall::Length(expression) => expression.define_types(_context),
            BuiltInFunctionCall::ReadFile(expression) => expression.define_types(_context),
            BuiltInFunctionCall::ReadLine => Ok(()),
            BuiltInFunctionCall::Sleep(expression) => expression.define_types(_context),
            BuiltInFunctionCall::WriteLine(expression) => expression.define_types(_context),
        }
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
        fn evaluate_expression(
            expression: Expression,
            context: &Context,
            _manage_memory: bool,
        ) -> Result<Value, RuntimeError> {
            let position = expression.position();
            let evaluation = expression.evaluate(context, _manage_memory)?;

            if let Some(Evaluation::Return(value)) = evaluation {
                Ok(value)
            } else {
                Err(RuntimeError::ValidationFailure(
                    ValidationError::ExpectedExpression(position),
                ))
            }
        }

        match self {
            BuiltInFunctionCall::JsonParse(_type, expression) => {
                let position = expression.position();
                let value = evaluate_expression(expression, context, _manage_memory)?;

                if let ValueInner::String(string) = value.inner().as_ref() {
                    let deserialized = serde_json::from_str(&string)?;

                    Ok(Some(Evaluation::Return(deserialized)))
                } else {
                    Err(RuntimeError::ValidationFailure(
                        ValidationError::ExpectedString {
                            actual: value.r#type(context)?,
                            position,
                        },
                    ))
                }
            }
            BuiltInFunctionCall::Length(expression) => {
                let value = evaluate_expression(expression, context, _manage_memory)?;
                let length = if let ValueInner::List(list) = value.inner().as_ref() {
                    list.len() as i64
                } else {
                    todo!("Create an error for this occurence.")
                };

                Ok(Some(Evaluation::Return(Value::integer(length))))
            }
            BuiltInFunctionCall::ReadFile(expression) => {
                let value = evaluate_expression(expression, context, _manage_memory)?;
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
                let value = evaluate_expression(expression, context, _manage_memory)?;

                if let ValueInner::Integer(milliseconds) = value.inner().as_ref() {
                    thread::sleep(Duration::from_millis(*milliseconds as u64));
                }

                Ok(None)
            }
            BuiltInFunctionCall::WriteLine(expression) => {
                let value = evaluate_expression(expression, context, _manage_memory)?;

                if let ValueInner::String(output) = value.inner().as_ref() {
                    let mut stdout = stdout();

                    stdout.write_all(output.as_bytes())?;
                    stdout.write(b"\n")?;
                    stdout.flush()?;
                }

                Ok(None)
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

use std::{fs::read_to_string, io::stdin};

use serde::{Deserialize, Serialize};
use serde_json::from_str;

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    identifier::Identifier,
    value::ValueInner,
    Value,
};

use super::{AbstractNode, Evaluation, Expression, Type, TypeConstructor};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BuiltInFunctionCall {
    Length(Length),
    ReadFile(ReadFile),
    ReadLine(ReadLine),
    Sleep(Sleep),
    WriteLine(WriteLine),
    JsonParse(JsonParse),
}

impl AbstractNode for BuiltInFunctionCall {
    fn define_types(&self, _context: &Context) -> Result<(), ValidationError> {
        Ok(())
    }

    fn validate(&self, _context: &Context, _manage_memory: bool) -> Result<(), ValidationError> {
        Ok(())
    }

    fn evaluate(
        self,
        _context: &Context,
        _manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        match self {
            BuiltInFunctionCall::Length(inner) => inner.call(_context, _manage_memory),
            BuiltInFunctionCall::ReadFile(inner) => inner.call(_context, _manage_memory),
            BuiltInFunctionCall::ReadLine(inner) => inner.call(_context, _manage_memory),
            BuiltInFunctionCall::Sleep(inner) => inner.call(_context, _manage_memory),
            BuiltInFunctionCall::WriteLine(inner) => inner.call(_context, _manage_memory),
            BuiltInFunctionCall::JsonParse(inner) => inner.call(_context, _manage_memory),
        }
    }

    fn expected_type(&self, _context: &Context) -> Result<Option<Type>, ValidationError> {
        match self {
            BuiltInFunctionCall::Length(inner) => inner.return_type(_context),
            BuiltInFunctionCall::ReadFile(inner) => inner.return_type(_context),
            BuiltInFunctionCall::ReadLine(inner) => inner.return_type(_context),
            BuiltInFunctionCall::Sleep(inner) => inner.return_type(_context),
            BuiltInFunctionCall::WriteLine(inner) => inner.return_type(_context),
            BuiltInFunctionCall::JsonParse(inner) => inner.return_type(_context),
        }
    }
}

trait FunctionLogic {
    fn return_type(&self, context: &Context) -> Result<Option<Type>, ValidationError>;
    fn call(
        self,
        context: &Context,
        manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError>;
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Length(Box<Expression>);

impl Length {
    pub fn new(expression: Expression) -> Self {
        Length(Box::new(expression))
    }
}

impl FunctionLogic for Length {
    fn return_type(&self, _: &Context) -> Result<Option<Type>, ValidationError> {
        Ok(Some(Type::Integer))
    }

    fn call(
        self,
        context: &Context,
        manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        let position = self.0.position();
        let evaluation = self.0.evaluate(context, manage_memory)?;
        let value = if let Some(Evaluation::Return(value)) = evaluation {
            value
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::ExpectedExpression(position),
            ));
        };
        let list = if let ValueInner::List(list) = value.inner().as_ref() {
            list
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::ExpectedList {
                    actual: value.r#type(context)?,
                    position,
                },
            ));
        };

        Ok(Some(Evaluation::Return(Value::integer(list.len() as i64))))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ReadFile(Box<Expression>);

impl ReadFile {
    pub fn new(expression: Expression) -> Self {
        ReadFile(Box::new(expression))
    }
}

impl FunctionLogic for ReadFile {
    fn return_type(&self, _: &Context) -> Result<Option<Type>, ValidationError> {
        Ok(Some(Type::String))
    }

    fn call(self, context: &Context, _: bool) -> Result<Option<Evaluation>, RuntimeError> {
        if let Ok(Some(value)) = context.get_value(&Identifier::new("path")) {
            if let ValueInner::String(path) = value.inner().as_ref() {
                let file_content = read_to_string(path)?;

                return Ok(Some(Evaluation::Return(Value::string(file_content))));
            }
        }

        Err(RuntimeError::ValidationFailure(
            ValidationError::BuiltInFunctionFailure(self.0.position()),
        ))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ReadLine;

impl FunctionLogic for ReadLine {
    fn return_type(&self, _: &Context) -> Result<Option<Type>, ValidationError> {
        Ok(Some(Type::String))
    }

    fn call(self, _: &Context, _: bool) -> Result<Option<Evaluation>, RuntimeError> {
        let mut user_input = String::new();

        stdin().read_line(&mut user_input)?;

        Ok(Some(Evaluation::Return(Value::string(
            user_input.trim_end_matches('\n'),
        ))))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Sleep(Box<Expression>);

impl Sleep {
    pub fn new(expression: Expression) -> Self {
        Sleep(Box::new(expression))
    }
}

impl FunctionLogic for Sleep {
    fn return_type(&self, _: &Context) -> Result<Option<Type>, ValidationError> {
        Ok(None)
    }

    fn call(self, _: &Context, _: bool) -> Result<Option<Evaluation>, RuntimeError> {
        todo!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WriteLine(Box<Expression>);

impl WriteLine {
    pub fn new(expression: Expression) -> Self {
        WriteLine(Box::new(expression))
    }
}

impl FunctionLogic for WriteLine {
    fn return_type(&self, _: &Context) -> Result<Option<Type>, ValidationError> {
        Ok(None)
    }

    fn call(
        self,
        context: &Context,
        manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        let position = self.0.position();
        let evaluation = self.0.evaluate(context, manage_memory)?;
        let value = if let Some(Evaluation::Return(value)) = evaluation {
            value
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::ExpectedExpression(position),
            ));
        };

        println!("{value}");

        Ok(None)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct JsonParse(TypeConstructor, Box<Expression>);

impl JsonParse {
    pub fn new(constructor: TypeConstructor, expression: Expression) -> Self {
        JsonParse(constructor, Box::new(expression))
    }
}

impl FunctionLogic for JsonParse {
    fn return_type(&self, context: &Context) -> Result<Option<Type>, ValidationError> {
        self.0.construct(context).map(|r#type| Some(r#type))
    }

    fn call(
        self,
        context: &Context,
        manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        let target_type = self.0.construct(context)?;
        let position = self.1.position();
        let evaluation = self.1.evaluate(context, manage_memory)?;
        let value = if let Some(Evaluation::Return(value)) = evaluation {
            value
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::ExpectedExpression(position),
            ));
        };
        let input = if let ValueInner::String(string) = value.inner().as_ref() {
            string
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::ExpectedString {
                    actual: value.r#type(context)?,
                    position,
                },
            ));
        };

        fn parse_value(input: &str, r#type: Type) -> Result<Value, RuntimeError> {
            let value = match r#type {
                Type::Any => from_str::<Value>(input)?,
                Type::Boolean => Value::boolean(from_str::<bool>(input)?),
                Type::Enum { .. } => todo!(),
                Type::Float => Value::float(from_str::<f64>(input)?),
                Type::Function { .. } => todo!(),
                Type::Generic { concrete_type, .. } => {
                    if let Some(r#type) = concrete_type {
                        parse_value(input, *r#type)?
                    } else {
                        todo!("Create an error for this occurence");
                    }
                }
                Type::Integer => Value::integer(from_str::<i64>(input)?),
                Type::List { .. } => todo!(),
                Type::ListOf(_) => todo!(),
                Type::Map(_) => todo!(),
                Type::Range => todo!(),
                Type::String => Value::string(from_str::<String>(input)?),
                Type::Structure { .. } => todo!(),
            };

            Ok(value)
        }

        let value = parse_value(&input, target_type)?;

        Ok(Some(Evaluation::Return(value)))
    }
}

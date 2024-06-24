use std::{fs::read_to_string, io::stdin, thread::sleep, time::Duration};

use serde::{Deserialize, Serialize};
use serde_json::from_str;

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    identifier::Identifier,
    value::ValueInner,
    Value,
};

use super::Type;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BuiltInFunction {
    Length,
    ReadLine,
    ReadFile,
    Sleep,
    WriteLine,
    JsonParse,
}

impl BuiltInFunction {
    pub fn r#type(&self) -> Type {
        match self {
            BuiltInFunction::Length => Length::r#type(),
            BuiltInFunction::ReadLine => ReadLine::r#type(),
            BuiltInFunction::ReadFile => ReadFile::r#type(),
            BuiltInFunction::Sleep => Sleep::r#type(),
            BuiltInFunction::WriteLine => WriteLine::r#type(),
            BuiltInFunction::JsonParse => JsonParse::r#type(),
        }
    }

    pub fn call(
        &self,
        context: &Context,
        manage_memory: bool,
    ) -> Result<Option<Value>, RuntimeError> {
        match self {
            BuiltInFunction::Length => Length::call(context, manage_memory),
            BuiltInFunction::ReadLine => ReadLine::call(context, manage_memory),
            BuiltInFunction::ReadFile => ReadFile::call(context, manage_memory),
            BuiltInFunction::Sleep => Sleep::call(context, manage_memory),
            BuiltInFunction::WriteLine => WriteLine::call(context, manage_memory),
            BuiltInFunction::JsonParse => JsonParse::call(context, manage_memory),
        }
    }
}

trait FunctionLogic {
    fn r#type() -> Type;
    fn call(context: &Context, manage_memory: bool) -> Result<Option<Value>, RuntimeError>;
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
struct Length;

impl FunctionLogic for Length {
    fn r#type() -> Type {
        Type::Function {
            type_parameters: None,
            value_parameters: Some(vec![(
                Identifier::new("list"),
                Type::ListOf(Box::new(Type::Any)),
            )]),
            return_type: Some(Box::new(Type::Integer)),
        }
    }

    fn call(context: &Context, manage_memory: bool) -> Result<Option<Value>, RuntimeError> {
        let value = if let Some(value) = context.get_value(&Identifier::new("input"))? {
            value
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::BuiltInFunctionFailure("input does not exist"),
            ));
        };
        let list = if let ValueInner::List(list) = value.inner().as_ref() {
            list
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::BuiltInFunctionFailure("list is not a list"),
            ));
        };

        Ok(Some(Value::integer(list.len() as i64)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
struct ReadFile;

impl FunctionLogic for ReadFile {
    fn r#type() -> Type {
        Type::Function {
            type_parameters: None,
            value_parameters: Some(vec![(Identifier::new("path"), Type::String)]),
            return_type: Some(Box::new(Type::String)),
        }
    }

    fn call(context: &Context, manage_memory: bool) -> Result<Option<Value>, RuntimeError> {
        let value = if let Some(value) = context.get_value(&Identifier::new("path"))? {
            value
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::BuiltInFunctionFailure("path does not exist"),
            ));
        };
        let path = if let ValueInner::String(string) = value.inner().as_ref() {
            string
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::BuiltInFunctionFailure("path is not a string"),
            ));
        };
        let file_content = read_to_string(path)?;

        Ok(Some(Value::string(file_content)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
struct ReadLine;

impl FunctionLogic for ReadLine {
    fn r#type() -> Type {
        Type::Function {
            type_parameters: None,
            value_parameters: None,
            return_type: Some(Box::new(Type::String)),
        }
    }

    fn call(context: &Context, manage_memory: bool) -> Result<Option<Value>, RuntimeError> {
        let mut user_input = String::new();

        stdin().read_line(&mut user_input)?;

        Ok(Some(Value::string(user_input)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
struct Sleep;

impl FunctionLogic for Sleep {
    fn r#type() -> Type {
        Type::Function {
            type_parameters: None,
            value_parameters: Some(vec![(Identifier::new("milliseconds"), Type::Integer)]),
            return_type: None,
        }
    }

    fn call(context: &Context, manage_memory: bool) -> Result<Option<Value>, RuntimeError> {
        let value = if let Some(value) = context.get_value(&Identifier::new("milliseconds"))? {
            value
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::BuiltInFunctionFailure("milliseconds does not exist"),
            ));
        };
        let milliseconds = if let ValueInner::Integer(integer) = value.inner().as_ref() {
            integer
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::BuiltInFunctionFailure("milliseconds is not an integer"),
            ));
        };

        sleep(Duration::from_millis(*milliseconds as u64));

        Ok(None)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
struct WriteLine;

impl FunctionLogic for WriteLine {
    fn r#type() -> Type {
        Type::Function {
            type_parameters: None,
            value_parameters: Some(vec![(Identifier::new("output"), Type::String)]),
            return_type: None,
        }
    }

    fn call(context: &Context, manage_memory: bool) -> Result<Option<Value>, RuntimeError> {
        let value = if let Some(value) = context.get_value(&Identifier::new("output"))? {
            value
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::BuiltInFunctionFailure("output does not exist"),
            ));
        };
        let output = if let ValueInner::String(string) = value.inner().as_ref() {
            string
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::BuiltInFunctionFailure("output is not a string"),
            ));
        };

        println!("{output}");

        Ok(None)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
struct JsonParse;

impl FunctionLogic for JsonParse {
    fn r#type() -> Type {
        let type_t = Type::Generic {
            identifier: Identifier::new("T"),
            concrete_type: None,
        };

        Type::Function {
            type_parameters: None,
            value_parameters: Some(vec![(Identifier::new("input"), type_t.clone())]),
            return_type: Some(Box::new(type_t)),
        }
    }

    fn call(context: &Context, manage_memory: bool) -> Result<Option<Value>, RuntimeError> {
        let target_type = if let Some(r#type) = context.get_type(&Identifier::new("T"))? {
            r#type
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::BuiltInFunctionFailure("T does not exist"),
            ));
        };
        let value = if let Some(value) = context.get_value(&Identifier::new("input"))? {
            value
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::BuiltInFunctionFailure("input does not exist"),
            ));
        };
        let input = if let ValueInner::String(string) = value.inner().as_ref() {
            string
        } else {
            return Err(RuntimeError::ValidationFailure(
                ValidationError::BuiltInFunctionFailure("input is not a string"),
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

        parse_value(&input, target_type).map(|value| Some(value))
    }
}

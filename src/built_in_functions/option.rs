use crate::{BuiltInFunction, Map, Result, Type, Value};

pub struct EitherOr;

impl BuiltInFunction for EitherOr {
    fn name(&self) -> &'static str {
        "either_or"
    }

    fn run(&self, arguments: &[Value], _context: &Map) -> Result<Value> {
        let option = arguments.first().unwrap_or_default().as_option()?;
        let value = if let Some(value) = option {
            *value.clone()
        } else {
            arguments.get(1).unwrap_or_default().clone()
        };

        Ok(value)
    }

    fn r#type(&self) -> Type {
        Type::Function {
            parameter_types: vec![Type::Option(Box::new(Type::Any)), Type::Any],
            return_type: Box::new(Type::Boolean),
        }
    }
}

pub struct IsNone;

impl BuiltInFunction for IsNone {
    fn name(&self) -> &'static str {
        "is_none"
    }

    fn run(&self, arguments: &[Value], _context: &Map) -> Result<Value> {
        let option = arguments.first().unwrap_or_default().as_option()?;

        Ok(Value::Boolean(option.is_none()))
    }

    fn r#type(&self) -> Type {
        Type::Function {
            parameter_types: vec![Type::Option(Box::new(Type::Any))],
            return_type: Box::new(Type::Boolean),
        }
    }
}

pub struct IsSome;

impl BuiltInFunction for IsSome {
    fn name(&self) -> &'static str {
        "is_some"
    }

    fn run(&self, arguments: &[Value], _context: &Map) -> Result<Value> {
        let option = arguments.first().unwrap_or_default().as_option()?;

        Ok(Value::Boolean(option.is_some()))
    }

    fn r#type(&self) -> Type {
        Type::Function {
            parameter_types: vec![Type::Option(Box::new(Type::Any))],
            return_type: Box::new(Type::Boolean),
        }
    }
}

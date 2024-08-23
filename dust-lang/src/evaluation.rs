use crate::{Constructor, RuntimeError, Span, Type, Value};

pub enum Evaluation {
    Break(Option<Value>),
    Constructor(Constructor),
    Return(Option<Value>),
}

impl Evaluation {
    pub fn value(self) -> Option<Value> {
        match self {
            Evaluation::Return(value_option) => value_option,
            _ => None,
        }
    }

    pub fn expect_value(self, position: Span) -> Result<Value, RuntimeError> {
        if let Evaluation::Return(Some(value)) = self {
            Ok(value)
        } else {
            Err(RuntimeError::ExpectedValue { position })
        }
    }
}

pub enum TypeEvaluation {
    Break(Option<Type>),
    Constructor(Type),
    Return(Option<Type>),
}

impl TypeEvaluation {
    pub fn r#type(self) -> Option<Type> {
        match self {
            TypeEvaluation::Return(type_option) => type_option,
            _ => None,
        }
    }
}

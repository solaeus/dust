use std::{
    array,
    fs::read_to_string,
    io::{stdin, stdout, Write},
    slice,
    sync::OnceLock,
    thread,
    time::Duration,
};

use rayon::iter::IntoParallelIterator;
use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
    identifier::Identifier,
    value::{Function, ValueInner},
    Value,
};

use super::{
    AbstractNode, Block, Evaluation, Expression, Statement, Type, TypeConstructor, WithPos,
};

pub enum BuiltInExpression {
    Length(BuiltInFunctionCall<Length>),
}

pub struct BuiltInFunctionCall<F> {
    function: F,
    context: Context,
}

pub trait FunctionLogic {
    fn type_parameters() -> Option<impl IntoIterator<Item = (Identifier, Type)>>;
    fn value_parameters() -> impl IntoIterator<Item = Identifier>;
    fn return_type() -> Type;
    fn call(
        self,
        context: &Context,
        manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError>;
}

impl<F: FunctionLogic> AbstractNode for BuiltInFunctionCall<F> {
    fn define_types(&self, _: &Context) -> Result<(), ValidationError> {
        if let Some(type_arguments) = F::type_parameters() {
            for (identifier, r#type) in type_arguments {
                self.context.set_type(identifier, r#type)?;
            }
        }

        Ok(())
    }

    fn validate(&self, _context: &Context, _manage_memory: bool) -> Result<(), ValidationError> {
        Ok(())
    }

    fn evaluate(
        self,
        _: &Context,
        manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        self.function.call(&self.context, manage_memory)
    }

    fn expected_type(&self, _: &Context) -> Result<Option<Type>, ValidationError> {
        Ok(Some(F::return_type()))
    }
}

pub struct Length {
    argument: Expression,
}

impl FunctionLogic for Length {
    fn type_parameters() -> Option<impl IntoIterator<Item = (Identifier, Type)>> {
        None::<array::IntoIter<(Identifier, Type), 0>>
    }

    fn value_parameters() -> impl IntoIterator<Item = Identifier> {
        [(Identifier::new("list"))].into_iter()
    }

    fn return_type() -> Type {
        todo!()
    }

    fn call(
        self,
        context: &Context,
        manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        let position = self.argument.position();
        let evaluation = self.argument.evaluate(context, manage_memory)?;
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

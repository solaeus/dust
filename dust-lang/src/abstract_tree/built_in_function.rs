use std::{
    array,
    fs::read_to_string,
    io::{self, stdin},
};

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
    Length(BuiltInContextBinding<Length>),
    ReadFile(BuiltInContextBinding<ReadFile>),
    ReadLine(BuiltInContextBinding<ReadLine>),
    Sleep(BuiltInContextBinding<Sleep>),
    WriteLine(BuiltInContextBinding<WriteLine>),
    JsonParse(BuiltInContextBinding<JsonParse>),
}

impl BuiltInFunctionCall {
    pub fn length(argument: Expression) -> Self {
        BuiltInFunctionCall::Length(BuiltInContextBinding::new(Length(Box::new(argument))))
    }

    pub fn read_file(argument: Expression) -> Self {
        BuiltInFunctionCall::ReadFile(BuiltInContextBinding::new(ReadFile(Box::new(argument))))
    }

    pub fn read_line() -> Self {
        BuiltInFunctionCall::ReadLine(BuiltInContextBinding::new(ReadLine))
    }

    pub fn sleep(argument: Expression) -> Self {
        BuiltInFunctionCall::Sleep(BuiltInContextBinding::new(Sleep(Box::new(argument))))
    }

    pub fn write_line(argument: Expression) -> Self {
        BuiltInFunctionCall::WriteLine(BuiltInContextBinding::new(WriteLine(Box::new(argument))))
    }

    pub fn json_parse(r#type: TypeConstructor, argument: Expression) -> Self {
        BuiltInFunctionCall::JsonParse(BuiltInContextBinding::new(JsonParse(
            r#type,
            Box::new(argument),
        )))
    }
}

impl AbstractNode for BuiltInFunctionCall {
    fn define_types(&self, _context: &Context) -> Result<(), ValidationError> {
        match self {
            BuiltInFunctionCall::Length(inner) => inner.define_types(_context),
            BuiltInFunctionCall::ReadFile(inner) => inner.define_types(_context),
            BuiltInFunctionCall::ReadLine(inner) => inner.define_types(_context),
            BuiltInFunctionCall::Sleep(inner) => inner.define_types(_context),
            BuiltInFunctionCall::WriteLine(inner) => inner.define_types(_context),
            BuiltInFunctionCall::JsonParse(inner) => inner.define_types(_context),
        }
    }

    fn validate(&self, _context: &Context, _manage_memory: bool) -> Result<(), ValidationError> {
        match self {
            BuiltInFunctionCall::Length(inner) => inner.validate(_context, _manage_memory),
            BuiltInFunctionCall::ReadFile(inner) => inner.validate(_context, _manage_memory),
            BuiltInFunctionCall::ReadLine(inner) => inner.validate(_context, _manage_memory),
            BuiltInFunctionCall::Sleep(inner) => inner.validate(_context, _manage_memory),
            BuiltInFunctionCall::WriteLine(inner) => inner.validate(_context, _manage_memory),
            BuiltInFunctionCall::JsonParse(inner) => inner.validate(_context, _manage_memory),
        }
    }

    fn evaluate(
        self,
        _context: &Context,
        _manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        match self {
            BuiltInFunctionCall::Length(inner) => inner.evaluate(_context, _manage_memory),
            BuiltInFunctionCall::ReadFile(inner) => inner.evaluate(_context, _manage_memory),
            BuiltInFunctionCall::ReadLine(inner) => inner.evaluate(_context, _manage_memory),
            BuiltInFunctionCall::Sleep(inner) => inner.evaluate(_context, _manage_memory),
            BuiltInFunctionCall::WriteLine(inner) => inner.evaluate(_context, _manage_memory),
            BuiltInFunctionCall::JsonParse(inner) => inner.evaluate(_context, _manage_memory),
        }
    }

    fn expected_type(&self, _context: &Context) -> Result<Option<Type>, ValidationError> {
        match self {
            BuiltInFunctionCall::Length(inner) => inner.expected_type(_context),
            BuiltInFunctionCall::ReadFile(inner) => inner.expected_type(_context),
            BuiltInFunctionCall::ReadLine(inner) => inner.expected_type(_context),
            BuiltInFunctionCall::Sleep(inner) => inner.expected_type(_context),
            BuiltInFunctionCall::WriteLine(inner) => inner.expected_type(_context),
            BuiltInFunctionCall::JsonParse(inner) => inner.expected_type(_context),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuiltInContextBinding<F> {
    function: F,
    #[serde(skip)]
    context: Context,
}

impl<F> BuiltInContextBinding<F> {
    pub fn new(function: F) -> Self {
        Self {
            function,
            context: Context::new(None),
        }
    }
}

impl<F: Eq> Eq for BuiltInContextBinding<F> {}

impl<F: PartialEq> PartialEq for BuiltInContextBinding<F> {
    fn eq(&self, other: &Self) -> bool {
        self.function == other.function
    }
}

impl<F: Ord> PartialOrd for BuiltInContextBinding<F> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.function.cmp(&other.function))
    }
}

impl<F: Ord> Ord for BuiltInContextBinding<F> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.function.cmp(&other.function)
    }
}

trait FunctionLogic {
    fn arguments(
        self,
    ) -> (
        Option<impl IntoIterator<Item = (Identifier, TypeConstructor)>>,
        Option<impl IntoIterator<Item = (Identifier, Expression)>>,
    );
    fn return_type(&self, context: &Context) -> Result<Option<Type>, ValidationError>;
    fn call(
        self,
        context: &Context,
        manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError>;
}

impl<F> AbstractNode for BuiltInContextBinding<F>
where
    F: FunctionLogic + Clone,
{
    fn define_types(&self, _: &Context) -> Result<(), ValidationError> {
        let (type_arguments, value_arguments) = self.function.clone().arguments();

        if let Some(type_arguments) = type_arguments {
            for (identifier, constructor) in type_arguments {
                let r#type = constructor.construct(&self.context)?;

                self.context.set_type(identifier, r#type)?;
            }
        }

        if let Some(value_arguments) = value_arguments {
            for (identifier, expression) in value_arguments {
                if let Some(r#type) = expression.expected_type(&self.context)? {
                    self.context.set_type(identifier, r#type)?;
                }
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

    fn expected_type(&self, context: &Context) -> Result<Option<Type>, ValidationError> {
        self.function.return_type(context)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Length(Box<Expression>);

impl FunctionLogic for Length {
    fn arguments(
        self,
    ) -> (
        Option<impl IntoIterator<Item = (Identifier, TypeConstructor)>>,
        Option<impl IntoIterator<Item = (Identifier, Expression)>>,
    ) {
        (
            None::<array::IntoIter<(Identifier, TypeConstructor), 0>>,
            Some([(Identifier::new("list"), *self.0)].into_iter()),
        )
    }

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

impl FunctionLogic for ReadFile {
    fn arguments(
        self,
    ) -> (
        Option<impl IntoIterator<Item = (Identifier, TypeConstructor)>>,
        Option<impl IntoIterator<Item = (Identifier, Expression)>>,
    ) {
        (
            None::<array::IntoIter<(Identifier, TypeConstructor), 0>>,
            Some([(Identifier::new("path"), *self.0)].into_iter()),
        )
    }

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
    fn arguments(
        self,
    ) -> (
        Option<impl IntoIterator<Item = (Identifier, TypeConstructor)>>,
        Option<impl IntoIterator<Item = (Identifier, Expression)>>,
    ) {
        (
            None::<array::IntoIter<(Identifier, TypeConstructor), 0>>,
            None::<array::IntoIter<(Identifier, Expression), 0>>,
        )
    }

    fn return_type(&self, _: &Context) -> Result<Option<Type>, ValidationError> {
        Ok(Some(Type::String))
    }

    fn call(self, _: &Context, _: bool) -> Result<Option<Evaluation>, RuntimeError> {
        let user_input = io::read_to_string(stdin())?;

        Ok(Some(Evaluation::Return(Value::string(user_input))))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Sleep(Box<Expression>);

impl FunctionLogic for Sleep {
    fn arguments(
        self,
    ) -> (
        Option<impl IntoIterator<Item = (Identifier, TypeConstructor)>>,
        Option<impl IntoIterator<Item = (Identifier, Expression)>>,
    ) {
        (
            None::<array::IntoIter<(Identifier, TypeConstructor), 0>>,
            Some([(Identifier::new("milliseconds"), *self.0)].into_iter()),
        )
    }

    fn return_type(&self, _: &Context) -> Result<Option<Type>, ValidationError> {
        Ok(None)
    }

    fn call(self, _: &Context, _: bool) -> Result<Option<Evaluation>, RuntimeError> {
        todo!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WriteLine(Box<Expression>);

impl FunctionLogic for WriteLine {
    fn arguments(
        self,
    ) -> (
        Option<impl IntoIterator<Item = (Identifier, TypeConstructor)>>,
        Option<impl IntoIterator<Item = (Identifier, Expression)>>,
    ) {
        (
            None::<array::IntoIter<(Identifier, TypeConstructor), 0>>,
            Some([(Identifier::new("message"), *self.0)].into_iter()),
        )
    }

    fn return_type(&self, _: &Context) -> Result<Option<Type>, ValidationError> {
        Ok(None)
    }

    fn call(self, context: &Context, _: bool) -> Result<Option<Evaluation>, RuntimeError> {
        let message = context.get_value(&Identifier::new("message"))?;

        if let Some(message) = message {
            println!("{message}");

            Ok(None)
        } else {
            Err(RuntimeError::ValidationFailure(
                ValidationError::BuiltInFunctionFailure(self.0.position()),
            ))
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct JsonParse(TypeConstructor, Box<Expression>);

impl FunctionLogic for JsonParse {
    fn arguments(
        self,
    ) -> (
        Option<impl IntoIterator<Item = (Identifier, TypeConstructor)>>,
        Option<impl IntoIterator<Item = (Identifier, Expression)>>,
    ) {
        (
            Some([(Identifier::new("T"), self.0)].into_iter()),
            Some([(Identifier::new("input"), *self.1)].into_iter()),
        )
    }

    fn return_type(&self, context: &Context) -> Result<Option<Type>, ValidationError> {
        self.0.construct(context).map(|r#type| Some(r#type))
    }

    fn call(self, context: &Context, _: bool) -> Result<Option<Evaluation>, RuntimeError> {
        let target_type = context.get_type(&Identifier::new("T"))?;
        let input = context.get_value(&Identifier::new("input"))?;

        if let (Some(r#type), Some(value)) = (target_type, input) {
            let input_string = if let ValueInner::String(string) = value.inner().as_ref() {
                string
            } else {
                return Err(RuntimeError::ValidationFailure(
                    ValidationError::BuiltInFunctionFailure(self.0.position()),
                ));
            };

            let parsed_value = match r#type {
                Type::Any => from_str::<Value>(input_string)?,
                Type::Boolean => Value::boolean(from_str::<bool>(input_string)?),
                Type::Enum { .. } => todo!(),
                Type::Float => Value::float(from_str::<f64>(input_string)?),
                Type::Function { .. } => todo!(),
                Type::Generic { .. } => todo!(),
                Type::Integer => Value::integer(from_str::<i64>(input_string)?),
                Type::List { .. } => todo!(),
                Type::ListOf(_) => todo!(),
                Type::Map => todo!(),
                Type::Range => todo!(),
                Type::String => Value::string(from_str::<String>(input_string)?),
                Type::Structure { .. } => todo!(),
            };

            return Ok(Some(Evaluation::Return(parsed_value)));
        }

        Err(RuntimeError::ValidationFailure(
            ValidationError::BuiltInFunctionFailure(self.0.position()),
        ))
    }
}

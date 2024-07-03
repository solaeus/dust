pub mod r#as;
pub mod assignment;
pub mod async_block;
pub mod block;
pub mod built_in_function;
pub mod enum_declaration;
pub mod expression;
pub mod function_call;
pub mod if_else;
pub mod list_index;
pub mod logic;
pub mod r#loop;
pub mod map_index;
pub mod math;
pub mod statement;
pub mod structure_definition;
pub mod r#type;
pub mod type_alias;
pub mod type_constructor;
pub mod r#use;
pub mod value_node;
pub mod r#while;

use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
    ops::Index,
};

use chumsky::span::{SimpleSpan, Span};
use serde::{Deserialize, Serialize};

pub use self::{
    assignment::{Assignment, AssignmentOperator},
    async_block::AsyncBlock,
    block::Block,
    built_in_function::BuiltInFunction,
    enum_declaration::{EnumDeclaration, EnumVariant},
    expression::Expression,
    function_call::FunctionCall,
    if_else::IfElse,
    list_index::ListIndex,
    logic::Logic,
    map_index::MapIndex,
    math::Math,
    r#as::As,
    r#loop::Loop,
    r#type::Type,
    r#use::Use,
    r#while::While,
    statement::Statement,
    structure_definition::StructureDefinition,
    type_alias::TypeAlias,
    type_constructor::{FunctionTypeConstructor, ListTypeConstructor, TypeConstructor},
    value_node::ValueNode,
};

use crate::{
    context::Context,
    error::{DustError, RuntimeError, ValidationError},
    Value,
};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WithPosition<T> {
    pub node: T,
    pub position: SourcePosition,
}

pub trait WithPos: Sized {
    fn with_position<T: Into<SourcePosition>>(self, span: T) -> WithPosition<Self> {
        WithPosition {
            node: self,
            position: span.into(),
        }
    }
}

impl<T> WithPos for T {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SourcePosition(pub usize, pub usize);

impl Display for SourcePosition {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

impl From<SimpleSpan> for SourcePosition {
    fn from(span: SimpleSpan) -> Self {
        SourcePosition(span.start(), span.end())
    }
}

impl From<(usize, usize)> for SourcePosition {
    fn from((start, end): (usize, usize)) -> Self {
        SourcePosition(start, end)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub enum Evaluation {
    Break,
    Continue,
    Return(Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbstractTree(Vec<Statement>);

impl AbstractTree {
    pub fn new(mut statements: Vec<Statement>) -> Self {
        statements.sort_by(|left, right| match (&left, &right) {
            (Statement::StructureDefinition(_), _) => Ordering::Less,
            (_, Statement::StructureDefinition(_)) => Ordering::Greater,
            (_, _) => Ordering::Equal,
        });

        AbstractTree(statements)
    }

    pub fn run(
        self,
        context: &Context,
        manage_memory: bool,
    ) -> Result<Option<Value>, Vec<DustError>> {
        let mut errors = Vec::new();

        for statement in &self.0 {
            let validation_result = statement.define_and_validate(context, manage_memory);

            if let Err(error) = validation_result {
                errors.push(DustError::Validation {
                    error,
                    position: statement.position(),
                });
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        let mut previous_value = None;

        for statement in self.0 {
            let position = statement.position();
            let run = statement.evaluate(context, manage_memory);

            match run {
                Ok(evaluation) => match evaluation {
                    Some(Evaluation::Return(value)) => previous_value = Some(value),
                    None => previous_value = None,
                    _ => {}
                },
                Err(runtime_error) => {
                    return Err(vec![DustError::Runtime {
                        error: runtime_error,
                        position,
                    }]);
                }
            }
        }

        Ok(previous_value)
    }
}

impl Index<usize> for AbstractTree {
    type Output = Statement;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl AbstractNode for AbstractTree {
    fn define_and_validate(
        &self,
        context: &Context,
        manage_memory: bool,
    ) -> Result<(), ValidationError> {
        for statement in &self.0 {
            statement.define_and_validate(context, manage_memory)?;
        }

        Ok(())
    }

    fn evaluate(
        self,
        context: &Context,
        manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError> {
        let mut previous = None;

        for statement in self.0 {
            previous = statement.evaluate(context, manage_memory)?;
        }

        Ok(previous)
    }

    fn expected_type(&self, context: &Context) -> Result<Option<Type>, ValidationError> {
        self.0.last().unwrap().expected_type(context)
    }
}

pub trait AbstractNode {
    fn define_and_validate(
        &self,
        context: &Context,
        manage_memory: bool,
    ) -> Result<(), ValidationError>;

    fn evaluate(
        self,
        context: &Context,
        manage_memory: bool,
    ) -> Result<Option<Evaluation>, RuntimeError>;

    fn expected_type(&self, context: &Context) -> Result<Option<Type>, ValidationError>;
}

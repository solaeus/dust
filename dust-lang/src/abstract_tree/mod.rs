pub mod r#as;
pub mod assignment;
pub mod async_block;
pub mod block;
pub mod built_in_function_call;
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
pub mod type_assignment;
pub mod type_constructor;
pub mod value_node;
pub mod r#while;

use std::{cmp::Ordering, ops::Index};

use chumsky::span::{SimpleSpan, Span};
use serde::{Deserialize, Serialize};

pub use self::{
    assignment::{Assignment, AssignmentOperator},
    async_block::AsyncBlock,
    block::Block,
    built_in_function_call::BuiltInFunctionCall,
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
    r#while::While,
    statement::Statement,
    structure_definition::StructureDefinition,
    type_assignment::TypeAssignment,
    type_constructor::{
        EnumTypeConstructor, FunctionTypeConstructor, ListTypeConstructor, TypeConstructor,
    },
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
    Return(Value),
    Break,
    None,
}

#[derive(Debug, Clone)]
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
        context: &mut Context,
        manage_memory: bool,
    ) -> Result<Option<Value>, Vec<DustError>> {
        let valid_statements = self.validate(context, manage_memory)?;
        let mut previous_value = None;

        for statement in valid_statements {
            let position = statement.position();
            let run = statement.evaluate(context, manage_memory);

            match run {
                Ok(action) => match action {
                    Evaluation::Return(value) => previous_value = Some(value),
                    Evaluation::None => previous_value = None,
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

    fn validate(
        self,
        context: &mut Context,
        manage_memory: bool,
    ) -> Result<Vec<Statement>, Vec<DustError>> {
        let mut errors = Vec::new();
        let mut valid_statements = Vec::new();

        for statement in self.0 {
            let validation = statement.validate(context, manage_memory);

            if let Err(validation_error) = validation {
                errors.push(DustError::Validation {
                    error: validation_error,
                    position: statement.position(),
                })
            } else if errors.is_empty() {
                if let Statement::StructureDefinition(_) = statement {
                    let position = statement.position();
                    let run = statement.evaluate(context, true);

                    if let Err(runtime_error) = run {
                        errors.push(DustError::Runtime {
                            error: runtime_error,
                            position,
                        });

                        return Err(errors);
                    }
                } else {
                    valid_statements.push(statement)
                }
            } else {
                continue;
            }
        }

        if errors.is_empty() {
            Ok(valid_statements)
        } else {
            Err(errors)
        }
    }
}

impl Index<usize> for AbstractTree {
    type Output = Statement;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

pub trait Evaluate: Sized {
    fn validate(&self, context: &mut Context, manage_memory: bool) -> Result<(), ValidationError>;
    fn evaluate(
        self,
        context: &mut Context,
        manage_memory: bool,
    ) -> Result<Evaluation, RuntimeError>;
}

pub trait ExpectedType {
    fn expected_type(&self, context: &mut Context) -> Result<Type, ValidationError>;
}

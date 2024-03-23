pub mod assignment;
pub mod async_block;
pub mod block;
pub mod expression;
pub mod function_call;
pub mod identifier;
pub mod if_else;
pub mod list_index;
pub mod logic;
pub mod r#loop;
pub mod map_index;
pub mod math;
pub mod statement;
pub mod structure_definition;
pub mod r#type;
pub mod value_node;
pub mod r#while;

use std::{cmp::Ordering, ops::Index};

use chumsky::span::{SimpleSpan, Span};

pub use self::{
    assignment::{Assignment, AssignmentOperator},
    async_block::AsyncBlock,
    block::Block,
    expression::Expression,
    function_call::FunctionCall,
    identifier::Identifier,
    if_else::IfElse,
    list_index::ListIndex,
    logic::Logic,
    map_index::MapIndex,
    math::Math,
    r#loop::Loop,
    r#type::Type,
    r#while::While,
    statement::Statement,
    structure_definition::StructureDefinition,
    value_node::ValueNode,
};

use crate::{
    context::Context,
    error::{Error, RuntimeError, ValidationError},
    Value,
};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct WithPosition<T> {
    pub node: T,
    pub position: SourcePosition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
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
pub enum Action {
    Return(Value),
    Break,
    None,
}

pub struct AbstractTree(Vec<WithPosition<Statement>>);

impl AbstractTree {
    pub fn new(mut statements: Vec<WithPosition<Statement>>) -> Self {
        statements.sort_by(|left, right| match (&left.node, &right.node) {
            (Statement::StructureDefinition(_), _) => Ordering::Less,
            (_, Statement::StructureDefinition(_)) => Ordering::Greater,
            (_, _) => Ordering::Equal,
        });

        AbstractTree(statements)
    }

    pub fn run(self, context: &Context) -> Result<Option<Value>, Vec<Error>> {
        let valid_statements = self.run_type_definitions(context)?;
        let mut previous_value = None;

        for statement in valid_statements {
            let run = statement.node.run(context);

            match run {
                Ok(action) => match action {
                    Action::Return(value) => previous_value = Some(value),
                    _ => {}
                },
                Err(runtime_error) => {
                    return Err(vec![Error::Runtime {
                        error: runtime_error,
                        position: statement.position,
                    }]);
                }
            }
        }

        Ok(previous_value)
    }

    fn run_type_definitions(
        self,
        context: &Context,
    ) -> Result<Vec<WithPosition<Statement>>, Vec<Error>> {
        let mut errors = Vec::new();
        let mut valid_statements = Vec::new();

        for statement in self.0 {
            let validation = statement.node.validate(context);

            if let Err(validation_error) = validation {
                errors.push(Error::Validation {
                    error: validation_error,
                    position: statement.position,
                })
            } else if errors.is_empty() {
                if let Statement::StructureDefinition(_) = statement.node {
                    let run = statement.node.run(context);

                    if let Err(runtime_error) = run {
                        errors.push(Error::Runtime {
                            error: runtime_error,
                            position: statement.position,
                        });

                        return Err(errors);
                    }
                } else {
                    valid_statements.push(statement)
                }
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
    type Output = WithPosition<Statement>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

pub trait AbstractNode: Sized {
    fn expected_type(&self, context: &Context) -> Result<Type, ValidationError>;
    fn validate(&self, context: &Context) -> Result<(), ValidationError>;
    fn run(self, context: &Context) -> Result<Action, RuntimeError>;

    fn with_position<T: Into<SourcePosition>>(self, span: T) -> WithPosition<Self> {
        WithPosition {
            node: self,
            position: span.into(),
        }
    }
}

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

use std::ops::Index;

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
    pub fn new(statements: Vec<WithPosition<Statement>>) -> Self {
        AbstractTree(statements)
    }

    pub fn run(self, context: &Context) -> Result<Option<Value>, Vec<Error>> {
        let mut valid_statements = Vec::with_capacity(self.0.len());
        let mut errors = Vec::new();

        for statement in self.0 {
            let validation = statement.node.validate(context);

            if let Statement::StructureDefinition(_) = statement.node {
                match validation {
                    Ok(_) => {
                        let run_result = statement.node.run(context);

                        match run_result {
                            Ok(_) => {}
                            Err(runtime_error) => {
                                return Err(vec![Error::Runtime {
                                    error: runtime_error,
                                    position: statement.position,
                                }]);
                            }
                        }
                    }
                    Err(validation_error) => errors.push(Error::Validation {
                        error: validation_error,
                        position: statement.position,
                    }),
                }
            } else {
                match validation {
                    Ok(_) => valid_statements.push(statement),
                    Err(validation_error) => errors.push(Error::Validation {
                        error: validation_error,
                        position: statement.position,
                    }),
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        let mut previous = None;

        for statement in valid_statements {
            let run_result = statement.node.run(context);

            match run_result {
                Ok(action) => match action {
                    Action::Return(value) => previous = Some(value),
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

        Ok(previous)
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

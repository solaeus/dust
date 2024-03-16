use chumsky::span::{SimpleSpan, Span};

use crate::{
    context::Context,
    error::{RuntimeError, ValidationError},
};

use super::{AbstractTree, Action, Assignment, Block, Expression, IfElse, Loop, Type, While};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Statement {
    pub inner: StatementInner,
    pub span: (usize, usize),
}

impl Statement {
    pub fn assignment(assignment: Assignment, span: SimpleSpan) -> Self {
        Statement {
            inner: StatementInner::Assignment(assignment),
            span: (span.start(), span.end()),
        }
    }

    pub fn block(block: Block, span: SimpleSpan) -> Self {
        Statement {
            inner: StatementInner::Block(block),
            span: (span.start(), span.end()),
        }
    }

    pub fn r#break(span: SimpleSpan) -> Self {
        Statement {
            inner: StatementInner::Break,
            span: (span.start(), span.end()),
        }
    }

    pub fn expression(expression: Expression, span: SimpleSpan) -> Self {
        Statement {
            inner: StatementInner::Expression(expression),
            span: (span.start(), span.end()),
        }
    }

    pub fn if_else(if_else: IfElse, span: SimpleSpan) -> Self {
        Statement {
            inner: StatementInner::IfElse(if_else),
            span: (span.start(), span.end()),
        }
    }

    pub fn r#loop(r#loop: Loop, span: SimpleSpan) -> Self {
        Statement {
            inner: StatementInner::Loop(r#loop),
            span: (span.start(), span.end()),
        }
    }

    pub fn r#while(r#while: While, span: SimpleSpan) -> Self {
        Statement {
            inner: StatementInner::While(r#while),
            span: (span.start(), span.end()),
        }
    }

    pub fn span(&self) -> (usize, usize) {
        self.span
    }

    pub fn inner(&self) -> &StatementInner {
        &self.inner
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum StatementInner {
    Assignment(Assignment),
    Block(Block),
    Break,
    Expression(Expression),
    IfElse(IfElse),
    Loop(Loop),
    While(While),
}

impl AbstractTree for Statement {
    fn expected_type(&self, _context: &Context) -> Result<Type, ValidationError> {
        match &self.inner {
            StatementInner::Assignment(assignment) => assignment.expected_type(_context),
            StatementInner::Block(block) => block.expected_type(_context),
            StatementInner::Break => Ok(Type::None),
            StatementInner::Expression(expression) => expression.expected_type(_context),
            StatementInner::IfElse(if_else) => if_else.expected_type(_context),
            StatementInner::Loop(r#loop) => r#loop.expected_type(_context),
            StatementInner::While(r#while) => r#while.expected_type(_context),
        }
    }

    fn validate(&self, _context: &Context) -> Result<(), ValidationError> {
        match &self.inner {
            StatementInner::Assignment(assignment) => assignment.validate(_context),
            StatementInner::Block(block) => block.validate(_context),
            StatementInner::Break => Ok(()),
            StatementInner::Expression(expression) => expression.validate(_context),
            StatementInner::IfElse(if_else) => if_else.validate(_context),
            StatementInner::Loop(r#loop) => r#loop.validate(_context),
            StatementInner::While(r#while) => r#while.validate(_context),
        }
    }

    fn run(self, _context: &Context) -> Result<Action, RuntimeError> {
        match self.inner {
            StatementInner::Assignment(assignment) => assignment.run(_context),
            StatementInner::Block(block) => block.run(_context),
            StatementInner::Break => Ok(Action::Break),
            StatementInner::Expression(expression) => expression.run(_context),
            StatementInner::IfElse(if_else) => if_else.run(_context),
            StatementInner::Loop(r#loop) => r#loop.run(_context),
            StatementInner::While(r#while) => r#while.run(_context),
        }
    }
}

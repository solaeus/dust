use crate::{Scope, Span, Type};

#[derive(Clone, Debug)]
pub struct Expression {
    pub index: ExpressionIndex,
    pub kind: ExpressionKind,
    pub ends_with_value: bool,
    pub r#type: Type,
    pub scope: Scope,
    pub position: Span,
}

#[derive(Clone, Copy, Debug)]
pub enum ExpressionIndex {
    Instruction(usize),
    Function(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExpressionKind {
    Assignment,
    Binary,
    Block,
    Call,
    ControlFlow,
    Function,
    ListCreation,
    Literal,
    Return,
    Unary,
    Variable,
}

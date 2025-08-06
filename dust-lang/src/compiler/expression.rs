use crate::{Span, Type};

#[derive(Clone, Debug)]
pub struct Expression {
    pub index: ExpressionIndex,
    pub _kind: ExpressionKind,
    pub r#type: Type,
    pub position: Span,
}

#[derive(Clone, Copy, Debug)]
pub enum ExpressionIndex {
    Instruction(usize),
    Function(usize),
}

#[derive(Clone, Copy, Debug)]
pub enum ExpressionKind {
    Binary,
    Call,
    ControlFlow,
    Function,
    List,
    Literal,
    Return,
    Unary,
    Variable,
}

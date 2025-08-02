use crate::{Span, Type};

#[derive(Clone, Debug)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub r#type: Type,
    pub position: Span,
}

#[derive(Clone, Copy, Debug)]
pub enum ExpressionKind {
    Instruction { instruction_index: usize },
    Function { prototype_index: usize },
}

use crate::{Identifier, Span, Value};

#[derive(Debug, PartialEq, Clone)]
pub struct Instruction {
    pub operation: Operation,
    pub span: Span,
}

impl Instruction {
    pub fn new(operation: Operation, span: Span) -> Self {
        Self { operation, span }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operation {
    Add(Box<(Instruction, Instruction)>),
    Assign(Box<(Instruction, Instruction)>),
    Constant(Value),
    Identifier(Identifier),
    List(Vec<Instruction>),
    Multiply(Box<(Instruction, Instruction)>),
}

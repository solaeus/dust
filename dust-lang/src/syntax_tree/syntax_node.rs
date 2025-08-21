use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::Span;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SyntaxNode {
    pub kind: SyntaxKind,
    pub span: Span,
    pub child: u32,
    pub payload: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyntaxKind {
    // Statements
    MainFunctionStatement,
    ExpressionStatement,
    FunctionStatement,
    LetStatement,
    SemicolonStatement,

    // Literal Expressions
    BooleanExpression,
    ByteExpression,
    CharacterExpression,
    FloatExpression,
    IntegerExpression,
    StringExpression,

    // Math Expressions
    AdditionExpression,
    SubtractionExpression,
    MultiplicationExpression,
    DivisionExpression,
    ModuloExpression,

    ArrayExpression,
    ArrayIndexExpression,
    BlockExpression,
    CallExpression,
    FunctionExpression,
    GroupedExpression,
    IfExpression,
    OperatorExpression,
    PathExpression,
    PredicateLoopExpression,
    ReturnExpression,
}

impl SyntaxKind {
    pub fn is_statement(self) -> bool {
        matches!(
            self,
            SyntaxKind::MainFunctionStatement
                | SyntaxKind::ExpressionStatement
                | SyntaxKind::FunctionStatement
                | SyntaxKind::LetStatement
                | SyntaxKind::SemicolonStatement
        )
    }

    pub fn is_expression(self) -> bool {
        !self.is_statement()
    }

    pub fn has_block(self) -> bool {
        matches!(
            self,
            SyntaxKind::BlockExpression
                | SyntaxKind::IfExpression
                | SyntaxKind::PredicateLoopExpression
        )
    }
}

impl Display for SyntaxKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyntaxKind::MainFunctionStatement => write!(f, "main function statement"),
            SyntaxKind::ExpressionStatement => write!(f, "expression statement"),
            SyntaxKind::FunctionStatement => write!(f, "function statement"),
            SyntaxKind::LetStatement => write!(f, "let statement"),
            SyntaxKind::SemicolonStatement => write!(f, "semicolon statement"),
            SyntaxKind::BooleanExpression => write!(f, "boolean expression"),
            SyntaxKind::ByteExpression => write!(f, "byte expression"),
            SyntaxKind::CharacterExpression => write!(f, "character expression"),
            SyntaxKind::FloatExpression => write!(f, "float expression"),
            SyntaxKind::IntegerExpression => write!(f, "integer expression"),
            SyntaxKind::StringExpression => write!(f, "string expression"),
            SyntaxKind::AdditionExpression => write!(f, "addition expression"),
            SyntaxKind::SubtractionExpression => write!(f, "subtraction expression"),
            SyntaxKind::MultiplicationExpression => write!(f, "multiplication expression"),
            SyntaxKind::DivisionExpression => write!(f, "division expression"),
            SyntaxKind::ModuloExpression => write!(f, "modulo expression"),
            SyntaxKind::ArrayExpression => write!(f, "array expression"),
            SyntaxKind::ArrayIndexExpression => write!(f, "array index expression"),
            SyntaxKind::BlockExpression => write!(f, "block expression"),
            SyntaxKind::CallExpression => write!(f, "call expression"),
            SyntaxKind::FunctionExpression => write!(f, "function expression"),
            SyntaxKind::GroupedExpression => write!(f, "grouped expression"),
            SyntaxKind::IfExpression => write!(f, "if expression"),
            SyntaxKind::OperatorExpression => write!(f, "operator expression"),
            SyntaxKind::PathExpression => write!(f, "path expression"),
            SyntaxKind::PredicateLoopExpression => write!(f, "predicate loop expression"),
            SyntaxKind::ReturnExpression => write!(f, "return expression"),
        }
    }
}

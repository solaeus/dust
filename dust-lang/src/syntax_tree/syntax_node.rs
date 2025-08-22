use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::Span;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SyntaxNode {
    pub kind: SyntaxKind,
    pub payload: (u32, u32),
    pub position: Span,
}

impl SyntaxNode {
    pub fn encode_float(float: f64) -> (u32, u32) {
        let float_bytes = float.to_le_bytes();
        let left_payload = u32::from_le_bytes([
            float_bytes[0],
            float_bytes[1],
            float_bytes[2],
            float_bytes[3],
        ]);
        let right_payload = u32::from_le_bytes([
            float_bytes[4],
            float_bytes[5],
            float_bytes[6],
            float_bytes[7],
        ]);

        (left_payload, right_payload)
    }

    pub fn decode_float(payload: (u32, u32)) -> f64 {
        let left_bytes = payload.0.to_le_bytes();
        let right_bytes = payload.1.to_le_bytes();
        let float_bytes = [
            left_bytes[0],
            left_bytes[1],
            left_bytes[2],
            left_bytes[3],
            right_bytes[0],
            right_bytes[1],
            right_bytes[2],
            right_bytes[3],
        ];

        f64::from_le_bytes(float_bytes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyntaxKind {
    // Items
    MainFunctionItem,
    ModuleItem,

    // Statements
    ExpressionStatement,
    FunctionStatement,
    LetStatement,
    LetMutStatement,
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

    // Other Expressions
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

    // Sub-Syntax
    FunctionSignature,
    FunctionParameters,
    FunctionParameter,
    TypePath,

    // Types
    BooleanType,
    ByteType,
    CharacterType,
    FloatType,
    IntegerType,
    StringType,
}

impl SyntaxKind {
    pub fn is_item(&self) -> bool {
        matches!(self, SyntaxKind::MainFunctionItem | SyntaxKind::ModuleItem)
    }

    pub fn is_statement(&self) -> bool {
        matches!(
            self,
            SyntaxKind::MainFunctionItem
                | SyntaxKind::ExpressionStatement
                | SyntaxKind::FunctionStatement
                | SyntaxKind::LetStatement
                | SyntaxKind::SemicolonStatement
        )
    }

    pub fn is_expression(&self) -> bool {
        !self.is_item() && !self.is_statement()
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
            SyntaxKind::MainFunctionItem => write!(f, "item: main function"),
            SyntaxKind::ModuleItem => write!(f, "item: module"),
            SyntaxKind::ExpressionStatement => write!(f, "statement: expression"),
            SyntaxKind::FunctionStatement => write!(f, "statement: function"),
            SyntaxKind::LetStatement => write!(f, "statement: let"),
            SyntaxKind::LetMutStatement => write!(f, "statement: let mut"),
            SyntaxKind::SemicolonStatement => write!(f, "statement: semicolon"),
            SyntaxKind::BooleanExpression => write!(f, "expression: boolean"),
            SyntaxKind::ByteExpression => write!(f, "expression: byte"),
            SyntaxKind::CharacterExpression => write!(f, "expression: character"),
            SyntaxKind::FloatExpression => write!(f, "expression: float"),
            SyntaxKind::IntegerExpression => write!(f, "expression: integer"),
            SyntaxKind::StringExpression => write!(f, "expression: string"),
            SyntaxKind::AdditionExpression => write!(f, "expression: addition"),
            SyntaxKind::SubtractionExpression => write!(f, "expression: subtraction"),
            SyntaxKind::MultiplicationExpression => write!(f, "expression: multiplication"),
            SyntaxKind::DivisionExpression => write!(f, "expression: division"),
            SyntaxKind::ModuloExpression => write!(f, "expression: modulo"),
            SyntaxKind::ArrayExpression => write!(f, "expression: array"),
            SyntaxKind::ArrayIndexExpression => write!(f, "expression: array index"),
            SyntaxKind::BlockExpression => write!(f, "expression: block"),
            SyntaxKind::CallExpression => write!(f, "expression: call"),
            SyntaxKind::FunctionExpression => write!(f, "expression: function"),
            SyntaxKind::GroupedExpression => write!(f, "expression: grouped"),
            SyntaxKind::IfExpression => write!(f, "expression: if"),
            SyntaxKind::PathExpression => write!(f, "expression: path"),
            SyntaxKind::PredicateLoopExpression => write!(f, "expression: predicate loop"),
            SyntaxKind::ReturnExpression => write!(f, "expression: return"),
            _ => write!(f, "unknown syntax kind"),
        }
    }
}

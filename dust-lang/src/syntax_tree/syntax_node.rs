use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::Span;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntaxNode {
    pub kind: SyntaxKind,
    pub children: (u32, u32),
    pub span: Span,
    pub payload: u32,
}

impl SyntaxNode {
    pub fn encode_character(character: char) -> (u32, u32) {
        let char_bytes = (character as u32).to_le_bytes();
        let left_payload = u32::from_le_bytes([char_bytes[0], char_bytes[1], char_bytes[2], 0]);
        let right_payload = u32::from_le_bytes([char_bytes[3], 0, 0, 0]);

        (left_payload, right_payload)
    }

    pub fn decode_character(payload: (u32, u32)) -> char {
        let left_bytes = payload.0.to_le_bytes();
        let right_bytes = payload.1.to_le_bytes();
        let char_bytes = [left_bytes[0], left_bytes[1], left_bytes[2], right_bytes[0]];

        char::from_u32(u32::from_le_bytes(char_bytes)).unwrap_or_default()
    }

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

    pub fn encode_integer(integer: i64) -> (u32, u32) {
        let integer_bytes = integer.to_le_bytes();
        let left_payload = u32::from_le_bytes([
            integer_bytes[0],
            integer_bytes[1],
            integer_bytes[2],
            integer_bytes[3],
        ]);
        let right_payload = u32::from_le_bytes([
            integer_bytes[4],
            integer_bytes[5],
            integer_bytes[6],
            integer_bytes[7],
        ]);

        (left_payload, right_payload)
    }

    pub fn decode_integer(payload: (u32, u32)) -> i64 {
        let left_bytes = payload.0.to_le_bytes();
        let right_bytes = payload.1.to_le_bytes();
        let integer_bytes = [
            left_bytes[0],
            left_bytes[1],
            left_bytes[2],
            left_bytes[3],
            right_bytes[0],
            right_bytes[1],
            right_bytes[2],
            right_bytes[3],
        ];

        i64::from_le_bytes(integer_bytes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyntaxKind {
    // Items
    MainFunctionItem,
    ModuleItem,
    UseItem,

    // Statements
    ExpressionStatement,
    FunctionStatement,
    LetStatement,
    LetMutStatement,
    ReassignStatement,
    SemicolonStatement,

    // Literal Expressions
    BooleanExpression,
    ByteExpression,
    CharacterExpression,
    FloatExpression,
    IntegerExpression,
    StringExpression,

    // Binary Math Expressions
    AdditionExpression,
    SubtractionExpression,
    MultiplicationExpression,
    DivisionExpression,
    ModuloExpression,

    // Binary Math Assignment Expressions
    AdditionAssignmentExpression,
    SubtractionAssignmentExpression,
    MultiplicationAssignmentExpression,
    DivisionAssignmentExpression,
    ModuloAssignmentExpression,

    // Binary Logic Expressions
    AndExpression,
    OrExpression,

    // Binary Comparison Expressions
    GreaterThanExpression,
    LessThanExpression,
    GreaterThanOrEqualExpression,
    LessThanOrEqualExpression,
    EqualExpression,
    NotEqualExpression,

    // Unary Expressions
    NegationExpression,
    NotExpression,

    // Other Expressions
    ArrayExpression,
    ArrayIndexExpression,
    BlockExpression,
    CallExpression,
    FunctionExpression,
    NativeFunctionExpression,
    GroupedExpression,
    IfExpression,
    OperatorExpression,
    PathExpression,
    WhileExpression,
    ReturnExpression,
    BreakExpression,

    // Sub-Syntax
    FunctionSignature,
    FunctionValueParameters,
    FunctionValueParameter,
    FunctionValueParameterName,
    FunctionValueParameterType,
    FunctionTypeParameters,
    CallValueArguments,

    // Types (Sub-Syntax)
    BooleanType,
    ByteType,
    CharacterType,
    FloatType,
    IntegerType,
    StringType,
    TypePath,

    // Ignored
    Trivia,
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
                | SyntaxKind::LetMutStatement
                | SyntaxKind::ReassignStatement
                | SyntaxKind::SemicolonStatement
        )
    }

    pub fn is_expression(&self) -> bool {
        matches!(
            self,
            SyntaxKind::BooleanExpression
                | SyntaxKind::ByteExpression
                | SyntaxKind::CharacterExpression
                | SyntaxKind::FloatExpression
                | SyntaxKind::IntegerExpression
                | SyntaxKind::StringExpression
                | SyntaxKind::AdditionExpression
                | SyntaxKind::SubtractionExpression
                | SyntaxKind::MultiplicationExpression
                | SyntaxKind::DivisionExpression
                | SyntaxKind::ModuloExpression
                | SyntaxKind::AndExpression
                | SyntaxKind::OrExpression
                | SyntaxKind::GreaterThanExpression
                | SyntaxKind::LessThanExpression
                | SyntaxKind::GreaterThanOrEqualExpression
                | SyntaxKind::LessThanOrEqualExpression
                | SyntaxKind::EqualExpression
                | SyntaxKind::NotEqualExpression
                | SyntaxKind::NegationExpression
                | SyntaxKind::NotExpression
                | SyntaxKind::ArrayExpression
                | SyntaxKind::ArrayIndexExpression
                | SyntaxKind::BlockExpression
                | SyntaxKind::CallExpression
                | SyntaxKind::FunctionExpression
                | SyntaxKind::GroupedExpression
                | SyntaxKind::IfExpression
                | SyntaxKind::PathExpression
                | SyntaxKind::WhileExpression
                | SyntaxKind::ReturnExpression
                | SyntaxKind::BreakExpression
        )
    }

    pub fn has_block(self) -> bool {
        matches!(
            self,
            SyntaxKind::BlockExpression | SyntaxKind::IfExpression | SyntaxKind::WhileExpression
        )
    }
}

impl Display for SyntaxKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyntaxKind::MainFunctionItem => write!(f, "main function item"),
            SyntaxKind::ModuleItem => write!(f, "module item"),
            SyntaxKind::ExpressionStatement => write!(f, "expression statement"),
            SyntaxKind::FunctionStatement => write!(f, "function statement"),
            SyntaxKind::LetStatement => write!(f, "let statement"),
            SyntaxKind::LetMutStatement => write!(f, "let mut statement"),
            SyntaxKind::ReassignStatement => write!(f, "reassign statement"),
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
            SyntaxKind::AdditionAssignmentExpression => {
                write!(f, "addition assignment expression")
            }
            SyntaxKind::SubtractionAssignmentExpression => {
                write!(f, "subtraction assignment expression")
            }
            SyntaxKind::MultiplicationAssignmentExpression => {
                write!(f, "multiplication assignment expression")
            }
            SyntaxKind::DivisionAssignmentExpression => {
                write!(f, "division assignment expression")
            }
            SyntaxKind::ModuloAssignmentExpression => {
                write!(f, "modulo assignment expression")
            }
            SyntaxKind::AndExpression => write!(f, "and expression"),
            SyntaxKind::OrExpression => write!(f, "or expression"),
            SyntaxKind::GreaterThanExpression => write!(f, "greater than expression"),
            SyntaxKind::LessThanExpression => write!(f, "less than expression"),
            SyntaxKind::GreaterThanOrEqualExpression => {
                write!(f, "greater than or equal expression")
            }
            SyntaxKind::LessThanOrEqualExpression => write!(f, "less than or equal expression"),
            SyntaxKind::EqualExpression => write!(f, "equal expression"),
            SyntaxKind::NotEqualExpression => write!(f, "not equal expression"),
            SyntaxKind::NegationExpression => write!(f, "negation expression"),
            SyntaxKind::NotExpression => write!(f, "not expression"),
            SyntaxKind::ArrayExpression => write!(f, "array expression"),
            SyntaxKind::ArrayIndexExpression => write!(f, "array index expression"),
            SyntaxKind::BlockExpression => write!(f, "block expression"),
            SyntaxKind::CallExpression => write!(f, "call expression"),
            SyntaxKind::FunctionExpression => write!(f, "function expression"),
            SyntaxKind::NativeFunctionExpression => write!(f, "native function expression"),
            SyntaxKind::GroupedExpression => write!(f, "grouped expression"),
            SyntaxKind::IfExpression => write!(f, "if expression"),
            SyntaxKind::OperatorExpression => write!(f, "operator expression"),
            SyntaxKind::PathExpression => write!(f, "path expression"),
            SyntaxKind::WhileExpression => write!(f, "while loop expression"),
            SyntaxKind::ReturnExpression => write!(f, "return expression"),
            SyntaxKind::BreakExpression => write!(f, "break expression"),
            SyntaxKind::FunctionSignature => write!(f, "function signature"),
            SyntaxKind::FunctionValueParameters => write!(f, "function value parameters"),
            SyntaxKind::FunctionValueParameter => write!(f, "function value parameter"),
            SyntaxKind::FunctionValueParameterName => write!(f, "function value parameter name"),
            SyntaxKind::FunctionValueParameterType => write!(f, "function value parameter type"),
            SyntaxKind::FunctionTypeParameters => write!(f, "function type parameters"),
            SyntaxKind::CallValueArguments => write!(f, "call value arguments"),
            SyntaxKind::BooleanType => write!(f, "boolean type"),
            SyntaxKind::ByteType => write!(f, "byte type"),
            SyntaxKind::CharacterType => write!(f, "character type"),
            SyntaxKind::FloatType => write!(f, "float type"),
            SyntaxKind::IntegerType => write!(f, "integer type"),
            SyntaxKind::StringType => write!(f, "string type"),
            SyntaxKind::TypePath => write!(f, "type path"),
            SyntaxKind::UseItem => write!(f, "use item"),
            SyntaxKind::Trivia => write!(f, "whitespace or comment"),
        }
    }
}

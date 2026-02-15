use std::{
    fmt::{self, Display, Formatter},
    ops::Range,
};

use serde::{Deserialize, Serialize};

use crate::{source::Span, syntax::SyntaxId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntaxNode {
    pub kind: SyntaxKind,
    pub payload: SyntaxPayload,
    pub payload_kind: SyntaxPayloadKind,
    pub span: Span,
}

impl SyntaxNode {
    pub fn empty(kind: SyntaxKind, span: Span) -> Self {
        Self {
            kind,
            payload: SyntaxPayload::empty(),
            payload_kind: SyntaxPayloadKind::Empty,
            span,
        }
    }

    pub fn with_child(kind: SyntaxKind, span: Span, child_id: SyntaxId) -> Self {
        Self {
            kind,
            payload: SyntaxPayload::child(child_id),
            payload_kind: SyntaxPayloadKind::SingleChild,
            span,
        }
    }

    pub fn with_binary_children(
        kind: SyntaxKind,
        span: Span,
        left_child_id: SyntaxId,
        right_child_id: SyntaxId,
    ) -> Self {
        Self {
            kind,
            payload: SyntaxPayload::children(left_child_id, right_child_id),
            payload_kind: SyntaxPayloadKind::BinaryChildren,
            span,
        }
    }

    pub fn with_multiple_children(
        kind: SyntaxKind,
        span: Span,
        start_index: u32,
        child_count: u32,
    ) -> Self {
        Self {
            kind,
            payload: SyntaxPayload {
                left: start_index,
                right: child_count,
            },
            payload_kind: SyntaxPayloadKind::MultipleChildren,
            span,
        }
    }
}

impl Display for SyntaxNode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.kind)?;

        match self.kind {
            SyntaxKind::BooleanExpression => {
                let boolean = self.payload.decode_boolean();

                write!(f, ": {boolean}")?;
            }
            SyntaxKind::ByteExpression => {
                let byte = self.payload.decode_byte();

                write!(f, ": {byte}")?;
            }
            SyntaxKind::CharacterExpression => {
                let character = self.payload.decode_character();

                write!(f, ": {character}")?;
            }
            SyntaxKind::FloatExpression => {
                let float = self.payload.decode_float();

                write!(f, ": {float}")?;
            }
            SyntaxKind::IntegerExpression => {
                let integer = self.payload.decode_integer();

                write!(f, ": {integer}")?;
            }
            SyntaxKind::StringExpression => {
                let string = self.payload.decode_string();

                write!(f, ": \"{string}\" (length: {})", self.span.length() - 2)?;
            }
            _ => {}
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyntaxKind {
    // Items
    Root,
    ModuleItem,
    PublicModuleItem,
    UseItem,
    PublicUseItem,
    FunctionItem,
    PublicFunctionItem,
    StructItem,
    PublicStructItem,

    // Statements
    ExpressionStatement,
    LetStatement,
    LetMutStatement,
    ReassignmentStatement,
    SemicolonStatement,
    AdditionAssignmentStatement,
    SubtractionAssignmentStatement,
    MultiplicationAssignmentStatement,
    DivisionAssignmentStatement,
    ModuloAssignmentStatement,
    ExponentAssignmentStatement,

    // Paths
    Path,
    PathSegment,
    PathExpression,

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
    ExponentExpression,

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

    // List Expressions
    ListExpression,
    ListIndexExpression,

    // Function Expressions
    FunctionExpression,
    NativeFunctionExpression,
    CallExpression,

    // Control Flow Expressions
    IfExpression,
    ElseExpression,

    // Loop Expressions
    WhileExpression,

    ReturnExpression,
    BreakExpression,
    AsExpression,
    StructExpression,
    GroupedExpression,
    BlockExpression,

    // Sub-Syntax
    CallValueArguments,
    FunctionSignature,
    ValueParameters,

    // Types (Sub-Syntax)
    BooleanType,
    ByteType,
    CharacterType,
    FloatType,
    IntegerType,
    StringType,
    TypePath,
    ListType,
    FunctionType,
    ValueParameterTypes,

    // Ignored
    Trivia,
}

impl SyntaxKind {
    pub fn is_item(&self) -> bool {
        matches!(
            self,
            SyntaxKind::Root
                | SyntaxKind::ModuleItem
                | SyntaxKind::PublicModuleItem
                | SyntaxKind::UseItem
                | SyntaxKind::PublicUseItem
                | SyntaxKind::FunctionItem
                | SyntaxKind::PublicFunctionItem
                | SyntaxKind::StructItem
                | SyntaxKind::PublicStructItem
        )
    }

    pub fn is_statement(&self) -> bool {
        matches!(
            self,
            SyntaxKind::ExpressionStatement
                | SyntaxKind::LetStatement
                | SyntaxKind::LetMutStatement
                | SyntaxKind::ReassignmentStatement
                | SyntaxKind::SemicolonStatement
                | SyntaxKind::AdditionAssignmentStatement
                | SyntaxKind::SubtractionAssignmentStatement
                | SyntaxKind::MultiplicationAssignmentStatement
                | SyntaxKind::DivisionAssignmentStatement
                | SyntaxKind::ModuloAssignmentStatement
                | SyntaxKind::ExponentAssignmentStatement
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
                | SyntaxKind::ListExpression
                | SyntaxKind::ListIndexExpression
                | SyntaxKind::BlockExpression
                | SyntaxKind::CallExpression
                | SyntaxKind::FunctionExpression
                | SyntaxKind::GroupedExpression
                | SyntaxKind::PathExpression
                | SyntaxKind::WhileExpression
                | SyntaxKind::ReturnExpression
                | SyntaxKind::BreakExpression
                | SyntaxKind::AsExpression
                | SyntaxKind::IfExpression
                | SyntaxKind::ElseExpression
                | SyntaxKind::StructExpression
        )
    }

    pub fn has_block(self) -> bool {
        matches!(
            self,
            SyntaxKind::BlockExpression
                | SyntaxKind::IfExpression
                | SyntaxKind::ElseExpression
                | SyntaxKind::WhileExpression
        )
    }

    pub fn as_str(&self) -> &str {
        match self {
            SyntaxKind::Root => "root",
            SyntaxKind::ModuleItem => "module item",
            SyntaxKind::PublicModuleItem => "public module item",
            SyntaxKind::UseItem => "use item",
            SyntaxKind::PublicUseItem => "public use item",
            SyntaxKind::FunctionItem => "function item",
            SyntaxKind::PublicFunctionItem => "public function item",
            SyntaxKind::StructItem => "struct item",
            SyntaxKind::PublicStructItem => "public struct item",
            SyntaxKind::ExpressionStatement => "expression statement",
            SyntaxKind::LetStatement => "let statement",
            SyntaxKind::LetMutStatement => "let mut statement",
            SyntaxKind::ReassignmentStatement => "reassignment statement",
            SyntaxKind::SemicolonStatement => "semicolon statement",
            SyntaxKind::BooleanExpression => "boolean expression",
            SyntaxKind::ByteExpression => "byte expression",
            SyntaxKind::CharacterExpression => "character expression",
            SyntaxKind::FloatExpression => "float expression",
            SyntaxKind::IntegerExpression => "integer expression",
            SyntaxKind::StringExpression => "string expression",
            SyntaxKind::AdditionExpression => "addition expression",
            SyntaxKind::SubtractionExpression => "subtraction expression",
            SyntaxKind::MultiplicationExpression => "multiplication expression",
            SyntaxKind::DivisionExpression => "division expression",
            SyntaxKind::ModuloExpression => "modulo expression",
            SyntaxKind::ExponentExpression => "exponent expression",
            SyntaxKind::AdditionAssignmentStatement => "addition assignment statement",
            SyntaxKind::SubtractionAssignmentStatement => "subtraction assignment statement",
            SyntaxKind::MultiplicationAssignmentStatement => "multiplication assignment statement",
            SyntaxKind::DivisionAssignmentStatement => "division assignment statement",
            SyntaxKind::ModuloAssignmentStatement => "modulo assignment statement",
            SyntaxKind::ExponentAssignmentStatement => "exponent assignment statement",
            SyntaxKind::AndExpression => "and expression",
            SyntaxKind::OrExpression => "or expression",
            SyntaxKind::GreaterThanExpression => "greater than expression",
            SyntaxKind::LessThanExpression => "less than expression",
            SyntaxKind::GreaterThanOrEqualExpression => "greater than or equal expression",
            SyntaxKind::LessThanOrEqualExpression => "less than or equal expression",
            SyntaxKind::EqualExpression => "equal expression",
            SyntaxKind::NotEqualExpression => "not equal expression",
            SyntaxKind::NegationExpression => "negation expression",
            SyntaxKind::NotExpression => "not expression",
            SyntaxKind::ListIndexExpression => "index expression",
            SyntaxKind::ListExpression => "list expression",
            SyntaxKind::BlockExpression => "block expression",
            SyntaxKind::CallExpression => "call expression",
            SyntaxKind::FunctionExpression => "function expression",
            SyntaxKind::NativeFunctionExpression => "native function expression",
            SyntaxKind::GroupedExpression => "grouped expression",
            SyntaxKind::IfExpression => "if expression",
            SyntaxKind::ElseExpression => "else expression",
            SyntaxKind::StructExpression => "struct expression",
            SyntaxKind::PathExpression => "path expression",
            SyntaxKind::WhileExpression => "while loop expression",
            SyntaxKind::ReturnExpression => "return expression",
            SyntaxKind::BreakExpression => "break expression",
            SyntaxKind::AsExpression => "as expression",
            SyntaxKind::FunctionSignature => "function signature",
            SyntaxKind::ValueParameters => "value parameters",
            SyntaxKind::FunctionType => "function type",
            SyntaxKind::ValueParameterTypes => "value parameter types",
            SyntaxKind::CallValueArguments => "call value arguments",
            SyntaxKind::Path => "path",
            SyntaxKind::PathSegment => "path segment",
            SyntaxKind::BooleanType => "boolean type",
            SyntaxKind::ByteType => "byte type",
            SyntaxKind::CharacterType => "character type",
            SyntaxKind::FloatType => "float type",
            SyntaxKind::IntegerType => "integer type",
            SyntaxKind::StringType => "string type",
            SyntaxKind::TypePath => "type path",
            SyntaxKind::ListType => "list type",
            SyntaxKind::Trivia => "whitespace or comment",
        }
    }
}

impl Display for SyntaxKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntaxPayload {
    pub(super) left: u32,
    pub(super) right: u32,
}

impl SyntaxPayload {
    pub fn empty() -> Self {
        Self {
            left: SyntaxId::NONE.0,
            right: SyntaxId::NONE.0,
        }
    }

    pub fn child(child_id: SyntaxId) -> Self {
        Self {
            left: child_id.0,
            right: SyntaxId::NONE.0,
        }
    }

    pub fn children(left: SyntaxId, right: SyntaxId) -> Self {
        Self {
            left: left.0,
            right: right.0,
        }
    }

    pub fn child_indices(start_index: u32, count: u32) -> Self {
        Self {
            left: start_index,
            right: count,
        }
    }

    pub fn encode_boolean(boolean: bool) -> Self {
        SyntaxPayload {
            left: boolean as u32,
            right: SyntaxId::NONE.0,
        }
    }

    pub fn decode_boolean(&self) -> bool {
        self.left != 0
    }

    pub fn encode_byte(byte: u8) -> Self {
        SyntaxPayload {
            left: byte as u32,
            right: SyntaxId::NONE.0,
        }
    }

    pub fn decode_byte(&self) -> u8 {
        self.left as u8
    }

    pub fn encode_character(character: char) -> Self {
        let char_bytes = (character as u32).to_le_bytes();
        let encoded =
            u32::from_le_bytes([char_bytes[0], char_bytes[1], char_bytes[2], char_bytes[3]]);

        SyntaxPayload {
            left: encoded,
            right: SyntaxId::NONE.0,
        }
    }

    pub fn decode_character(&self) -> char {
        let left_bytes = self.left.to_le_bytes();

        char::from_u32(u32::from_le_bytes(left_bytes)).unwrap_or_default()
    }

    pub fn encode_float(float: f64) -> Self {
        let float_bytes = float.to_le_bytes();
        let first_four_bytes = u32::from_le_bytes([
            float_bytes[0],
            float_bytes[1],
            float_bytes[2],
            float_bytes[3],
        ]);
        let last_four_bytes = u32::from_le_bytes([
            float_bytes[4],
            float_bytes[5],
            float_bytes[6],
            float_bytes[7],
        ]);

        SyntaxPayload {
            left: first_four_bytes,
            right: last_four_bytes,
        }
    }

    pub fn decode_float(&self) -> f64 {
        let left_bytes = self.left.to_le_bytes();
        let right_bytes = self.right.to_le_bytes();
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

    pub fn encode_integer(integer: i64) -> Self {
        let integer_bytes = integer.to_le_bytes();
        let first_four_bytes = u32::from_le_bytes([
            integer_bytes[0],
            integer_bytes[1],
            integer_bytes[2],
            integer_bytes[3],
        ]);
        let last_four_bytes = u32::from_le_bytes([
            integer_bytes[4],
            integer_bytes[5],
            integer_bytes[6],
            integer_bytes[7],
        ]);

        SyntaxPayload {
            left: first_four_bytes,
            right: last_four_bytes,
        }
    }

    pub fn decode_integer(&self) -> i64 {
        let left_bytes = self.left.to_le_bytes();
        let right_bytes = self.right.to_le_bytes();
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

    pub fn encode_string(string_bytes: &[u8]) -> Self {
        let length = string_bytes.len().min(7);
        let mut encoded_bytes = [0u8; 8];

        encoded_bytes[0] = length as u8;
        encoded_bytes[1..length + 1].copy_from_slice(&string_bytes[..length]);

        let left = u32::from_le_bytes([
            encoded_bytes[0],
            encoded_bytes[1],
            encoded_bytes[2],
            encoded_bytes[3],
        ]);
        let right = u32::from_le_bytes([
            encoded_bytes[4],
            encoded_bytes[5],
            encoded_bytes[6],
            encoded_bytes[7],
        ]);

        SyntaxPayload { left, right }
    }

    pub fn decode_string(&self) -> String {
        let left_bytes = self.left.to_le_bytes();
        let right_bytes = self.right.to_le_bytes();

        let length = left_bytes[0] as usize;
        let encoded_bytes = [
            left_bytes[1],
            left_bytes[2],
            left_bytes[3],
            right_bytes[0],
            right_bytes[1],
            right_bytes[2],
            right_bytes[3],
        ];

        let string_bytes: Vec<u8> = encoded_bytes.into_iter().take(length).collect();

        String::from_utf8(string_bytes).unwrap_or_default()
    }

    pub fn left_id(&self) -> SyntaxId {
        SyntaxId(self.left)
    }

    pub fn right_id(&self) -> SyntaxId {
        SyntaxId(self.right)
    }

    pub fn as_usize_range(&self) -> Range<usize> {
        let start = self.left as usize;
        let end = start.saturating_add(self.right as usize);

        start..end
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyntaxPayloadKind {
    Empty,
    SingleChild,
    BinaryChildren,
    MultipleChildren,
    Value,
}

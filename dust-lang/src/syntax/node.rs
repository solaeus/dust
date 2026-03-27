use std::{
    fmt::{self, Display, Formatter},
    ops::Range,
};

use serde::{Deserialize, Serialize};

use crate::{source::Span, syntax::SyntaxId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntaxNode {
    pub(crate) kind: SyntaxKind,
    pub(crate) children: SyntaxPayload,
    pub(crate) children_kind: SyntaxPayloadKind,
    pub(crate) span: Span,
    pub(crate) attachments: SyntaxPayload,
    pub(crate) attachments_kind: SyntaxPayloadKind,
    pub(crate) modifier: bool,
}

impl SyntaxNode {
    pub(crate) fn with_modifier(mut self, modifier: bool) -> Self {
        self.modifier = modifier;

        self
    }
}

impl Display for SyntaxNode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum SyntaxKind {
    Root,

    // Items
    ModuleItem,
    UseItem,
    FunctionItem,
    StructItem,
    EnumItem,

    // Statements
    LetStatement,
    ExpressionStatement,

    // Assignment expressions
    AssignmentExpression,
    AdditionAssignmentExpression,
    SubtractionAssignmentExpression,
    MultiplicationAssignmentExpression,
    DivisionAssignmentExpression,
    ModuloAssignmentExpression,
    ExponentAssignmentExpression,

    // Literal expressions
    BooleanExpression,
    CharacterExpression,
    FloatExpression,
    HexadecimalIntegerExpression,
    IntegerExpression,
    StringExpression,

    // Binary math expressions
    AdditionExpression,
    SubtractionExpression,
    MultiplicationExpression,
    DivisionExpression,
    ModuloExpression,
    ExponentExpression,

    // Binary logic expressions
    AndExpression,
    OrExpression,

    // Binary comparison expressions
    GreaterThanExpression,
    LessThanExpression,
    GreaterThanOrEqualExpression,
    LessThanOrEqualExpression,
    EqualExpression,
    NotEqualExpression,

    // Unary expressions
    NegationExpression,
    NotExpression,

    // Array expressions
    ArrayExpression,
    ArrayRepeatExpression,
    IndexExpression,

    // Function expressions
    CallExpression,

    // Control flow expressions
    IfExpression,

    // Loop expressions
    WhileExpression,

    ReturnExpression,
    BreakExpression,
    AsExpression,
    StructExpression,
    GroupedExpression,
    BlockExpression,

    // Paths
    Path,
    PathSegment,
    SimplePath,
    PathExpression,

    // Sub-Syntax
    ModuleBody,
    ValueArguments,
    FunctionSignature,
    FunctionParameters,
    ValueParameters,
    TypeParameters,
    EnumVariants,
    EnumUnitVariant,
    EnumTupleVariant,
    EnumStructVariant,
    StructItemUnit,
    StructItemTupleFields,
    StructItemStructFields,
    StructExpressionStructFields,

    // Types
    TypePath,
    BooleanType,
    CharacterType,
    StringType,
    U8Type,
    I8Type,
    U16Type,
    I16Type,
    U32Type,
    I32Type,
    U64Type,
    I64Type,
    U128Type,
    I128Type,
    F32Type,
    F64Type,
    SliceType,
    FunctionType,
    NeverType,
    TupleType,
    ValueParameterTypes,

    // Ignored
    Trivia,
}

impl SyntaxKind {
    pub fn empty(self, span: Span) -> SyntaxNode {
        SyntaxNode {
            kind: self,
            children: SyntaxPayload::empty(),
            children_kind: SyntaxPayloadKind::Empty,
            span,
            attachments: SyntaxPayload::empty(),
            attachments_kind: SyntaxPayloadKind::Empty,
            modifier: false,
        }
    }

    pub fn with_child(self, span: Span, child_id: SyntaxId) -> SyntaxNode {
        SyntaxNode {
            kind: self,
            children: SyntaxPayload {
                left: child_id.0,
                right: SyntaxId::NONE.0,
            },
            children_kind: SyntaxPayloadKind::SingleChild,
            span,
            attachments: SyntaxPayload::empty(),
            attachments_kind: SyntaxPayloadKind::Empty,
            modifier: false,
        }
    }

    pub fn with_binary_children(
        self,
        span: Span,
        left_child_id: SyntaxId,
        right_child_id: SyntaxId,
    ) -> SyntaxNode {
        SyntaxNode {
            kind: self,
            children: SyntaxPayload::binary_children(left_child_id, right_child_id),
            children_kind: SyntaxPayloadKind::BinaryChildren,
            span,
            attachments: SyntaxPayload::empty(),
            attachments_kind: SyntaxPayloadKind::Empty,
            modifier: false,
        }
    }

    pub fn with_multiple_children(self, span: Span, payload: SyntaxPayload) -> SyntaxNode {
        SyntaxNode {
            kind: self,
            children: payload,
            children_kind: SyntaxPayloadKind::MultipleChildren,
            span,
            attachments: SyntaxPayload::empty(),
            attachments_kind: SyntaxPayloadKind::Empty,
            modifier: false,
        }
    }

    pub fn is_item(&self) -> bool {
        matches!(
            self,
            SyntaxKind::ModuleItem
                | SyntaxKind::UseItem
                | SyntaxKind::FunctionItem
                | SyntaxKind::StructItem
                | SyntaxKind::EnumItem
        )
    }

    pub fn is_statement(&self) -> bool {
        matches!(
            self,
            SyntaxKind::ModuleItem
                | SyntaxKind::UseItem
                | SyntaxKind::FunctionItem
                | SyntaxKind::StructItem
                | SyntaxKind::EnumItem
                | SyntaxKind::LetStatement
                | SyntaxKind::ExpressionStatement
        )
    }

    pub fn is_expression(&self) -> bool {
        matches!(
            self,
            SyntaxKind::AdditionExpression
                | SyntaxKind::AndExpression
                | SyntaxKind::ArrayExpression
                | SyntaxKind::ArrayRepeatExpression
                | SyntaxKind::AsExpression
                | SyntaxKind::BlockExpression
                | SyntaxKind::BreakExpression
                | SyntaxKind::CallExpression
                | SyntaxKind::CharacterExpression
                | SyntaxKind::DivisionExpression
                | SyntaxKind::EqualExpression
                | SyntaxKind::FloatExpression
                | SyntaxKind::GreaterThanExpression
                | SyntaxKind::GreaterThanOrEqualExpression
                | SyntaxKind::GroupedExpression
                | SyntaxKind::HexadecimalIntegerExpression
                | SyntaxKind::IfExpression
                | SyntaxKind::IndexExpression
                | SyntaxKind::IntegerExpression
                | SyntaxKind::LessThanExpression
                | SyntaxKind::LessThanOrEqualExpression
                | SyntaxKind::ModuloExpression
                | SyntaxKind::MultiplicationExpression
                | SyntaxKind::NegationExpression
                | SyntaxKind::NotEqualExpression
                | SyntaxKind::NotExpression
                | SyntaxKind::OrExpression
                | SyntaxKind::PathExpression
                | SyntaxKind::ReturnExpression
                | SyntaxKind::StringExpression
                | SyntaxKind::StructExpression
                | SyntaxKind::SubtractionExpression
                | SyntaxKind::WhileExpression
                | SyntaxKind::BooleanExpression
        )
    }

    pub fn has_block(self) -> bool {
        matches!(
            self,
            SyntaxKind::BlockExpression | SyntaxKind::IfExpression | SyntaxKind::WhileExpression
        )
    }

    pub fn encodes_value_hint(self) -> bool {
        matches!(
            self,
            SyntaxKind::BooleanExpression
                | SyntaxKind::HexadecimalIntegerExpression
                | SyntaxKind::CharacterExpression
                | SyntaxKind::FloatExpression
                | SyntaxKind::IntegerExpression
                | SyntaxKind::StringExpression
        )
    }

    pub fn as_str(&self) -> &str {
        match self {
            SyntaxKind::AdditionAssignmentExpression => "addition assignment expression",
            SyntaxKind::AdditionExpression => "addition expression",
            SyntaxKind::AndExpression => "and expression",
            SyntaxKind::AsExpression => "as expression",
            SyntaxKind::AssignmentExpression => "reassignment expression",
            SyntaxKind::ArrayExpression => "array expression",
            SyntaxKind::ArrayRepeatExpression => "array repeat expression",
            SyntaxKind::BlockExpression => "block expression",
            SyntaxKind::BooleanExpression => "boolean expression",
            SyntaxKind::BooleanType => "boolean type",
            SyntaxKind::BreakExpression => "break expression",
            SyntaxKind::HexadecimalIntegerExpression => "hexadecimal integer expression",
            SyntaxKind::CallExpression => "call expression",
            SyntaxKind::CharacterExpression => "character expression",
            SyntaxKind::CharacterType => "character type",
            SyntaxKind::DivisionAssignmentExpression => "division assignment expression",
            SyntaxKind::DivisionExpression => "division expression",
            SyntaxKind::EnumItem => "enum item",
            SyntaxKind::EnumVariants => "enum variants",
            SyntaxKind::EnumUnitVariant => "enum unit variant",
            SyntaxKind::EnumTupleVariant => "enum tuple variant",
            SyntaxKind::EnumStructVariant => "enum struct variant",
            SyntaxKind::EqualExpression => "equal expression",
            SyntaxKind::ExponentAssignmentExpression => "exponent assignment expression",
            SyntaxKind::ExponentExpression => "exponent expression",
            SyntaxKind::ExpressionStatement => "expression statement",
            SyntaxKind::F32Type => "f32 type",
            SyntaxKind::F64Type => "f64 type",
            SyntaxKind::FloatExpression => "float expression",
            SyntaxKind::FunctionItem => "function item",
            SyntaxKind::FunctionParameters => "function parameters",
            SyntaxKind::FunctionSignature => "function signature",
            SyntaxKind::FunctionType => "function type",
            SyntaxKind::GreaterThanExpression => "greater than expression",
            SyntaxKind::GreaterThanOrEqualExpression => "greater than or equal expression",
            SyntaxKind::GroupedExpression => "grouped expression",
            SyntaxKind::I128Type => "i128 type",
            SyntaxKind::I16Type => "i16 type",
            SyntaxKind::I32Type => "i32 type",
            SyntaxKind::I64Type => "i64 type",
            SyntaxKind::I8Type => "i8 type",
            SyntaxKind::IfExpression => "if expression",
            SyntaxKind::IndexExpression => "index expression",
            SyntaxKind::IntegerExpression => "integer expression",
            SyntaxKind::LessThanExpression => "less than expression",
            SyntaxKind::LessThanOrEqualExpression => "less than or equal expression",
            SyntaxKind::LetStatement => "let statement",
            SyntaxKind::ModuleBody => "module body",
            SyntaxKind::ModuleItem => "module item",
            SyntaxKind::ModuloAssignmentExpression => "modulo assignment expression",
            SyntaxKind::ModuloExpression => "modulo expression",
            SyntaxKind::MultiplicationAssignmentExpression => {
                "multiplication assignment expression"
            }
            SyntaxKind::MultiplicationExpression => "multiplication expression",
            SyntaxKind::NegationExpression => "negation expression",
            SyntaxKind::NeverType => "never type",
            SyntaxKind::NotEqualExpression => "not equal expression",
            SyntaxKind::NotExpression => "not expression",
            SyntaxKind::OrExpression => "or expression",
            SyntaxKind::Path => "path",
            SyntaxKind::PathExpression => "path expression",
            SyntaxKind::PathSegment => "path segment",
            SyntaxKind::ReturnExpression => "return expression",
            SyntaxKind::Root => "root",
            SyntaxKind::SimplePath => "simple path",
            SyntaxKind::SliceType => "slice type",
            SyntaxKind::StringExpression => "string expression",
            SyntaxKind::StringType => "string type",
            SyntaxKind::StructItemStructFields => "struct item struct fields",
            SyntaxKind::StructExpression => "struct expression",
            SyntaxKind::StructExpressionStructFields => "struct expression struct fields",
            SyntaxKind::StructItem => "struct item",
            SyntaxKind::StructItemUnit => "struct item unit",
            SyntaxKind::SubtractionAssignmentExpression => "subtraction assignment expression",
            SyntaxKind::SubtractionExpression => "subtraction expression",
            SyntaxKind::Trivia => "trivia",
            SyntaxKind::StructItemTupleFields => "struct item tuple fields",
            SyntaxKind::TypeParameters => "type parameters",
            SyntaxKind::TypePath => "type path",
            SyntaxKind::TupleType => "tuple type",
            SyntaxKind::U128Type => "u128 type",
            SyntaxKind::U16Type => "u16 type",
            SyntaxKind::U32Type => "u32 type",
            SyntaxKind::U64Type => "u64 type",
            SyntaxKind::U8Type => "u8 type",
            SyntaxKind::UseItem => "use item",
            SyntaxKind::ValueArguments => "value arguments",
            SyntaxKind::ValueParameters => "value parameters",
            SyntaxKind::ValueParameterTypes => "value parameter types",
            SyntaxKind::WhileExpression => "while loop expression",
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

    pub fn binary_children(left: SyntaxId, right: SyntaxId) -> Self {
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
}

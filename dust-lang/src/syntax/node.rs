use std::fmt::{self, Display, Formatter};

use crate::{source::Span, syntax::SyntaxId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SyntaxNode {
    pub kind: SyntaxKind,
    pub payload: SyntaxPayload,
    pub span: Span,
}

impl SyntaxNode {
    pub fn children(&self) -> SyntaxNodeChildren {
        match self.kind {
            SyntaxKind::MainFunctionItem
            | SyntaxKind::ModuleItem
            | SyntaxKind::LetStatement
            | SyntaxKind::LetMutStatement
            | SyntaxKind::BlockExpression
            | SyntaxKind::ListExpression
            | SyntaxKind::Path
            | SyntaxKind::CallValueArguments
            | SyntaxKind::ValueParametersDefinition
            | SyntaxKind::IfExpression
            | SyntaxKind::StructFieldsDefinition
            | SyntaxKind::StructFields => SyntaxNodeChildren::Multiple(self.payload),
            SyntaxKind::ExpressionStatement
            | SyntaxKind::PathExpression
            | SyntaxKind::GroupedExpression
            | SyntaxKind::NegationExpression
            | SyntaxKind::NotExpression
            | SyntaxKind::ElseExpression => SyntaxNodeChildren::Single(SyntaxId(self.payload.left)),
            SyntaxKind::FunctionItem
            | SyntaxKind::PublicFunctionItem
            | SyntaxKind::ReassignmentStatement
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ValueParameterDefinition
            | SyntaxKind::FunctionSignature
            | SyntaxKind::AdditionExpression
            | SyntaxKind::SubtractionExpression
            | SyntaxKind::MultiplicationExpression
            | SyntaxKind::DivisionExpression
            | SyntaxKind::ModuloExpression
            | SyntaxKind::ExponentExpression
            | SyntaxKind::AdditionAssignmentStatement
            | SyntaxKind::SubtractionAssignmentStatement
            | SyntaxKind::MultiplicationAssignmentStatement
            | SyntaxKind::DivisionAssignmentStatement
            | SyntaxKind::ModuloAssignmentStatement
            | SyntaxKind::AndExpression
            | SyntaxKind::OrExpression
            | SyntaxKind::EqualExpression
            | SyntaxKind::NotEqualExpression
            | SyntaxKind::LessThanExpression
            | SyntaxKind::LessThanOrEqualExpression
            | SyntaxKind::GreaterThanExpression
            | SyntaxKind::GreaterThanOrEqualExpression
            | SyntaxKind::WhileExpression
            | SyntaxKind::CallExpression
            | SyntaxKind::ListIndexExpression
            | SyntaxKind::AsExpression
            | SyntaxKind::FunctionType
            | SyntaxKind::StructItem
            | SyntaxKind::PublicStructItem
            | SyntaxKind::StructFieldDefinition
            | SyntaxKind::StructExpression
            | SyntaxKind::StructField => SyntaxNodeChildren::Binary(
                SyntaxId(self.payload.left),
                SyntaxId(self.payload.right),
            ),
            _ => SyntaxNodeChildren::None,
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

                write!(f, ": {string}...")?;
            }
            _ => {}
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxKind {
    // Items
    MainFunctionItem,
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
    ValueParametersDefinition,
    ValueParameterDefinition,
    ValueParameterName,
    ValueParameterType,
    ValueParameterTypes,
    StructFieldsDefinition,
    StructFieldDefinition,
    StructFields,
    StructField,

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

    // Ignored
    Trivia,
}

impl SyntaxKind {
    pub fn is_item(&self) -> bool {
        matches!(
            self,
            SyntaxKind::MainFunctionItem
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
}

impl Display for SyntaxKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyntaxKind::MainFunctionItem => write!(f, "main function item"),
            SyntaxKind::ModuleItem => write!(f, "module item"),
            SyntaxKind::PublicModuleItem => write!(f, "public module item"),
            SyntaxKind::FunctionItem => write!(f, "function item"),
            SyntaxKind::PublicFunctionItem => write!(f, "public function item"),
            SyntaxKind::StructItem => write!(f, "struct item"),
            SyntaxKind::PublicStructItem => write!(f, "public struct item"),
            SyntaxKind::ExpressionStatement => write!(f, "expression statement"),
            SyntaxKind::LetStatement => write!(f, "let statement"),
            SyntaxKind::LetMutStatement => write!(f, "let mut statement"),
            SyntaxKind::ReassignmentStatement => write!(f, "reassignment statement"),
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
            SyntaxKind::ExponentExpression => write!(f, "exponent expression"),
            SyntaxKind::AdditionAssignmentStatement => {
                write!(f, "addition assignment statement")
            }
            SyntaxKind::SubtractionAssignmentStatement => {
                write!(f, "subtraction assignment statement")
            }
            SyntaxKind::MultiplicationAssignmentStatement => {
                write!(f, "multiplication assignment statement")
            }
            SyntaxKind::DivisionAssignmentStatement => {
                write!(f, "division assignment statement")
            }
            SyntaxKind::ModuloAssignmentStatement => {
                write!(f, "modulo assignment statement")
            }
            SyntaxKind::ExponentAssignmentStatement => {
                write!(f, "exponent assignment statement")
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
            SyntaxKind::ListIndexExpression => write!(f, "index expression"),
            SyntaxKind::ListExpression => write!(f, "list expression"),
            SyntaxKind::BlockExpression => write!(f, "block expression"),
            SyntaxKind::CallExpression => write!(f, "call expression"),
            SyntaxKind::FunctionExpression => write!(f, "function expression"),
            SyntaxKind::NativeFunctionExpression => write!(f, "native function expression"),
            SyntaxKind::GroupedExpression => write!(f, "grouped expression"),
            SyntaxKind::IfExpression => write!(f, "if expression"),
            SyntaxKind::ElseExpression => write!(f, "else expression"),
            SyntaxKind::StructExpression => write!(f, "struct expression"),
            SyntaxKind::PathExpression => write!(f, "path expression"),
            SyntaxKind::WhileExpression => write!(f, "while loop expression"),
            SyntaxKind::ReturnExpression => write!(f, "return expression"),
            SyntaxKind::BreakExpression => write!(f, "break expression"),
            SyntaxKind::AsExpression => write!(f, "as expression"),
            SyntaxKind::FunctionSignature => write!(f, "function signature"),
            SyntaxKind::ValueParametersDefinition => {
                write!(f, "value parameters definition")
            }
            SyntaxKind::ValueParameterDefinition => {
                write!(f, "value parameter definition")
            }
            SyntaxKind::ValueParameterName => write!(f, "value parameter name"),
            SyntaxKind::ValueParameterType => write!(f, "value parameter type"),
            SyntaxKind::ValueParameterTypes => write!(f, "value parameter types"),
            SyntaxKind::StructFieldsDefinition => write!(f, "struct fields definition"),
            SyntaxKind::StructFieldDefinition => write!(f, "struct field definition"),
            SyntaxKind::StructFields => write!(f, "struct fields"),
            SyntaxKind::StructField => write!(f, "struct field"),
            SyntaxKind::FunctionType => write!(f, "function type"),
            SyntaxKind::CallValueArguments => write!(f, "call value arguments"),
            SyntaxKind::Path => write!(f, "path"),
            SyntaxKind::PathSegment => write!(f, "path segment"),
            SyntaxKind::BooleanType => write!(f, "boolean type"),
            SyntaxKind::ByteType => write!(f, "byte type"),
            SyntaxKind::CharacterType => write!(f, "character type"),
            SyntaxKind::FloatType => write!(f, "float type"),
            SyntaxKind::IntegerType => write!(f, "integer type"),
            SyntaxKind::StringType => write!(f, "string type"),
            SyntaxKind::TypePath => write!(f, "type path"),
            SyntaxKind::ListType => write!(f, "list type"),
            SyntaxKind::UseItem => write!(f, "use item"),
            SyntaxKind::PublicUseItem => write!(f, "public use item"),
            SyntaxKind::Trivia => write!(f, "whitespace or comment"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SyntaxPayload {
    pub left: u32,
    pub right: u32,
}

impl SyntaxPayload {
    pub fn child(value: u32) -> Self {
        Self {
            left: value,
            right: SyntaxId::NONE.0,
        }
    }

    pub fn children(left: u32, right: u32) -> Self {
        Self { left, right }
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

    pub fn encode_string(str: &str) -> Self {
        let length = str.len().min(7);
        let mut encoded_bytes = [0u8; 8];
        let string_bytes = &str.as_bytes()[..length];

        encoded_bytes[0] = length as u8;

        encoded_bytes[1..].copy_from_slice(string_bytes);

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
        let mut string_bytes = vec![0; length];

        println!("length {length}");

        for encoded_byte in encoded_bytes.into_iter().take(length) {
            string_bytes.push(encoded_byte);
        }

        unsafe { String::from_utf8_unchecked(string_bytes) }
    }

    pub fn left_id(&self) -> SyntaxId {
        SyntaxId(self.left)
    }

    pub fn right_id(&self) -> SyntaxId {
        SyntaxId(self.right)
    }
}

impl Default for SyntaxPayload {
    fn default() -> Self {
        SyntaxPayload {
            left: SyntaxId::NONE.0,
            right: SyntaxId::NONE.0,
        }
    }
}

pub enum SyntaxNodeChildren {
    None,
    Single(SyntaxId),
    Binary(SyntaxId, SyntaxId),
    Multiple(SyntaxPayload),
}

use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{source::Span, syntax::SyntaxId};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyntaxNode {
    pub(crate) kind: SyntaxKind,
    pub(crate) children: SyntaxChildren,
    pub(crate) children_kind: SyntaxChildrenKind,
    pub(crate) flags: SyntaxFlags,
    pub(crate) span: Span,
}

impl SyntaxNode {
    #[cfg(test)]
    pub fn with_flags(mut self, flags: SyntaxFlags) -> Self {
        self.flags.set_flag(flags);

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
    ModItem,
    UseItem,
    FunctionItem,
    ConstItem,
    TypeItem,
    StructItem,
    EnumItem,
    ImplItem,
    TraitItem,

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
    HexadecimalExpression,
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

    // Range expressions
    RangeExpression,
    RangeInclusiveExpression,

    // Other expressions
    GroupedExpression,
    BlockExpression,
    CallExpression,
    MethodCallExpression,
    FieldAccessExpression,
    IfExpression,
    WhileExpression,
    ReturnExpression,
    BreakExpression,
    AsExpression,
    StructExpression,

    // Paths
    Path,
    PathSegment,
    SimplePath,
    PathExpression,

    // Sub-Syntax
    ModuleBody,
    ImplBody,
    TraitBody,
    ValueArguments,

    ValueParameters,
    TypeParameters,
    TypeArguments,
    TraitBounds,
    EnumVariants,
    TypeParameter,
    WhereClause,
    WherePredicate,

    EnumUnitVariant,
    EnumTupleFieldsVariant,
    EnumNamedFieldsVariant,

    TupleFields,
    NamedFields,

    StructExpressionNamedFields,

    // Types
    TypePath,
    BooleanType,
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
    USizeType,
    ISizeType,
    F32Type,
    F64Type,
    CharacterType,
    StringType,
    ArrayType,
    FunctionType,
    FunctionTypeValueParameterTypes,
    NeverType,
    TupleType,
    SelfType,

    // Ignored
    Trivia,
}

impl SyntaxKind {
    pub fn empty(self, span: Span) -> SyntaxNode {
        SyntaxNode {
            kind: self,
            children: SyntaxChildren::empty(),
            children_kind: SyntaxChildrenKind::None,
            flags: SyntaxFlags::default(),
            span,
        }
    }

    pub fn empty_with_flags(self, span: Span, flags: SyntaxFlags) -> SyntaxNode {
        SyntaxNode {
            kind: self,
            children: SyntaxChildren::empty(),
            children_kind: SyntaxChildrenKind::None,
            flags,
            span,
        }
    }

    pub fn with_single_child(self, span: Span, child_id: SyntaxId) -> SyntaxNode {
        SyntaxNode {
            kind: self,
            children: SyntaxChildren::new(child_id.0, 0),
            children_kind: SyntaxChildrenKind::Single,
            flags: SyntaxFlags::default(),
            span,
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
            children: SyntaxChildren::new(left_child_id.0, right_child_id.0),
            children_kind: SyntaxChildrenKind::Binary,
            flags: SyntaxFlags::default(),
            span,
        }
    }

    pub fn with_children(self, span: Span, children: SyntaxChildren) -> SyntaxNode {
        SyntaxNode {
            kind: self,
            children,
            children_kind: SyntaxChildrenKind::Many,
            flags: SyntaxFlags::default(),
            span,
        }
    }

    pub fn is_item(&self) -> bool {
        matches!(
            self,
            SyntaxKind::ModItem
                | SyntaxKind::UseItem
                | SyntaxKind::FunctionItem
                | SyntaxKind::StructItem
                | SyntaxKind::EnumItem
                | SyntaxKind::ConstItem
                | SyntaxKind::TypeItem
                | SyntaxKind::ImplItem
                | SyntaxKind::TraitItem
        )
    }

    pub fn is_statement(&self) -> bool {
        matches!(
            self,
            SyntaxKind::ModItem
                | SyntaxKind::UseItem
                | SyntaxKind::FunctionItem
                | SyntaxKind::StructItem
                | SyntaxKind::EnumItem
                | SyntaxKind::ConstItem
                | SyntaxKind::TypeItem
                | SyntaxKind::ImplItem
                | SyntaxKind::TraitItem
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
                | SyntaxKind::BooleanExpression
                | SyntaxKind::BreakExpression
                | SyntaxKind::CallExpression
                | SyntaxKind::CharacterExpression
                | SyntaxKind::DivisionExpression
                | SyntaxKind::EqualExpression
                | SyntaxKind::ExponentExpression
                | SyntaxKind::FieldAccessExpression
                | SyntaxKind::FloatExpression
                | SyntaxKind::GreaterThanExpression
                | SyntaxKind::GreaterThanOrEqualExpression
                | SyntaxKind::GroupedExpression
                | SyntaxKind::HexadecimalExpression
                | SyntaxKind::IfExpression
                | SyntaxKind::IndexExpression
                | SyntaxKind::IntegerExpression
                | SyntaxKind::LessThanExpression
                | SyntaxKind::LessThanOrEqualExpression
                | SyntaxKind::MethodCallExpression
                | SyntaxKind::ModuloExpression
                | SyntaxKind::MultiplicationExpression
                | SyntaxKind::NegationExpression
                | SyntaxKind::NotEqualExpression
                | SyntaxKind::NotExpression
                | SyntaxKind::OrExpression
                | SyntaxKind::PathExpression
                | SyntaxKind::RangeExpression
                | SyntaxKind::RangeInclusiveExpression
                | SyntaxKind::ReturnExpression
                | SyntaxKind::StringExpression
                | SyntaxKind::StructExpression
                | SyntaxKind::SubtractionExpression
                | SyntaxKind::WhileExpression
        )
    }

    pub fn has_block(self) -> bool {
        matches!(
            self,
            SyntaxKind::BlockExpression | SyntaxKind::IfExpression | SyntaxKind::WhileExpression
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
            SyntaxKind::ArrayType => "array type",
            SyntaxKind::ArrayRepeatExpression => "array repeat expression",
            SyntaxKind::BlockExpression => "block expression",
            SyntaxKind::BooleanExpression => "boolean expression",
            SyntaxKind::BooleanType => "boolean type",
            SyntaxKind::BreakExpression => "break expression",
            SyntaxKind::HexadecimalExpression => "hexadecimal expression",
            SyntaxKind::CallExpression => "call expression",
            SyntaxKind::CharacterExpression => "character expression",
            SyntaxKind::CharacterType => "character type",
            SyntaxKind::ConstItem => "const item",
            SyntaxKind::DivisionAssignmentExpression => "division assignment expression",
            SyntaxKind::DivisionExpression => "division expression",
            SyntaxKind::EnumItem => "enum item",
            SyntaxKind::EnumVariants => "enum variants",
            SyntaxKind::EnumUnitVariant => "enum unit variant",
            SyntaxKind::EnumTupleFieldsVariant => "enum tuple fields variant",
            SyntaxKind::EnumNamedFieldsVariant => "enum named fields variant",
            SyntaxKind::EqualExpression => "equal expression",
            SyntaxKind::ExponentAssignmentExpression => "exponent assignment expression",
            SyntaxKind::ExponentExpression => "exponent expression",
            SyntaxKind::ExpressionStatement => "expression statement",
            SyntaxKind::F32Type => "f32 type",
            SyntaxKind::F64Type => "f64 type",
            SyntaxKind::FieldAccessExpression => "field access expression",
            SyntaxKind::FloatExpression => "float expression",
            SyntaxKind::FunctionItem => "function item",
            SyntaxKind::FunctionType => "function type",
            SyntaxKind::GreaterThanExpression => "greater than expression",
            SyntaxKind::GreaterThanOrEqualExpression => "greater than or equal expression",
            SyntaxKind::GroupedExpression => "grouped expression",
            SyntaxKind::I128Type => "i128 type",
            SyntaxKind::I16Type => "i16 type",
            SyntaxKind::I32Type => "i32 type",
            SyntaxKind::I64Type => "i64 type",
            SyntaxKind::I8Type => "i8 type",
            SyntaxKind::ISizeType => "isize type",
            SyntaxKind::IfExpression => "if expression",
            SyntaxKind::ImplBody => "impl body",
            SyntaxKind::ImplItem => "impl item",
            SyntaxKind::IndexExpression => "index expression",
            SyntaxKind::IntegerExpression => "integer expression",
            SyntaxKind::LessThanExpression => "less than expression",
            SyntaxKind::LessThanOrEqualExpression => "less than or equal expression",
            SyntaxKind::LetStatement => "let statement",
            SyntaxKind::ModuleBody => "module body",
            SyntaxKind::ModItem => "module item",
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
            SyntaxKind::RangeExpression => "range expression",
            SyntaxKind::RangeInclusiveExpression => "range inclusive expression",
            SyntaxKind::ReturnExpression => "return expression",
            SyntaxKind::Root => "root",
            SyntaxKind::SelfType => "self type",
            SyntaxKind::SimplePath => "simple path",
            SyntaxKind::StringExpression => "string expression",
            SyntaxKind::StringType => "string type",
            SyntaxKind::MethodCallExpression => "method call expression",
            SyntaxKind::NamedFields => "named fields",
            SyntaxKind::StructExpression => "struct expression",
            SyntaxKind::StructExpressionNamedFields => "struct expression named fields",
            SyntaxKind::StructItem => "struct item",
            SyntaxKind::SubtractionAssignmentExpression => "subtraction assignment expression",
            SyntaxKind::SubtractionExpression => "subtraction expression",
            SyntaxKind::Trivia => "trivia",
            SyntaxKind::TraitBody => "trait body",
            SyntaxKind::TraitBounds => "trait bounds",
            SyntaxKind::TraitItem => "trait item",
            SyntaxKind::TupleFields => "struct item tuple fields",
            SyntaxKind::TypeItem => "type item",
            SyntaxKind::TypeArguments => "type arguments",
            SyntaxKind::TypeParameters => "type parameters",
            SyntaxKind::TypeParameter => "type parameter",
            SyntaxKind::TypePath => "type path",
            SyntaxKind::TupleType => "tuple type",
            SyntaxKind::U128Type => "u128 type",
            SyntaxKind::U16Type => "u16 type",
            SyntaxKind::U32Type => "u32 type",
            SyntaxKind::U64Type => "u64 type",
            SyntaxKind::U8Type => "u8 type",
            SyntaxKind::USizeType => "usize type",
            SyntaxKind::UseItem => "use item",
            SyntaxKind::ValueArguments => "value arguments",
            SyntaxKind::ValueParameters => "value parameters",
            SyntaxKind::FunctionTypeValueParameterTypes => "function type value parameter types",
            SyntaxKind::WhileExpression => "while loop expression",
            SyntaxKind::WhereClause => "where clause",
            SyntaxKind::WherePredicate => "where predicate",
        }
    }
}

impl Display for SyntaxKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntaxChildren {
    pub(super) left: u32,
    pub(super) right: u32,
}

impl SyntaxChildren {
    pub fn new(left: u32, right: u32) -> Self {
        Self { left, right }
    }

    pub fn from_id(id: SyntaxId) -> Self {
        Self {
            left: id.0,
            right: 0,
        }
    }

    pub fn from_ids(left: SyntaxId, right: SyntaxId) -> Self {
        Self {
            left: left.0,
            right: right.0,
        }
    }

    pub fn empty() -> Self {
        Self { left: 0, right: 0 }
    }

    pub fn left_id(&self) -> SyntaxId {
        SyntaxId(self.left)
    }

    pub fn right_id(&self) -> SyntaxId {
        SyntaxId(self.right)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyntaxChildrenKind {
    None,
    Single,
    Binary,
    Many,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntaxFlags(u8);

impl SyntaxFlags {
    // Info flags
    pub const PUBLIC: Self = Self(1);
    pub const MUTABLE: Self = Self(1);
    pub const SELF_VALUE: Self = Self(1);
    pub const TRUE: Self = Self(1);

    // Children flags
    pub const TYPE_PARAMETERS: Self = Self(2);
    pub const TYPE_ARGUMENTS: Self = Self(2);
    pub const VALUE_PARAMETERS: Self = Self(4);
    pub const VALUE_ARGUMENTS: Self = Self(4);
    pub const RETURN_TYPE: Self = Self(8);
    pub const SUPERTRAITS: Self = Self(8);
    pub const TYPE_NAME: Self = Self(8);
    pub const FIELDS: Self = Self(8);
    pub const WHERE_CLAUSE: Self = Self(16);

    const _RESERVED: [Self; 3] = [Self(32), Self(64), Self(128)];

    pub fn new(flags: u8) -> Self {
        Self(flags)
    }

    pub fn and(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub fn set_flag(&mut self, flag: SyntaxFlags) {
        self.0 |= flag.0;
    }

    pub fn get_flag(&self, flag: SyntaxFlags) -> bool {
        (self.0 & flag.0) != 0
    }

    pub fn info_display(&self, kind: SyntaxKind) -> Option<&'static str> {
        match kind {
            SyntaxKind::ModItem
            | SyntaxKind::UseItem
            | SyntaxKind::FunctionItem
            | SyntaxKind::StructItem
            | SyntaxKind::EnumItem
            | SyntaxKind::ConstItem
            | SyntaxKind::TypeItem
                if self.get_flag(SyntaxFlags::PUBLIC) =>
            {
                Some("public")
            }
            SyntaxKind::LetStatement if self.get_flag(SyntaxFlags::MUTABLE) => Some("mutable"),
            SyntaxKind::ValueParameters if self.get_flag(SyntaxFlags::SELF_VALUE) => {
                Some("with self")
            }
            SyntaxKind::BooleanExpression => {
                if self.get_flag(SyntaxFlags::TRUE) {
                    Some("true")
                } else {
                    Some("false")
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::array;

    use super::*;

    #[test]
    fn flags() {
        let mut flag_values: [SyntaxFlags; 8] =
            array::from_fn(|index| SyntaxFlags((1 << index) as u8));

        for _ in 0..2 {
            let mut flags = SyntaxFlags::default();

            for (index, flag) in flag_values.iter().enumerate() {
                flags.set_flag(*flag);

                for set_flag in &flag_values[..=index] {
                    assert!(flags.get_flag(*set_flag));
                }

                for unset_flag in &flag_values[index + 1..] {
                    assert!(!flags.get_flag(*unset_flag));
                }
            }

            flag_values.reverse();
        }
    }
}

use std::num::{ParseFloatError, ParseIntError};

use smallvec::{smallvec, SmallVec};

use crate::{AnnotatedError, LexError, Scope, Span, TokenKind, TokenOwned, Type, TypeConflict};

/// Compilation errors
#[derive(Clone, Debug, PartialEq)]
pub enum CompileError {
    // Token errors
    ExpectedToken {
        expected: TokenKind,
        found: TokenOwned,
        position: Span,
    },
    ExpectedTokenMultiple {
        expected: &'static [TokenKind],
        found: TokenOwned,
        position: Span,
    },

    // Parsing errors
    CannotChainComparison {
        position: Span,
    },
    ExpectedBoolean {
        found: TokenOwned,
        position: Span,
    },
    ExpectedExpression {
        found: TokenOwned,
        position: Span,
    },
    ExpectedFunction {
        found: TokenOwned,
        actual_type: Type,
        position: Span,
    },
    ExpectedFunctionType {
        found: Type,
        position: Span,
    },
    InvalidAssignmentTarget {
        found: TokenOwned,
        position: Span,
    },
    UnexpectedReturn {
        position: Span,
    },

    // Variable errors
    CannotMutateImmutableVariable {
        identifier: String,
        position: Span,
    },
    ExpectedMutableVariable {
        found: TokenOwned,
        position: Span,
    },
    UndeclaredVariable {
        identifier: String,
        position: Span,
    },
    VariableOutOfScope {
        identifier: String,
        variable_scope: Scope,
        access_scope: Scope,
        position: Span,
    },

    // Type errors
    CannotAddType {
        argument_type: Type,
        position: Span,
    },
    CannotAddArguments {
        left_type: Type,
        left_position: Span,
        right_type: Type,
        right_position: Span,
    },
    CannotDivideType {
        argument_type: Type,
        position: Span,
    },
    CannotDivideArguments {
        left_type: Type,
        right_type: Type,
        position: Span,
    },
    CannotModuloType {
        argument_type: Type,
        position: Span,
    },
    CannotModuloArguments {
        left_type: Type,
        right_type: Type,
        position: Span,
    },
    CannotMultiplyType {
        argument_type: Type,
        position: Span,
    },
    CannotMultiplyArguments {
        left_type: Type,
        right_type: Type,
        position: Span,
    },
    CannotSubtractType {
        argument_type: Type,
        position: Span,
    },
    CannotSubtractArguments {
        left_type: Type,
        right_type: Type,
        position: Span,
    },
    CannotResolveRegisterType {
        register_index: usize,
        position: Span,
    },
    CannotResolveVariableType {
        identifier: String,
        position: Span,
    },
    IfElseBranchMismatch {
        conflict: TypeConflict,
        position: Span,
    },
    IfMissingElse {
        position: Span,
    },
    ListItemTypeConflict {
        conflict: TypeConflict,
        position: Span,
    },
    ReturnTypeConflict {
        conflict: TypeConflict,
        position: Span,
    },

    // Chunk errors
    ConstantIndexOutOfBounds {
        index: usize,
        position: Span,
    },
    InstructionIndexOutOfBounds {
        index: usize,
        position: Span,
    },
    LocalIndexOutOfBounds {
        index: usize,
        position: Span,
    },

    // Wrappers around foreign errors
    Lex(LexError),
    ParseFloatError {
        error: ParseFloatError,
        position: Span,
    },
    ParseIntError {
        error: ParseIntError,
        position: Span,
    },
}

impl CompileError {}

impl AnnotatedError for CompileError {
    fn title() -> &'static str {
        "Compilation Error"
    }

    fn description(&self) -> &'static str {
        match self {
            Self::CannotAddArguments { .. } => "Cannot add these types",
            Self::CannotAddType { .. } => "Cannot add to this type",
            Self::CannotChainComparison { .. } => "Cannot chain comparison operations",
            Self::CannotDivideArguments { .. } => "Cannot divide these types",
            Self::CannotDivideType { .. } => "Cannot divide this type",
            Self::CannotModuloArguments { .. } => "Cannot modulo these types",
            Self::CannotModuloType { .. } => "Cannot modulo this type",
            Self::CannotMutateImmutableVariable { .. } => "Cannot mutate immutable variable",
            Self::CannotMultiplyArguments { .. } => "Cannot multiply these types",
            Self::CannotMultiplyType { .. } => "Cannot multiply this type",
            Self::CannotResolveRegisterType { .. } => "Cannot resolve register type",
            Self::CannotResolveVariableType { .. } => "Cannot resolve type",
            Self::CannotSubtractType { .. } => "Cannot subtract from this type",
            Self::CannotSubtractArguments { .. } => "Cannot subtract these types",
            Self::ConstantIndexOutOfBounds { .. } => "Constant index out of bounds",
            Self::ExpectedBoolean { .. } => "Expected a boolean",
            Self::ExpectedExpression { .. } => "Expected an expression",
            Self::ExpectedFunction { .. } => "Expected a function",
            Self::ExpectedFunctionType { .. } => "Expected a function type",
            Self::ExpectedMutableVariable { .. } => "Expected a mutable variable",
            Self::ExpectedToken { .. } => "Expected a specific token",
            Self::ExpectedTokenMultiple { .. } => "Expected one of multiple tokens",
            Self::IfElseBranchMismatch { .. } => "Type mismatch in if/else branches",
            Self::IfMissingElse { .. } => "If statement missing else branch",
            Self::InstructionIndexOutOfBounds { .. } => "Instruction index out of bounds",
            Self::InvalidAssignmentTarget { .. } => "Invalid assignment target",
            Self::Lex(error) => error.description(),
            Self::ListItemTypeConflict { .. } => "List item type conflict",
            Self::LocalIndexOutOfBounds { .. } => "Local index out of bounds",
            Self::ParseFloatError { .. } => "Failed to parse float",
            Self::ParseIntError { .. } => "Failed to parse integer",
            Self::ReturnTypeConflict { .. } => "Return type conflict",
            Self::UndeclaredVariable { .. } => "Undeclared variable",
            Self::UnexpectedReturn { .. } => "Unexpected return",
            Self::VariableOutOfScope { .. } => "Variable out of scope",
        }
    }

    fn detail_snippets(&self) -> SmallVec<[(String, Span); 2]> {
        match self {
            Self::CannotAddArguments {
                left_type,
                left_position,
                right_type,
                right_position,
            } => {
                smallvec![
                    (
                        format!("A value of type \"{left_type}\" was used here."),
                        *left_position
                    ),
                    (
                        format!("A value of type \"{right_type}\" was used here."),
                        *right_position
                    )
                ]
            }
            _ => SmallVec::new(),
        }
    }

    fn help_snippets(&self) -> SmallVec<[(String, Span); 2]> {
        match self {
            Self::CannotAddArguments {
                left_type,
                left_position,
                right_type,
                right_position,
            } => {
                smallvec![(
                    format!("Type \"{left_type}\" cannot be added to type \"{right_type}\". Try converting one of the values to the other type."),
                    Span(left_position.0, right_position.1)
                )]
            }
            _ => SmallVec::new(),
        }
    }
}

impl From<LexError> for CompileError {
    fn from(error: LexError) -> Self {
        Self::Lex(error)
    }
}

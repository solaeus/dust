use std::num::{ParseFloatError, ParseIntError};

use crate::{
    AnnotatedError, BlockScope, ErrorMessage, LexError, Span, TokenKind, TokenOwned, Type,
    TypeConflict,
};

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
    ComparisonChain {
        position: Span,
    },
    DivisionByZero {
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
    InvalidLibraryPath {
        found: String,
    },
    InvalidProgramPath {
        found: String,
    },
    InvalidPath {
        found: String,
        position: Span,
    },
    UnexpectedReturn {
        position: Span,
    },
    UnknownModule {
        module_name: String,
        position: Span,
    },
    UnknownItem {
        item_name: String,
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
    UndeclaredModule {
        path: String,
        position: Span,
    },
    UndeclaredVariable {
        identifier: String,
        position: Span,
    },
    VariableOutOfScope {
        identifier: String,
        variable_scope: BlockScope,
        access_scope: BlockScope,
        position: Span,
    },

    // Type errors
    AdditionTypeInvalid {
        argument_type: Type,
        position: Span,
    },
    AdditionTypeConflict {
        left_type: Type,
        left_position: Span,
        right_type: Type,
        right_position: Span,
    },
    DivisionTypeInvalid {
        argument_type: Type,
        position: Span,
    },
    DivisionTypeConflict {
        left_type: Type,
        right_type: Type,
        position: Span,
    },
    ModuloTypeInvalid {
        argument_type: Type,
        position: Span,
    },
    ModuloTypeConflict {
        left_type: Type,
        right_type: Type,
        position: Span,
    },
    MultiplicationTypeInvalid {
        argument_type: Type,
        position: Span,
    },
    MultiplicationTypeConflict {
        left_type: Type,
        right_type: Type,
        position: Span,
    },
    NegationTypeInvalid {
        argument_type: Type,
        position: Span,
    },
    SubtractionTypeInvalid {
        argument_type: Type,
        position: Span,
    },
    SubtractionTypeConflict {
        left_type: Type,
        right_type: Type,
        position: Span,
    },
    CannotResolveVariableType {
        identifier: String,
        position: Span,
    },
    ComparisonTypeConflict {
        left_type: Type,
        left_position: Span,
        right_type: Type,
        right_position: Span,
    },
    IfElseBranchMismatch {
        conflict: TypeConflict,
        position: Span,
    },
    IfMissingElse {
        position: Span,
    },
    LetStatementTypeConflict {
        conflict: TypeConflict,
        expected_position: Span,
        actual_position: Span,
    },
    ListTypeUnknown {
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

impl AnnotatedError for CompileError {
    fn annotated_error(&self) -> ErrorMessage {
        if let CompileError::Lex(error) = self {
            return error.annotated_error();
        }

        let title = "Compilation Error";

        let (description, detail_snippets, help_snippet) = match self {
            // Token errors
            CompileError::ExpectedToken {
                expected,
                found,
                position,
            } => (
                "Unexpected token",
                vec![(
                    format!("Expected {expected}, found {found}."),
                    *position,
                )],
                None,
            ),
            CompileError::ExpectedTokenMultiple {
                expected,
                found,
                position,
            } => (
                "Unexpected token",
                vec![(
                    format!("Expected one of {expected:?}, found {found}."),
                    *position,
                )],
                None,
            ),

            // Parsing errors
            CompileError::ComparisonChain { position } => (
                "Invalid comparison chain",
                vec![(
                    "Chained comparisons are not allowed; use logical operators (&&, ||) or separate comparisons.".to_string(),
                    *position,
                )],
                None,
            ),
            CompileError::DivisionByZero { position } => (
                "Division by zero",
                vec![("Division by zero.".to_string(), *position)],
                None,
            ),
            CompileError::ExpectedBoolean { found, position } => (
                "Expected a boolean",
                vec![(
                    format!("Expected a boolean expression, found {found}."),
                    *position,
                )],
                None,
            ),
            CompileError::ExpectedExpression { found, position } => (
                "Expected an expression",
                vec![(
                    format!("Expected an expression, found {found}."),
                    *position,
                )],
                None,
            ),
            CompileError::ExpectedFunction { found, position } => (
                "Expected a function",
                vec![(
                    format!("Expected a function value, found {found}."),
                    *position,
                )],
                None,
            ),
            CompileError::ExpectedFunctionType { found, position } => (
                "Expected a function type",
                vec![(
                    format!("Expected a function type, found {found}."),
                    *position,
                )],
                None,
            ),
            CompileError::InvalidAssignmentTarget { found, position } => (
                "Invalid assignment target",
                vec![(
                    format!("Cannot assign to {found} here."),
                    *position,
                )],
                None,
            ),
            CompileError::InvalidLibraryPath { found } => (
                "Invalid library path",
                vec![(
                    format!("Invalid library path: {found}"),
                    Span::default(),
                )],
                None,
            ),
            CompileError::InvalidProgramPath { found } => (
                "Invalid program path",
                vec![(
                    format!("Invalid program path: {found}"),
                    Span::default(),
                )],
                None,
            ),
            CompileError::InvalidPath { found, position } => (
                "Invalid path",
                vec![(format!("Invalid path: {found}"), *position)],
                None,
            ),
            CompileError::UnexpectedReturn { position } => (
                "Unexpected return",
                vec![(
                    "Return statement outside of a function.".to_string(),
                    *position,
                )],
                None,
            ),
            CompileError::UnknownModule {
                module_name,
                position,
            } => (
                "Unknown module",
                vec![(
                    format!("Module '{module_name}' not found."),
                    *position,
                )],
                None,
            ),
            CompileError::UnknownItem { item_name, position } => (
                "Unknown item",
                vec![(
                    format!("Item '{item_name}' not found."),
                    *position,
                )],
                None,
            ),

            // Variable errors
            CompileError::CannotMutateImmutableVariable { identifier, position } => (
                "Cannot mutate immutable variable",
                vec![(
                    format!("Variable '{identifier}' is not mutable."),
                    *position,
                )],
                Some(format!("Declare it as mutable: let mut {identifier} = ...")),
            ),
            CompileError::ExpectedMutableVariable { found, position } => (
                "Expected a mutable variable",
                vec![(
                    format!("Expected a mutable variable on the left-hand side, found {found}."),
                    *position,
                )],
                Some("Compound assignments (+=, -=, *=, /=, %=) require a mutable variable on the left-hand side.".to_string()),
            ),
            CompileError::UndeclaredModule { path, position } => (
                "Undeclared module",
                vec![(
                    format!("Module '{path}' was not declared in this scope."),
                    *position,
                )],
                None,
            ),
            CompileError::UndeclaredVariable {
                identifier,
                position,
            } => (
                "Undeclared variable",
                vec![(
                    format!("Variable '{identifier}' was not declared in this scope."),
                    *position,
                )],
                None,
            ),
            CompileError::VariableOutOfScope {
                identifier,
                variable_scope,
                access_scope,
                position,
            } => (
                "Variable out of scope",
                vec![(
                    format!(
                        "Variable '{identifier}' declared at scope {} is not accessible from scope {}.",
                        variable_scope, access_scope
                    ),
                    *position,
                )],
                None,
            ),

            // Type errors
            CompileError::AdditionTypeInvalid {
                argument_type,
                position,
            } => (
                "Invalid type for addition",
                vec![(
                    format!("Addition is not defined for type {argument_type}."),
                    *position,
                )],
                None,
            ),
            CompileError::AdditionTypeConflict {
                left_type,
                left_position,
                right_type,
                right_position,
            } => (
                "Type conflict in addition",
                vec![
                    (format!("Left operand has type {left_type}."), *left_position),
                    (format!("Right operand has type {right_type}."), *right_position),
                ],
                None,
            ),
            CompileError::DivisionTypeInvalid {
                argument_type,
                position,
            } => (
                "Invalid type for division",
                vec![(
                    format!("Division is not defined for type {argument_type}."),
                    *position,
                )],
                None,
            ),
            CompileError::DivisionTypeConflict {
                left_type,
                right_type,
                position,
            } => (
                "Type conflict in division",
                vec![(
                    format!("Mismatched types in division: left is {left_type}, right is {right_type}."),
                    *position,
                )],
                None,
            ),
            CompileError::ModuloTypeInvalid {
                argument_type,
                position,
            } => (
                "Invalid type for modulo",
                vec![(
                    format!("Modulo is not defined for type {argument_type}."),
                    *position,
                )],
                None,
            ),
            CompileError::ModuloTypeConflict {
                left_type,
                right_type,
                position,
            } => (
                "Type conflict in modulo",
                vec![(
                    format!("Mismatched types in modulo: left is {left_type}, right is {right_type}."),
                    *position,
                )],
                None,
            ),
            CompileError::MultiplicationTypeInvalid {
                argument_type,
                position,
            } => (
                "Invalid type for multiplication",
                vec![(
                    format!("Multiplication is not defined for type {argument_type}."),
                    *position,
                )],
                None,
            ),
            CompileError::MultiplicationTypeConflict {
                left_type,
                right_type,
                position,
            } => (
                "Type conflict in multiplication",
                vec![(
                    format!("Mismatched types in multiplication: left is {left_type}, right is {right_type}."),
                    *position,
                )],
                None,
            ),
            CompileError::NegationTypeInvalid {
                argument_type,
                position,
            } => (
                "Invalid type for negation",
                vec![(
                    format!("Negation is not defined for type {argument_type}."),
                    *position,
                )],
                None,
            ),
            CompileError::SubtractionTypeInvalid {
                argument_type,
                position,
            } => (
                "Invalid type for subtraction",
                vec![(
                    format!("Subtraction is not defined for type {argument_type}."),
                    *position,
                )],
                None,
            ),
            CompileError::SubtractionTypeConflict {
                left_type,
                right_type,
                position,
            } => (
                "Type conflict in subtraction",
                vec![(
                    format!("Mismatched types in subtraction: left is {left_type}, right is {right_type}."),
                    *position,
                )],
                None,
            ),
            CompileError::CannotResolveVariableType { identifier, position } => (
                "Cannot resolve variable type",
                vec![(
                    format!("Type of variable '{identifier}' could not be inferred."),
                    *position,
                )],
                None,
            ),
            CompileError::ComparisonTypeConflict {
                left_type,
                left_position,
                right_type,
                right_position,
            } => (
                "Type conflict in comparison",
                vec![
                    (format!("Left operand has type {left_type}."), *left_position),
                    (format!("Right operand has type {right_type}."), *right_position),
                ],
                None,
            ),
            CompileError::IfElseBranchMismatch { conflict, position } => (
                "Mismatched if/else branch types",
                vec![(
                    format!(
                        "If and else branches must have the same type. Expected {expected}, found {actual}.",
                        expected = conflict.expected,
                        actual = conflict.actual
                    ),
                    *position,
                )],
                None,
            ),
            CompileError::IfMissingElse { position } => (
                "If expression missing else branch",
                vec![(
                    "This 'if' expression is missing an 'else' branch.".to_string(),
                    *position,
                )],
                None,
            ),
            CompileError::LetStatementTypeConflict {
                conflict,
                expected_position,
                actual_position,
            } => (
                "Type conflict in let statement",
                vec![
                    (format!("Declared type is {expected}.", expected = conflict.expected), *expected_position),
                    (format!("Initializer has type {actual}.", actual = conflict.actual), *actual_position),
                ],
                None,
            ),
            CompileError::ListTypeUnknown { position } => (
                "Cannot infer list item type",
                vec![(
                    "The item type of this list could not be inferred.".to_string(),
                    *position,
                )],
                Some("Initialize the list with items so that the type can be inferred or create the list with a `let` binding and give the explicit type: `let foobar: [int] = []`.".to_string()),
            ),
            CompileError::ListItemTypeConflict { conflict, position } => (
                "List item type conflict",
                vec![(
                    format!(
                        "List contains mismatched item types: expected {expected}, found {actual}.",
                        expected = conflict.expected,
                        actual = conflict.actual
                    ),
                    *position,
                )],
                None,
            ),
            CompileError::ReturnTypeConflict { conflict, position } => (
                "Return type conflict",
                vec![(
                    format!(
                        "Function return type mismatch: expected {expected}, found {actual}.",
                        expected = conflict.expected,
                        actual = conflict.actual
                    ),
                    *position,
                )],
                None,
            ),

            // Chunk errors
            CompileError::ConstantIndexOutOfBounds { index, position } => (
                "Constant index out of bounds",
                vec![(
                    format!("Constant index {index} is out of bounds."),
                    *position,
                )],
                None,
            ),
            CompileError::InstructionIndexOutOfBounds { index, position } => (
                "Instruction index out of bounds",
                vec![(
                    format!("Instruction index {index} is out of bounds."),
                    *position,
                )],
                None,
            ),
            CompileError::LocalIndexOutOfBounds { index, position } => (
                "Local index out of bounds",
                vec![(
                    format!("Local index {index} is out of bounds."),
                    *position,
                )],
                None,
            ),

            // Wrappers around foreign errors (non-lex)
            CompileError::ParseFloatError { error, position } => (
                "Invalid float literal",
                vec![(
                    format!("Failed to parse float: {error}."),
                    *position,
                )],
                None,
            ),
            CompileError::ParseIntError { error, position } => (
                "Invalid integer literal",
                vec![(
                    format!("Failed to parse integer: {error}."),
                    *position,
                )],
                None,
            ),
            // The Lex variant is handled at the top of the function with an early return.
            CompileError::Lex(_) => unreachable!(),
        };

        ErrorMessage {
            title,
            description,
            detail_snippets,
            help_snippet,
        }
    }
}

impl From<LexError> for CompileError {
    fn from(error: LexError) -> Self {
        Self::Lex(error)
    }
}

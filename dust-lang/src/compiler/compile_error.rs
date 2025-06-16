use std::num::{ParseFloatError, ParseIntError};

use crate::{
    AnnotatedError, BlockScope, LexError, Span, TokenKind, TokenOwned, Type, TypeConflict,
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
    AnonymousFunctionItem {
        position: Span,
    },
    ComparisonChain {
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
            Self::ComparisonTypeConflict { .. } => "Cannot compare values of different types",
            Self::AnonymousFunctionItem { .. } => {
                "Anonymous functions are not allowed as top-level items"
            }
            Self::AdditionTypeConflict { .. } => "Cannot add these types",
            Self::AdditionTypeInvalid { .. } => "Cannot add this type",
            Self::ComparisonChain { .. } => "Cannot chain comparison operations",
            Self::DivisionTypeConflict { .. } => "Cannot divide these types",
            Self::DivisionTypeInvalid { .. } => "Cannot divide this type",
            Self::ModuloTypeConflict { .. } => "Cannot modulo these types",
            Self::ModuloTypeInvalid { .. } => "Cannot modulo this type",
            Self::CannotMutateImmutableVariable { .. } => "Cannot mutate immutable variable",
            Self::MultiplicationTypeConflict { .. } => "Cannot multiply these types",
            Self::MultiplicationTypeInvalid { .. } => "Cannot multiply this type",
            Self::NegationTypeInvalid { .. } => "Cannot negate this type",
            Self::CannotResolveVariableType { .. } => "Cannot resolve type",
            Self::SubtractionTypeInvalid { .. } => "Cannot subtract from this type",
            Self::SubtractionTypeConflict { .. } => "Cannot subtract these types",
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
            Self::InvalidPath { .. } => "Invalid path",
            Self::LetStatementTypeConflict { .. } => "Let statement type conflict",
            Self::Lex(error) => error.description(),
            Self::ListItemTypeConflict { .. } => "List item type conflict",
            Self::LocalIndexOutOfBounds { .. } => "Local index out of bounds",
            Self::ParseFloatError { .. } => "Failed to parse float",
            Self::ParseIntError { .. } => "Failed to parse integer",
            Self::ReturnTypeConflict { .. } => "Return type conflict",
            Self::UndeclaredModule { .. } => "Undeclared module",
            Self::UndeclaredVariable { .. } => "Undeclared variable",
            Self::UnexpectedReturn { .. } => "Unexpected return",
            Self::UnknownModule { .. } => "Unknown module",
            Self::UnknownItem { .. } => "Unknown item",
            Self::VariableOutOfScope { .. } => "Variable out of scope",
        }
    }

    fn detail_snippets(&self) -> Vec<(String, Span)> {
        match self {
            // Token errors
            Self::ExpectedToken {
                expected,
                found,
                position,
            } => {
                vec![(
                    format!("Expected token `{expected}` but found `{found}`"),
                    *position,
                )]
            }
            Self::ExpectedTokenMultiple {
                expected,
                found,
                position,
            } => {
                vec![(
                    format!("Expected one of the tokens `{expected:?}` but found `{found}`",),
                    *position,
                )]
            }

            // Parsing errors
            Self::AnonymousFunctionItem { position } => {
                vec![(
                    "Anonymous functions are only allowed as expressions".to_string(),
                    *position,
                )]
            }
            Self::ComparisonChain { position } => {
                vec![("Cannot chain comparison operations".to_string(), *position)]
            }
            Self::ExpectedBoolean { found, position } => {
                vec![(format!("Expected a boolean but found `{found}`"), *position)]
            }
            Self::ExpectedExpression { found, position } => {
                vec![(
                    format!("Expected an expression but found `{found}`"),
                    *position,
                )]
            }
            Self::ExpectedFunction {
                found,
                actual_type,
                position,
            } => {
                vec![(
                    format!("Expected a function but found `{found}` of type `{actual_type}`",),
                    *position,
                )]
            }
            Self::ExpectedFunctionType { found, position } => {
                vec![(
                    format!("Expected a function type but found `{found}`"),
                    *position,
                )]
            }
            Self::InvalidAssignmentTarget { found, position } => {
                vec![(format!("Invalid assignment target `{found}`"), *position)]
            }
            Self::InvalidPath { found, position } => {
                vec![(format!("Invalid path `{found}`"), *position)]
            }
            Self::UnexpectedReturn { position } => {
                vec![("Unexpected return statement".to_string(), *position)]
            }
            Self::UnknownModule {
                module_name,
                position,
            } => {
                vec![(format!("Unknown module `{module_name}`"), *position)]
            }
            Self::UnknownItem {
                item_name,
                position,
            } => {
                vec![(format!("Unknown item `{item_name}`"), *position)]
            }

            // Variable errors
            Self::CannotMutateImmutableVariable {
                identifier,
                position,
            } => {
                vec![(
                    format!("Cannot mutate immutable variable `{identifier}`"),
                    *position,
                )]
            }
            Self::ExpectedMutableVariable { found, position } => {
                vec![(
                    format!("Expected a mutable variable but found `{found}`"),
                    *position,
                )]
            }
            Self::UndeclaredModule { path, position } => {
                vec![(format!("Module `{path}` is not in scope"), *position)]
            }
            Self::UndeclaredVariable {
                identifier,
                position,
            } => {
                vec![(
                    format!("Variable `{identifier}` is not declared"),
                    *position,
                )]
            }
            Self::VariableOutOfScope {
                identifier,
                variable_scope,
                access_scope,
                position,
            } => {
                vec![(
                    format!(
                        "Variable `{identifier}` is out of scope. Declared in scope `{variable_scope}` but accessed in scope `{access_scope}`"
                    ),
                    *position,
                )]
            }

            // Type errors
            Self::AdditionTypeInvalid {
                argument_type,
                position,
            } => {
                vec![(format!("Cannot add type `{argument_type}`"), *position)]
            }
            Self::AdditionTypeConflict {
                left_type,
                left_position,
                right_type,
                right_position,
            } => {
                vec![
                    (format!("`{left_type}` value was used here"), *left_position),
                    (
                        format!("`{right_type}` value was used here"),
                        *right_position,
                    ),
                ]
            }
            Self::DivisionTypeInvalid {
                argument_type,
                position,
            } => {
                vec![(format!("Cannot divide type `{argument_type}`"), *position)]
            }
            Self::DivisionTypeConflict {
                left_type,
                right_type,
                position,
            } => {
                vec![(
                    format!("Cannot divide type `{left_type}` by type `{right_type}`",),
                    *position,
                )]
            }
            Self::ModuloTypeInvalid {
                argument_type,
                position,
            } => {
                vec![(
                    format!("Cannot compute modulo for type `{argument_type}`"),
                    *position,
                )]
            }
            Self::ModuloTypeConflict {
                left_type,
                right_type,
                position,
            } => {
                vec![(
                    format!(
                        "Cannot compute modulo for type `{left_type}` with type `{right_type}`"
                    ),
                    *position,
                )]
            }
            Self::MultiplicationTypeInvalid {
                argument_type,
                position,
            } => {
                vec![(format!("Cannot multiply type `{argument_type}`"), *position)]
            }
            Self::MultiplicationTypeConflict {
                left_type,
                right_type,
                position,
            } => {
                vec![(
                    format!("Cannot multiply type `{left_type}` with type `{right_type}`"),
                    *position,
                )]
            }
            Self::NegationTypeInvalid {
                argument_type,
                position,
            } => {
                vec![(format!("Cannot negate type `{argument_type}`"), *position)]
            }
            Self::SubtractionTypeInvalid {
                argument_type,
                position,
            } => {
                vec![(format!("Cannot subtract type `{argument_type}`"), *position)]
            }
            Self::SubtractionTypeConflict {
                left_type,
                right_type,
                position,
            } => {
                vec![(
                    format!("Cannot subtract type `{left_type}` from type `{right_type}`"),
                    *position,
                )]
            }
            Self::CannotResolveVariableType {
                identifier,
                position,
            } => {
                vec![(
                    format!("Cannot resolve type for variable `{identifier}`"),
                    *position,
                )]
            }
            Self::ComparisonTypeConflict {
                left_type,
                left_position,
                right_type,
                right_position,
            } => {
                vec![
                    (format!("`{left_type}` value was used here"), *left_position),
                    (
                        format!("`{right_type}` value was used here"),
                        *right_position,
                    ),
                ]
            }
            Self::IfElseBranchMismatch { conflict, position } => {
                vec![(
                    format!(
                        "Type mismatch in if/else branches: expected `{}` but found `{}`",
                        conflict.expected, conflict.actual
                    ),
                    *position,
                )]
            }
            Self::IfMissingElse { position } => {
                vec![(
                    "If statement is missing an else branch".to_string(),
                    *position,
                )]
            }
            Self::LetStatementTypeConflict {
                conflict,
                expected_position,
                actual_position,
            } => {
                vec![
                    (
                        format!(
                            "Let statement expected type `{}` but found `{}`",
                            conflict.expected, conflict.actual
                        ),
                        *expected_position,
                    ),
                    (
                        format!("Actual type found at `{}`", conflict.actual),
                        *actual_position,
                    ),
                ]
            }
            Self::ListItemTypeConflict { conflict, position } => {
                vec![(
                    format!(
                        "List item type conflict: expected `{}` but found `{}`",
                        conflict.expected, conflict.actual
                    ),
                    *position,
                )]
            }
            Self::ReturnTypeConflict { conflict, position } => {
                vec![(
                    format!(
                        "Return type conflict: expected `{}` but found `{}`",
                        conflict.expected, conflict.actual
                    ),
                    *position,
                )]
            }

            // Chunk errors
            Self::ConstantIndexOutOfBounds { index, position } => {
                vec![(
                    format!("Constant index `{index}` is out of bounds"),
                    *position,
                )]
            }
            Self::InstructionIndexOutOfBounds { index, position } => {
                vec![(
                    format!("Instruction index `{index}` is out of bounds"),
                    *position,
                )]
            }
            Self::LocalIndexOutOfBounds { index, position } => {
                vec![(format!("Local index `{index}` is out of bounds"), *position)]
            }

            // Wrappers around foreign errors
            Self::Lex(error) => error.detail_snippets(),
            Self::ParseFloatError { error, position } => {
                vec![(format!("Failed to parse float: {error}"), *position)]
            }
            Self::ParseIntError { error, position } => {
                vec![(format!("Failed to parse integer: {error}"), *position)]
            }
        }
    }

    fn help_snippets(&self) -> Vec<(String, Span)> {
        match self {
            // Token errors
            Self::ExpectedToken {
                expected, position, ..
            } => {
                vec![(
                    format!("Insert the expected token `{expected}` here"),
                    *position,
                )]
            }
            Self::ExpectedTokenMultiple {
                expected, position, ..
            } => {
                vec![(
                    format!("Insert one of the expected tokens `{expected:?}` here"),
                    *position,
                )]
            }

            // Parsing errors
            Self::AnonymousFunctionItem { position } => {
                vec![("Give the function a name".to_string(), *position)]
            }
            Self::ComparisonChain { position } => {
                vec![(
                    "Break the comparison chain into separate comparisons".to_string(),
                    *position,
                )]
            }
            Self::ExpectedBoolean { position, .. } => {
                vec![(
                    "Provide a boolean value (e.g., `true` or `false`) here".to_string(),
                    *position,
                )]
            }
            Self::ExpectedExpression { position, .. } => {
                vec![("Provide a valid expression here".to_string(), *position)]
            }
            Self::ExpectedFunction { position, .. } => {
                vec![(
                    "Provide a function or callable value here".to_string(),
                    *position,
                )]
            }
            Self::ExpectedFunctionType { position, .. } => {
                vec![("Provide a valid function type here".to_string(), *position)]
            }
            Self::InvalidAssignmentTarget { position, .. } => {
                vec![(
                    "Ensure the left-hand side of the assignment is a valid variable or property"
                        .to_string(),
                    *position,
                )]
            }
            Self::InvalidPath { position, .. } => {
                vec![(
                    "Ensure each part of the path is a valid identifier".to_string(),
                    *position,
                )]
            }
            Self::UnexpectedReturn { position } => {
                vec![(
                    "Remove the `return` statement or place it inside a function".to_string(),
                    *position,
                )]
            }
            Self::UnknownModule { position, .. } => {
                vec![(
                    "Ensure the path is correct and that its root is in scope".to_string(),
                    *position,
                )]
            }
            Self::UnknownItem { position, .. } => {
                vec![("Ensure the item is in scope".to_string(), *position)]
            }

            // Variable errors
            Self::CannotMutateImmutableVariable { position, .. } => {
                vec![(
                    "Declare the variable as `mut` to make it mutable".to_string(),
                    *position,
                )]
            }
            Self::ExpectedMutableVariable { position, .. } => {
                vec![(
                    "Use a mutable variable here or declare it with `mut`".to_string(),
                    *position,
                )]
            }
            Self::UndeclaredModule { position, .. } => {
                vec![(
                    "Declare the module or ensure it is imported".to_string(),
                    *position,
                )]
            }
            Self::UndeclaredVariable { position, .. } => {
                vec![(
                    "Declare the variable before using it".to_string(),
                    *position,
                )]
            }
            Self::VariableOutOfScope { position, .. } => {
                vec![(
                    "Ensure the variable is declared in the current scope or passed as an argument"
                        .to_string(),
                    *position,
                )]
            }

            // Type errors
            Self::AdditionTypeConflict {
                left_position,
                right_position,
                ..
            } => {
                vec![(
                    "Ensure both arguments are of compatible types for addition".to_string(),
                    Span(left_position.0, right_position.1),
                )]
            }
            Self::AdditionTypeInvalid { position, .. } => {
                vec![("Ensure the type supports addition".to_string(), *position)]
            }
            Self::DivisionTypeConflict { position, .. } => {
                vec![(
                    "Ensure both arguments are of compatible types for division".to_string(),
                    *position,
                )]
            }
            Self::DivisionTypeInvalid { position, .. } => {
                vec![("Ensure the type supports division".to_string(), *position)]
            }
            Self::ModuloTypeConflict { position, .. } => {
                vec![(
                    "Ensure both arguments are of compatible types for modulo operation"
                        .to_string(),
                    *position,
                )]
            }
            Self::ModuloTypeInvalid { position, .. } => {
                vec![(
                    "Ensure the type supports modulo operation".to_string(),
                    *position,
                )]
            }
            Self::MultiplicationTypeConflict { position, .. } => {
                vec![(
                    "Ensure both arguments are of compatible types for multiplication".to_string(),
                    *position,
                )]
            }
            Self::MultiplicationTypeInvalid { position, .. } => {
                vec![(
                    "Ensure the type supports multiplication".to_string(),
                    *position,
                )]
            }
            Self::NegationTypeInvalid { position, .. } => {
                vec![(
                    "Ensure the type supports negation (e.g., numeric types)".to_string(),
                    *position,
                )]
            }
            Self::SubtractionTypeConflict { position, .. } => {
                vec![(
                    "Ensure both arguments are of compatible types for subtraction".to_string(),
                    *position,
                )]
            }
            Self::SubtractionTypeInvalid { position, .. } => {
                vec![(
                    "Ensure the type supports subtraction".to_string(),
                    *position,
                )]
            }
            Self::CannotResolveVariableType { position, .. } => {
                vec![(
                    "Ensure the variable is declared with a valid type".to_string(),
                    *position,
                )]
            }
            Self::ComparisonTypeConflict {
                left_position,
                right_position,
                ..
            } => {
                vec![(
                    "Convert one of the values to match the type of the other".to_string(),
                    Span(left_position.0, right_position.1),
                )]
            }
            Self::IfElseBranchMismatch { position, .. } => {
                vec![(
                    "Ensure both branches of the if/else statement return the same type"
                        .to_string(),
                    *position,
                )]
            }
            Self::IfMissingElse { position } => {
                vec![(
                    "Add an else branch to handle all possible cases".to_string(),
                    *position,
                )]
            }
            Self::LetStatementTypeConflict {
                expected_position,
                actual_position,
                ..
            } => {
                vec![
                    (
                        "Ensure the type of the value matches the expected type".to_string(),
                        *expected_position,
                    ),
                    (
                        "Check the actual type of the value being assigned".to_string(),
                        *actual_position,
                    ),
                ]
            }
            Self::ListItemTypeConflict { position, .. } => {
                vec![(
                    "Ensure all items in the list are of the same type".to_string(),
                    *position,
                )]
            }
            Self::ReturnTypeConflict { position, .. } => {
                vec![(
                    "Ensure the return type matches the function's declared return type"
                        .to_string(),
                    *position,
                )]
            }

            // Chunk errors
            Self::ConstantIndexOutOfBounds { position, .. } => {
                vec![(
                    "Ensure the constant index is within the valid range".to_string(),
                    *position,
                )]
            }
            Self::InstructionIndexOutOfBounds { position, .. } => {
                vec![(
                    "Ensure the instruction index is within the valid range".to_string(),
                    *position,
                )]
            }
            Self::LocalIndexOutOfBounds { position, .. } => {
                vec![(
                    "Ensure the local index is within the valid range".to_string(),
                    *position,
                )]
            }

            // Wrappers around foreign errors
            Self::Lex(_) => vec![(
                "Fix the lexing error in the source code".to_string(),
                Span(0, 0),
            )],
            Self::ParseFloatError { position, .. } => {
                vec![(
                    "Ensure the float value is valid and properly formatted".to_string(),
                    *position,
                )]
            }
            Self::ParseIntError { position, .. } => {
                vec![(
                    "Ensure the integer value is valid and properly formatted".to_string(),
                    *position,
                )]
            }
        }
    }
}

impl From<LexError> for CompileError {
    fn from(error: LexError) -> Self {
        Self::Lex(error)
    }
}

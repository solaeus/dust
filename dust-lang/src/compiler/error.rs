use crate::{
    Span,
    dust_error::{AnnotatedError, ErrorMessage},
    resolver::TypeId,
    syntax_tree::{SyntaxId, SyntaxKind},
};

const INVALID_TREE: &str = "The syntax tree is invalid, this is a bug in the parser.";

#[derive(Debug, Clone)]
pub enum CompileError {
    InvalidEncodedConstant {
        node_kind: SyntaxKind,
        position: Span,
        payload: (u32, u32),
    },
    DivisionByZero {
        node_kind: SyntaxKind,
        position: Span,
    },
    ExpectedItem {
        node_kind: SyntaxKind,
        position: Span,
    },
    ExpectedStatement {
        node_kind: SyntaxKind,
        position: Span,
    },
    ExpectedExpression {
        node_kind: SyntaxKind,
        position: Span,
    },
    MissingChild {
        parent_kind: SyntaxKind,
        child_index: u32,
    },
    MissingConstant {
        constant_index: u16,
    },
    MissingSyntaxNode {
        id: SyntaxId,
    },
    MissingType {
        type_id: TypeId,
    },
}

impl AnnotatedError for CompileError {
    fn annotated_error(&self) -> ErrorMessage {
        let title = "Compilation Error";

        match self {
            CompileError::DivisionByZero { position, .. } => ErrorMessage {
                title,
                description: "Dividing by zero is mathematically undefined for integers. Dust does not allow it.",
                detail_snippets: vec![("This value is zero.".to_string(), *position)],
                help_snippet: Some("This is a compile-time error caused by hard-coded values. Check your math for errors. If you absolutely must divide by zero, floats allow it but the result is always Infinity or NaN.".to_string()),
            },
            CompileError::ExpectedItem { node_kind, position } => ErrorMessage {
                title,
                description: "Expected an item.",
                detail_snippets: vec![(node_kind.to_string(), *position)],
                help_snippet: Some(INVALID_TREE.to_string()),
            },
            CompileError::ExpectedStatement { node_kind, position } => ErrorMessage {
                title,
                description: "Expected a statement.",
                detail_snippets: vec![(node_kind.to_string(), *position)],
                help_snippet: Some(INVALID_TREE.to_string()),
            },
            CompileError::ExpectedExpression { node_kind, position } => ErrorMessage {
                title,
                description: "Expected an expression.",
                detail_snippets: vec![(node_kind.to_string(), *position)],
                help_snippet: Some(INVALID_TREE.to_string()),
            },
            CompileError::InvalidEncodedConstant {
                node_kind,
                position,
                payload,
            } => ErrorMessage {
                title,
                description: "The syntax tree contains an encoded constant that is invalid.",
                detail_snippets: vec![
                    (node_kind.to_string(), *position),
                    (format!("Payload: {:?}", payload), *position),
                ],
                help_snippet: Some(INVALID_TREE.to_string()),
            },
            CompileError::MissingChild {
                parent_kind,
                child_index,
            } => ErrorMessage {
                title,
                description: "The syntax tree is missing a child index.",
                detail_snippets: vec![(format!("Parent node kind {parent_kind}, child index {child_index}"), Span::default())],
                help_snippet: Some(INVALID_TREE.to_string()),
            },
            CompileError::MissingConstant { constant_index } => ErrorMessage {
                title,
                description: "The syntax tree is missing a constant that is required for compilation.",
                detail_snippets: vec![(format!("Constant index {constant_index}"), Span::default())],
                help_snippet: Some(INVALID_TREE.to_string()),
            },
            CompileError::MissingSyntaxNode { id } => ErrorMessage {
                title,
                description: "The syntax tree is missing a node that is required for compilation.",
                detail_snippets: vec![(format!("Node id: {}", id.0), Span::default())],
                help_snippet: Some(INVALID_TREE.to_string()),
            },
            CompileError::MissingType { type_id } => ErrorMessage {
                title,
                description: "A type required for compilation is missing from the resolver.",
                detail_snippets: vec![(format!("Type id: {}", type_id.0), Span::default())],
                help_snippet: Some(INVALID_TREE.to_string()),
            },
        }
    }
}

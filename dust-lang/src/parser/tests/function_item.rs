use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{FileId, Span},
    syntax::{
        SyntaxId,
        node::{SyntaxKind::*, SyntaxPayload},
    },
};

#[test]
fn empty() {
    let parser = Parser::new(
        FileId::MAIN,
        Lexer::with_unvalidated_source(b"fn foo() {}"),
    );
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 11), SyntaxId(6)),
            FunctionItem
                .with_multiple_children(Span::new(0, 11), SyntaxPayload::child_indices(1, 4)),
            SimplePath.empty(Span::new(3, 6)),
            FunctionSignature
                .with_multiple_children(Span::new(0, 8), SyntaxPayload::child_indices(0, 1)),
            FunctionParameters.with_child(Span::new(0, 8), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 8)),
            BlockExpression.empty(Span::new(9, 11)),
        ]
    );
}

#[test]
fn value_parameters() {
    let parser = Parser::new(
        FileId::MAIN,
        Lexer::with_unvalidated_source(b"fn foo(x: i64, y: bool) {}"),
    );
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 26), SyntaxId(10)),
            FunctionItem
                .with_multiple_children(Span::new(0, 26), SyntaxPayload::child_indices(5, 8)),
            SimplePath.empty(Span::new(3, 6)),
            FunctionSignature
                .with_multiple_children(Span::new(0, 23), SyntaxPayload::child_indices(4, 5)),
            FunctionParameters.with_child(Span::new(0, 23), SyntaxId(6)),
            ValueParameters
                .with_multiple_children(Span::new(0, 23), SyntaxPayload::child_indices(0, 4)),
            SimplePath.empty(Span::new(7, 8)),
            I64Type.empty(Span::new(10, 13)),
            SimplePath.empty(Span::new(15, 16)),
            BooleanType.empty(Span::new(18, 22)),
            BlockExpression.empty(Span::new(24, 26)),
        ]
    );
}

#[test]
fn type_parameters() {
    let parser = Parser::new(
        FileId::MAIN,
        Lexer::with_unvalidated_source(b"fn foo<A, B, C>() {}"),
    );
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 20), SyntaxId(13)),
            FunctionItem
                .with_multiple_children(Span::new(0, 20), SyntaxPayload::child_indices(4, 7)),
            SimplePath.empty(Span::new(3, 6)),
            FunctionSignature
                .with_multiple_children(Span::new(0, 17), SyntaxPayload::child_indices(3, 4)),
            FunctionParameters.with_binary_children(Span::new(0, 17), SyntaxId(9), SyntaxId(8)),
            ValueParameters.empty(Span::new(0, 17)),
            TypeParameters
                .with_multiple_children(Span::new(6, 15), SyntaxPayload::child_indices(0, 3)),
            TypeParameter.with_child(Span::new(7, 8), SyntaxId(2)),
            SimplePath.empty(Span::new(7, 8)),
            TypeParameter.with_child(Span::new(10, 11), SyntaxId(4)),
            SimplePath.empty(Span::new(10, 11)),
            TypeParameter.with_child(Span::new(13, 14), SyntaxId(6)),
            SimplePath.empty(Span::new(13, 14)),
            BlockExpression.empty(Span::new(18, 20)),
        ]
    );
}

#[test]
fn return_type() {
    let parser = Parser::new(
        FileId::MAIN,
        Lexer::with_unvalidated_source(b"fn foo() -> i64 {}"),
    );
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 18), SyntaxId(7)),
            FunctionItem
                .with_multiple_children(Span::new(0, 18), SyntaxPayload::child_indices(2, 5)),
            SimplePath.empty(Span::new(3, 6)),
            {
                let mut node = FunctionSignature
                    .with_multiple_children(Span::new(0, 15), SyntaxPayload::child_indices(0, 2));
                node.modifier.set_has_return_type();
                node
            },
            FunctionParameters.with_child(Span::new(0, 8), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 8)),
            I64Type.empty(Span::new(12, 15)),
            BlockExpression.empty(Span::new(16, 18)),
        ]
    );
}

#[test]
fn mixed() {
    let parser = Parser::new(
        FileId::MAIN,
        Lexer::with_unvalidated_source(b"fn foo<A, B, C>(x: A, y: B) -> C {}"),
    );
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 35), SyntaxId(21)),
            FunctionItem
                .with_multiple_children(Span::new(0, 35), SyntaxPayload::child_indices(9, 12)),
            SimplePath.empty(Span::new(3, 6)),
            {
                let mut node = FunctionSignature
                    .with_multiple_children(Span::new(0, 32), SyntaxPayload::child_indices(7, 9));
                node.modifier.set_has_return_type();
                node
            },
            FunctionParameters.with_binary_children(Span::new(0, 27), SyntaxId(15), SyntaxId(8)),
            ValueParameters
                .with_multiple_children(Span::new(0, 27), SyntaxPayload::child_indices(3, 7)),
            SimplePath.empty(Span::new(16, 17)),
            TypePath.with_child(Span::new(19, 20), SyntaxId(10)),
            PathSegment.empty(Span::new(19, 20)),
            SimplePath.empty(Span::new(22, 23)),
            TypePath.with_child(Span::new(25, 26), SyntaxId(13)),
            PathSegment.empty(Span::new(25, 26)),
            TypeParameters
                .with_multiple_children(Span::new(6, 15), SyntaxPayload::child_indices(0, 3)),
            TypeParameter.with_child(Span::new(7, 8), SyntaxId(2)),
            SimplePath.empty(Span::new(7, 8)),
            TypeParameter.with_child(Span::new(10, 11), SyntaxId(4)),
            SimplePath.empty(Span::new(10, 11)),
            TypeParameter.with_child(Span::new(13, 14), SyntaxId(6)),
            SimplePath.empty(Span::new(13, 14)),
            TypePath.with_child(Span::new(31, 32), SyntaxId(17)),
            PathSegment.empty(Span::new(31, 32)),
            BlockExpression.empty(Span::new(33, 35)),
        ]
    );
}

use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceFileId, Span},
    syntax::{
        SyntaxId,
        node::{SyntaxKind::*, SyntaxPayload},
    },
};

#[test]
fn empty() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(b"trait Foo {}"));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 12), SyntaxId(3)),
            TraitItem.with_binary_children(Span::new(0, 12), SyntaxId(1), SyntaxId(2)),
            SimplePath.empty(Span::new(6, 9)),
            TraitBody.empty(Span::new(10, 12)),
        ]
    );
}

#[test]
fn with_supertraits() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(b"trait Foo: Bar + Baz {}"),
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
            Root.with_child(Span::new(0, 23), SyntaxId(7)),
            TraitItem
                .with_multiple_children(Span::new(0, 23), SyntaxPayload::child_indices(0, 4)),
            SimplePath.empty(Span::new(6, 9)),
            TraitBody.empty(Span::new(21, 23)),
            TraitBound.with_child(Span::new(11, 14), SyntaxId(2)),
            PathSegment.empty(Span::new(11, 14)),
            TraitBound.with_child(Span::new(17, 20), SyntaxId(4)),
            PathSegment.empty(Span::new(17, 20)),
        ]
    );
}

#[test]
fn with_type_parameters_and_supertraits() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(b"trait Foo<T>: Bar + Baz {}"),
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
            Root.with_child(Span::new(0, 26), SyntaxId(9)),
            TraitItem
                .with_multiple_children(Span::new(0, 26), SyntaxPayload::child_indices(0, 5)),
            SimplePath.empty(Span::new(6, 9)),
            TraitBody.empty(Span::new(24, 26)),
            TypeParameters.with_child(Span::new(9, 12), SyntaxId(2)),
            SimplePath.empty(Span::new(10, 11)),
            TraitBound.with_child(Span::new(14, 17), SyntaxId(4)),
            PathSegment.empty(Span::new(14, 17)),
            TraitBound.with_child(Span::new(20, 23), SyntaxId(6)),
            PathSegment.empty(Span::new(20, 23)),
        ]
    );
}

#[test]
fn with_where_clause() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(b"trait Foo<T> where T: Bar {}"),
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
            Root.with_child(Span::new(0, 28), SyntaxId(10)),
            TraitItem
                .with_multiple_children(Span::new(0, 28), SyntaxPayload::child_indices(0, 4)),
            SimplePath.empty(Span::new(6, 9)),
            TraitBody.empty(Span::new(26, 28)),
            TypeParameters.with_child(Span::new(9, 12), SyntaxId(2)),
            SimplePath.empty(Span::new(10, 11)),
            WhereClause.with_binary_children(Span::new(13, 25), SyntaxId(5), SyntaxId(7)),
            TypePath.with_child(Span::new(19, 20), SyntaxId(4)),
            PathSegment.empty(Span::new(19, 20)),
            TraitBound.with_child(Span::new(22, 25), SyntaxId(6)),
            PathSegment.empty(Span::new(22, 25)),
        ]
    );
}

#[test]
fn with_const_member() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(b"trait Foo { const X: i64; }"),
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
            Root.with_child(Span::new(0, 27), SyntaxId(6)),
            TraitItem.with_binary_children(Span::new(0, 27), SyntaxId(1), SyntaxId(5)),
            SimplePath.empty(Span::new(6, 9)),
            TraitBody.with_child(Span::new(10, 27), SyntaxId(4)),
            TraitConst.with_binary_children(Span::new(12, 25), SyntaxId(2), SyntaxId(3)),
            SimplePath.empty(Span::new(18, 19)),
            I64Type.empty(Span::new(21, 24)),
        ]
    );
}

#[test]
fn with_const_member_default() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(b"trait Foo { const X: i64 = 42; }"),
    );
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");

    let nodes = syntax_tree.sorted_nodes();

    assert_eq!(nodes.len(), 8, "expected 8 nodes, got {}: {nodes:#?}", nodes.len());
}

#[test]
fn with_method_signature() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(b"trait Foo { fn bar(self); }"),
    );
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");

    let nodes = syntax_tree.sorted_nodes();

    assert_eq!(nodes.len(), 8, "expected 8 nodes, got {}: {nodes:#?}", nodes.len());
}

#[test]
fn with_default_method() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(b"trait Foo { fn bar(self) {} }"),
    );
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");

    let nodes = syntax_tree.sorted_nodes();

    assert_eq!(nodes.len(), 9, "expected 9 nodes, got {}: {nodes:#?}", nodes.len());
}

#[test]
fn with_method_signature_and_return_type() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(b"trait Foo { fn bar(self) -> i64; }"),
    );
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");

    let nodes = syntax_tree.sorted_nodes();

    assert_eq!(nodes.len(), 9, "expected 9 nodes, got {}: {nodes:#?}", nodes.len());
}

#[test]
fn with_type_member() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(b"trait Foo { type Bar; }"),
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
            Root.with_child(Span::new(0, 23), SyntaxId(5)),
            TraitItem.with_binary_children(Span::new(0, 23), SyntaxId(1), SyntaxId(4)),
            SimplePath.empty(Span::new(6, 9)),
            TraitBody.with_child(Span::new(10, 23), SyntaxId(3)),
            TraitType.with_child(Span::new(12, 21), SyntaxId(2)),
            SimplePath.empty(Span::new(17, 20)),
        ]
    );
}

#[test]
fn with_type_member_default() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(b"trait Foo { type Bar = i64; }"),
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
            Root.with_child(Span::new(0, 29), SyntaxId(6)),
            TraitItem.with_binary_children(Span::new(0, 29), SyntaxId(1), SyntaxId(5)),
            SimplePath.empty(Span::new(6, 9)),
            TraitBody.with_child(Span::new(10, 29), SyntaxId(4)),
            TraitType.with_binary_children(Span::new(12, 27), SyntaxId(2), SyntaxId(3)),
            SimplePath.empty(Span::new(17, 20)),
            I64Type.empty(Span::new(23, 26)),
        ]
    );
}

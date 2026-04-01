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
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(b"impl Foo {}"));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 11), SyntaxId(4)),
            ImplItem.with_binary_children(Span::new(0, 11), SyntaxId(2), SyntaxId(3)),
            TypePath.with_child(Span::new(5, 8), SyntaxId(1)),
            PathSegment.empty(Span::new(5, 8)),
            ImplBody.empty(Span::new(9, 11)),
        ]
    );
}

#[test]
fn with_function() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(b"impl Foo { fn bar(self) {} }"),
    );
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");

    let nodes = syntax_tree.sorted_nodes();

    assert_eq!(nodes.len(), 10, "expected 10 nodes, got {}: {nodes:#?}", nodes.len());
}

#[test]
fn with_pub_function() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(b"impl Foo { pub fn bar(self) {} }"),
    );
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");

    let nodes = syntax_tree.sorted_nodes();

    assert_eq!(nodes.len(), 10, "expected 10 nodes, got {}: {nodes:#?}", nodes.len());
}

#[test]
fn trait_impl() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(b"impl Bar for Foo {}"),
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
            Root.with_child(Span::new(0, 19), SyntaxId(6)),
            ImplItem
                .with_multiple_children(Span::new(0, 19), SyntaxPayload::child_indices(0, 3)),
            TypePath.with_child(Span::new(13, 16), SyntaxId(3)),
            PathSegment.empty(Span::new(13, 16)),
            ImplBody.empty(Span::new(17, 19)),
            TypePath.with_child(Span::new(5, 8), SyntaxId(1)),
            PathSegment.empty(Span::new(5, 8)),
        ]
    );
}

#[test]
fn with_where_clause() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(b"impl Foo where Foo: Bar {}"),
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
            ImplItem
                .with_multiple_children(Span::new(0, 26), SyntaxPayload::child_indices(0, 3)),
            TypePath.with_child(Span::new(5, 8), SyntaxId(1)),
            PathSegment.empty(Span::new(5, 8)),
            ImplBody.empty(Span::new(24, 26)),
            WhereClause.with_binary_children(Span::new(9, 23), SyntaxId(4), SyntaxId(6)),
            TypePath.with_child(Span::new(15, 18), SyntaxId(3)),
            PathSegment.empty(Span::new(15, 18)),
            TraitBound.with_child(Span::new(20, 23), SyntaxId(5)),
            PathSegment.empty(Span::new(20, 23)),
        ]
    );
}

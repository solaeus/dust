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
    let parser = Parser::new(SourceFileId::MAIN, Lexer::with_unvalidated_source(b"impl Foo {}"));
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
        Lexer::with_unvalidated_source(b"impl Foo { fn bar(self) {} }"),
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
            Root.with_child(Span::new(0, 28), SyntaxId(11)),
            ImplItem.with_binary_children(Span::new(0, 28), SyntaxId(2), SyntaxId(10)),
            TypePath.with_child(Span::new(5, 8), SyntaxId(1)),
            PathSegment.empty(Span::new(5, 8)),
            ImplBody.with_child(Span::new(9, 28), SyntaxId(9)),
            FunctionItem
                .with_multiple_children(Span::new(11, 26), SyntaxPayload::child_indices(0, 3),),
            SimplePath.empty(Span::new(14, 17)),
            FunctionParameters.with_child(Span::new(11, 23), SyntaxId(6)),
            ValueParameters.with_binary_children(Span::new(11, 23), SyntaxId(4), SyntaxId(5)),
            SimplePath.empty(Span::new(18, 22)),
            SelfType.empty(Span::new(18, 22)),
            BlockExpression.empty(Span::new(24, 26)),
        ]
    );
}

#[test]
fn with_pub_function() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(b"impl Foo { pub fn bar(self) {} }"),
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
            Root.with_child(Span::new(0, 32), SyntaxId(11)),
            ImplItem.with_binary_children(Span::new(0, 32), SyntaxId(2), SyntaxId(10)),
            TypePath.with_child(Span::new(5, 8), SyntaxId(1)),
            PathSegment.empty(Span::new(5, 8)),
            ImplBody.with_child(Span::new(9, 32), SyntaxId(9)),
            FunctionItem
                .with_multiple_children(Span::new(15, 30), SyntaxPayload::child_indices(0, 3),)
                .with_modifier(true),
            SimplePath.empty(Span::new(18, 21)),
            FunctionParameters.with_child(Span::new(15, 27), SyntaxId(6)),
            ValueParameters.with_binary_children(Span::new(15, 27), SyntaxId(4), SyntaxId(5)),
            SimplePath.empty(Span::new(22, 26)),
            SelfType.empty(Span::new(22, 26)),
            BlockExpression.empty(Span::new(28, 30)),
        ]
    );
}

#[test]
fn trait_impl() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(b"impl Bar for Foo {}"),
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
            ImplTraitItem
                .with_multiple_children(Span::new(0, 19), SyntaxPayload::child_indices(0, 3)),
            TypePath.with_child(Span::new(13, 16), SyntaxId(2)),
            PathSegment.empty(Span::new(13, 16)),
            ImplBody.empty(Span::new(17, 19)),
            Path.with_child(Span::new(5, 8), SyntaxId(1)),
            PathSegment.empty(Span::new(5, 8)),
        ]
    );
}

#[test]
fn with_where_clause() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::with_unvalidated_source(b"impl Foo where Foo: Bar {}"),
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
            Root.with_child(Span::new(0, 26), SyntaxId(11)),
            ImplItem.with_multiple_children(Span::new(0, 26), SyntaxPayload::child_indices(0, 3)),
            TypePath.with_child(Span::new(5, 8), SyntaxId(1)),
            PathSegment.empty(Span::new(5, 8)),
            ImplBody.empty(Span::new(24, 26)),
            WhereClause.with_child(Span::new(9, 23), SyntaxId(8)),
            WherePredicate.with_binary_children(Span::new(15, 23), SyntaxId(4), SyntaxId(7)),
            TypePath.with_child(Span::new(15, 18), SyntaxId(3)),
            PathSegment.empty(Span::new(15, 18)),
            TraitBounds.with_child(Span::new(20, 23), SyntaxId(6)),
            Path.with_child(Span::new(20, 23), SyntaxId(5)),
            PathSegment.empty(Span::new(20, 23)),
        ]
    );
}

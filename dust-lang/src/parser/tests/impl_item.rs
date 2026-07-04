use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::Span,
    syntax::{
        SyntaxId,
        node::{SyntaxChildren, SyntaxFlags, SyntaxKind::*},
    },
};

#[test]
fn empty() {
    let parser = Parser::new_standalone(Lexer::unvalidated(b"impl Foo {}"));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 11), SyntaxId(4)),
            ImplItem.with_binary_children(Span::new(0, 11), SyntaxId(2), SyntaxId(3)),
            TypePath.with_single_child(Span::new(5, 8), SyntaxId(1)),
            PathSegment.empty(Span::new(5, 8)),
            ImplBody.empty(Span::new(9, 11)),
        ]
    );
}

#[test]
fn with_function() {
    let parser = Parser::new_standalone(Lexer::unvalidated(b"impl Foo { fn bar(self) {} }"));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 28), SyntaxId(8)),
            ImplItem.with_binary_children(Span::new(0, 28), SyntaxId(2), SyntaxId(7)),
            TypePath.with_single_child(Span::new(5, 8), SyntaxId(1)),
            PathSegment.empty(Span::new(5, 8)),
            ImplBody.with_single_child(Span::new(9, 28), SyntaxId(6)),
            FunctionItem
                .with_children(Span::new(11, 26), SyntaxChildren::new(0, 3))
                .with_flags(SyntaxFlags::VALUE_PARAMETERS),
            SimplePath.empty(Span::new(14, 17)),
            ValueParameters
                .empty(Span::new(17, 23))
                .with_flags(SyntaxFlags::SELF_VALUE),
            BlockExpression.empty(Span::new(24, 26)),
        ]
    );
}

#[test]
fn with_pub_function() {
    let parser = Parser::new_standalone(Lexer::unvalidated(b"impl Foo { pub fn bar(self) {} }"));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 32), SyntaxId(8)),
            ImplItem.with_binary_children(Span::new(0, 32), SyntaxId(2), SyntaxId(7)),
            TypePath.with_single_child(Span::new(5, 8), SyntaxId(1)),
            PathSegment.empty(Span::new(5, 8)),
            ImplBody.with_single_child(Span::new(9, 32), SyntaxId(6)),
            FunctionItem
                .with_children(Span::new(15, 30), SyntaxChildren::new(0, 3))
                .with_flags(SyntaxFlags::PUBLIC.and(SyntaxFlags::VALUE_PARAMETERS)),
            SimplePath.empty(Span::new(18, 21)),
            ValueParameters
                .empty(Span::new(21, 27))
                .with_flags(SyntaxFlags::SELF_VALUE),
            BlockExpression.empty(Span::new(28, 30)),
        ]
    );
}

#[test]
fn trait_impl() {
    let parser = Parser::new_standalone(Lexer::unvalidated(b"impl Bar for Foo {}"));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 19), SyntaxId(6)),
            ImplItem
                .with_children(Span::new(0, 19), SyntaxChildren::new(0, 3))
                .with_flags(SyntaxFlags::TYPE_NAME),
            Path.with_single_child(Span::new(5, 8), SyntaxId(1)),
            PathSegment.empty(Span::new(5, 8)),
            TypePath.with_single_child(Span::new(13, 16), SyntaxId(3)),
            PathSegment.empty(Span::new(13, 16)),
            ImplBody.empty(Span::new(17, 19)),
        ]
    );
}

#[test]
fn with_where_clause() {
    let parser = Parser::new_standalone(Lexer::unvalidated(b"impl Foo where Foo: Bar {}"));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 26), SyntaxId(11)),
            ImplItem
                .with_children(Span::new(0, 26), SyntaxChildren::new(0, 3))
                .with_flags(SyntaxFlags::WHERE_CLAUSE),
            TypePath.with_single_child(Span::new(5, 8), SyntaxId(1)),
            PathSegment.empty(Span::new(5, 8)),
            WhereClause.with_single_child(Span::new(9, 23), SyntaxId(8)),
            WherePredicate.with_binary_children(Span::new(15, 23), SyntaxId(4), SyntaxId(7)),
            TypePath.with_single_child(Span::new(15, 18), SyntaxId(3)),
            PathSegment.empty(Span::new(15, 18)),
            TraitBounds.with_single_child(Span::new(20, 23), SyntaxId(6)),
            Path.with_single_child(Span::new(20, 23), SyntaxId(5)),
            PathSegment.empty(Span::new(20, 23)),
            ImplBody.empty(Span::new(24, 26)),
        ]
    );
}

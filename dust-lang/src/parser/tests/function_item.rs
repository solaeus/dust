use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceFileId, Span},
    syntax::{SyntaxId, SyntaxKind::*},
};

#[test]
fn empty() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(b"fn foo() {}"));
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
            FunctionItem.with_binary_children(Span::new(0, 11), SyntaxId(1), SyntaxId(5)),
            SimplePath.empty(Span::new(3, 6)),
            FunctionExpression.with_binary_children(Span::new(6, 11), SyntaxId(3), SyntaxId(4)),
            FunctionSignature.with_child(Span::new(6, 8), SyntaxId(2)),
            ValueParameters.empty(Span::new(6, 8)),
            BlockExpression.empty(Span::new(9, 11)),
        ]
    );
}

#[test]
fn value_parameters() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(b"fn foo(x: int, y: bool) {}"),
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
            Root.with_child(Span::new(0, 26), SyntaxId(12)),
            FunctionItem.with_binary_children(Span::new(0, 26), SyntaxId(1), SyntaxId(11)),
            SimplePath.empty(Span::new(3, 6)),
            FunctionExpression.with_binary_children(Span::new(6, 26), SyntaxId(9), SyntaxId(10)),
            FunctionSignature.with_child(Span::new(6, 23), SyntaxId(8)),
            ValueParameters.with_multiple_children(Span::new(6, 23), 0, 4),
            Path.with_child(Span::new(7, 8), SyntaxId(2)),
            PathSegment.empty(Span::new(7, 8)),
            IntegerType.empty(Span::new(10, 13)),
            Path.with_child(Span::new(15, 16), SyntaxId(5)),
            PathSegment.empty(Span::new(15, 16)),
            BooleanType.empty(Span::new(18, 22)),
            BlockExpression.empty(Span::new(24, 26)),
        ]
    );
}

#[test]
fn type_parameters() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(b"fn foo<A, B, C>() {}"),
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
            Root.with_child(Span::new(0, 20), SyntaxId(10)),
            FunctionItem.with_binary_children(Span::new(0, 20), SyntaxId(1), SyntaxId(9)),
            SimplePath.empty(Span::new(3, 6)),
            FunctionExpression.with_binary_children(Span::new(6, 20), SyntaxId(7), SyntaxId(8)),
            FunctionSignature.with_binary_children(Span::new(6, 17), SyntaxId(6), SyntaxId(5)),
            ValueParameters.empty(Span::new(15, 17)),
            TypeParameters.with_multiple_children(Span::new(6, 15), 0, 3),
            SimplePath.empty(Span::new(7, 8)),
            SimplePath.empty(Span::new(10, 11)),
            SimplePath.empty(Span::new(13, 14)),
            BlockExpression.empty(Span::new(18, 20)),
        ]
    );
}

#[test]
fn return_type() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(b"fn foo() -> int {}"));
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
            FunctionItem.with_binary_children(Span::new(0, 18), SyntaxId(1), SyntaxId(6)),
            SimplePath.empty(Span::new(3, 6)),
            FunctionExpression.with_binary_children(Span::new(6, 18), SyntaxId(4), SyntaxId(5)),
            FunctionSignature.with_binary_children(Span::new(6, 15), SyntaxId(2), SyntaxId(3)),
            ValueParameters.empty(Span::new(6, 8)),
            IntegerType.empty(Span::new(12, 15)),
            BlockExpression.empty(Span::new(16, 18)),
        ]
    );
}

#[test]
fn mixed() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(b"fn foo<A, B, C>(x: A, y: B) -> C {}"),
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
            Root.with_child(Span::new(0, 35), SyntaxId(20)),
            FunctionItem.with_binary_children(Span::new(0, 35), SyntaxId(1), SyntaxId(19)),
            SimplePath.empty(Span::new(3, 6)),
            FunctionExpression.with_binary_children(Span::new(6, 35), SyntaxId(17), SyntaxId(18)),
            FunctionSignature.with_multiple_children(Span::new(6, 32), 7, 3),
            ValueParameters.with_multiple_children(Span::new(15, 27), 3, 4),
            Path.with_child(Span::new(16, 17), SyntaxId(6)),
            PathSegment.empty(Span::new(16, 17)),
            TypePath.with_child(Span::new(19, 20), SyntaxId(8)),
            PathSegment.empty(Span::new(19, 20)),
            Path.with_child(Span::new(22, 23), SyntaxId(10)),
            PathSegment.empty(Span::new(22, 23)),
            TypePath.with_child(Span::new(25, 26), SyntaxId(12)),
            PathSegment.empty(Span::new(25, 26)),
            TypePath.with_child(Span::new(31, 32), SyntaxId(15)),
            PathSegment.empty(Span::new(31, 32)),
            TypeParameters.with_multiple_children(Span::new(6, 15), 0, 3),
            SimplePath.empty(Span::new(7, 8)),
            SimplePath.empty(Span::new(10, 11)),
            SimplePath.empty(Span::new(13, 14)),
            BlockExpression.empty(Span::new(33, 35)),
        ]
    );
}

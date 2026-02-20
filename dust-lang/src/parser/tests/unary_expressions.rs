use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceFileId, Span},
    syntax::{SyntaxId, SyntaxKind::*},
    tests::unary_expressions::{NEGATE, NOT},
};

#[test]
fn negation() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(NEGATE));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 20), SyntaxId(9)),
            FunctionItem.with_binary_children(Span::new(0, 20), SyntaxId(1), SyntaxId(8)),
            SimplePath.empty(Span::new(3, 7)),
            ValueParameters.empty(Span::new(7, 9)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            FunctionExpression.with_binary_children(Span::new(7, 20), SyntaxId(3), SyntaxId(7)),
            BlockExpression.with_child(Span::new(10, 20), SyntaxId(6)),
            NegationExpression.with_child(Span::new(16, 18), SyntaxId(5)),
            PathSegment.empty(Span::new(17, 18)),
            Path.with_child(Span::new(17, 18), SyntaxId(4)),
        ]
    );
}

#[test]
fn logical_not() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(NOT));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 20), SyntaxId(9)),
            FunctionItem.with_binary_children(Span::new(0, 20), SyntaxId(1), SyntaxId(8)),
            SimplePath.empty(Span::new(3, 7)),
            ValueParameters.empty(Span::new(7, 9)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            FunctionExpression.with_binary_children(Span::new(7, 20), SyntaxId(3), SyntaxId(7)),
            BlockExpression.with_child(Span::new(10, 20), SyntaxId(6)),
            NotExpression.with_child(Span::new(16, 18), SyntaxId(5)),
            PathSegment.empty(Span::new(17, 18)),
            Path.with_child(Span::new(17, 18), SyntaxId(4)),
        ]
    );
}

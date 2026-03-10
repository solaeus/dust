use crate::function_wrapper;
use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceFileId, Span},
    syntax::{SyntaxId, node::SyntaxKind::*},
};

#[test]
fn negation() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(function_wrapper!("-x")),
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
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 20), SyntaxId(4), SyntaxId(8)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
            FunctionParameters.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 20), SyntaxId(7)),
            NegationExpression.with_child(Span::new(16, 18), SyntaxId(6)),
            PathExpression.with_child(Span::new(17, 18), SyntaxId(5)),
            PathSegment.empty(Span::new(17, 18)),
        ]
    );
}

#[test]
fn logical_not() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(function_wrapper!("!x")),
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
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 20), SyntaxId(4), SyntaxId(8)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
            FunctionParameters.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 20), SyntaxId(7)),
            NotExpression.with_child(Span::new(16, 18), SyntaxId(6)),
            PathExpression.with_child(Span::new(17, 18), SyntaxId(5)),
            PathSegment.empty(Span::new(17, 18)),
        ]
    );
}

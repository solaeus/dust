use crate::function_wrapper;
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
            Root.with_child(Span::new(0, 20), SyntaxId(8)),
            FunctionItem
                .with_multiple_children(Span::new(0, 20), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 20), SyntaxId(6)),
            NegationExpression.with_child(Span::new(16, 18), SyntaxId(5)),
            PathExpression.with_child(Span::new(17, 18), SyntaxId(4)),
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
            Root.with_child(Span::new(0, 20), SyntaxId(8)),
            FunctionItem
                .with_multiple_children(Span::new(0, 20), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 20), SyntaxId(6)),
            NotExpression.with_child(Span::new(16, 18), SyntaxId(5)),
            PathExpression.with_child(Span::new(17, 18), SyntaxId(4)),
            PathSegment.empty(Span::new(17, 18)),
        ]
    );
}

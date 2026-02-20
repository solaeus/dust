mod binary_assignment_statements;
mod binary_expressions;
mod block_expressions;
mod if_expressions;
mod let_statements;
mod unary_expressions;
mod value_expressions;

use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceFileId, Span},
    syntax::{SyntaxId, SyntaxKind::*, SyntaxPayload},
    tests::{GROUPED_EXPRESSION, REASSIGNMENT_STATEMENT},
};

#[test]
fn reassignment_statement() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(REASSIGNMENT_STATEMENT),
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
            Root.with_child(Span::new(0, 25), SyntaxId(10)),
            FunctionItem.with_binary_children(Span::new(0, 25), SyntaxId(1), SyntaxId(9)),
            SimplePath.empty(Span::new(3, 7)),
            ValueParameters.empty(Span::new(7, 9)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            FunctionExpression.with_binary_children(Span::new(7, 25), SyntaxId(3), SyntaxId(8)),
            BlockExpression.with_child(Span::new(10, 25), SyntaxId(7)),
            PathSegment.empty(Span::new(16, 17)),
            Path.with_child(Span::new(16, 17), SyntaxId(4)),
            ReassignmentStatement.with_binary_children(Span::new(16, 23), SyntaxId(5), SyntaxId(6)),
            IntegerExpression.with_value(Span::new(20, 22), SyntaxPayload::encode_integer(42)),
        ]
    );
}

#[test]
fn grouped_expression() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(GROUPED_EXPRESSION));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 25), SyntaxId(14)),
            FunctionItem.with_binary_children(Span::new(0, 25), SyntaxId(1), SyntaxId(13)),
            SimplePath.empty(Span::new(3, 7)),
            ValueParameters.empty(Span::new(7, 9)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            FunctionExpression.with_binary_children(Span::new(7, 25), SyntaxId(3), SyntaxId(12)),
            BlockExpression.with_child(Span::new(10, 25), SyntaxId(11)),
            GroupedExpression.with_child(Span::new(16, 23), SyntaxId(10)),
            PathSegment.empty(Span::new(17, 18)),
            Path.with_child(Span::new(17, 18), SyntaxId(4)),
            PathExpression.with_child(Span::new(17, 18), SyntaxId(5)),
            AdditionExpression.with_binary_children(Span::new(17, 22), SyntaxId(6), SyntaxId(9)),
            PathSegment.empty(Span::new(21, 22)),
            Path.with_child(Span::new(21, 22), SyntaxId(7)),
            PathExpression.with_child(Span::new(21, 22), SyntaxId(8)),
        ]
    );
}

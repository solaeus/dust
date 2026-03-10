mod binary_assignment_expressions;
mod binary_expressions;
mod block_expression;
mod enum_item;
mod function_item;
mod if_expression;
mod let_statement;
mod mod_item;
mod struct_expression;
mod struct_item;
mod unary_expressions;
mod value_expressions;

use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceFileId, Span},
    syntax::{
        SyntaxId,
        node::{SyntaxKind::*, SyntaxPayload},
    },
};

#[macro_export]
macro_rules! function_wrapper {
    ($content:expr) => {
        concat!("fn main() {\n    ", $content, "\n}").as_bytes()
    };
}

#[test]
fn reassignment_statement() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(function_wrapper!("x = 42;")),
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
            Root.with_child(Span::new(0, 25), SyntaxId(11)),
            FunctionItem.with_binary_children(Span::new(0, 25), SyntaxId(1), SyntaxId(10)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 25), SyntaxId(4), SyntaxId(9)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
            FunctionParameters.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 25), SyntaxId(8)),
            AssignmentExpression.with_binary_children(Span::new(16, 23), SyntaxId(6), SyntaxId(7)),
            PathExpression.with_child(Span::new(16, 17), SyntaxId(5)),
            PathSegment.empty(Span::new(16, 17)),
            IntegerExpression.with_value(Span::new(20, 22), SyntaxPayload::encode_integer(42)),
        ]
    );
}

#[test]
fn grouped_expression() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(function_wrapper!("(x + y)")),
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
            Root.with_child(Span::new(0, 25), SyntaxId(13)),
            FunctionItem.with_binary_children(Span::new(0, 25), SyntaxId(1), SyntaxId(12)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 25), SyntaxId(4), SyntaxId(11)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(3)),
            FunctionParameters.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 25), SyntaxId(10)),
            GroupedExpression.with_child(Span::new(16, 23), SyntaxId(9)),
            AdditionExpression.with_binary_children(Span::new(17, 22), SyntaxId(6), SyntaxId(8)),
            PathExpression.with_child(Span::new(17, 18), SyntaxId(5)),
            PathSegment.empty(Span::new(17, 18)),
            PathExpression.with_child(Span::new(21, 22), SyntaxId(7)),
            PathSegment.empty(Span::new(21, 22)),
        ]
    );
}

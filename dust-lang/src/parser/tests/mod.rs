mod binary_assignment_expressions;
mod binary_expressions;
mod block_expression;
mod const_item;
mod enum_item;
mod field_access_expression;
mod function_item;
mod if_expression;
mod impl_item;
mod let_statement;
mod module_item;
mod path_expression;
mod struct_expression;
mod struct_item;
mod trait_item;
mod type_item;
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
fn assignment_expression() {
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
            Root.with_child(Span::new(0, 25), SyntaxId(10)),
            FunctionItem
                .with_multiple_children(Span::new(0, 25), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 25), SyntaxId(8)),
            ExpressionStatement.with_child(Span::new(16, 23), SyntaxId(7)),
            AssignmentExpression.with_binary_children(Span::new(16, 22), SyntaxId(5), SyntaxId(6)),
            PathExpression.with_child(Span::new(16, 17), SyntaxId(4)),
            PathSegment.empty(Span::new(16, 17)),
            IntegerExpression.empty(Span::new(20, 22)),
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
            Root.with_child(Span::new(0, 25), SyntaxId(11)),
            FunctionItem
                .with_multiple_children(Span::new(0, 25), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionParameters.with_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_child(Span::new(10, 25), SyntaxId(9)),
            GroupedExpression.with_child(Span::new(16, 23), SyntaxId(8)),
            AdditionExpression.with_binary_children(Span::new(17, 22), SyntaxId(5), SyntaxId(7)),
            PathExpression.with_child(Span::new(17, 18), SyntaxId(4)),
            PathSegment.empty(Span::new(17, 18)),
            PathExpression.with_child(Span::new(21, 22), SyntaxId(6)),
            PathSegment.empty(Span::new(21, 22)),
        ]
    );
}

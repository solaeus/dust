mod as_expression;
mod binary_assignment_expressions;
mod binary_expressions;
mod block_expression;
mod call_expression;
mod const_item;
mod enum_item;
mod field_access_expression;
mod function_item;
mod if_expression;
mod impl_item;
mod index_expression;
mod let_statement;
mod loop_expressions;
mod module_item;
mod path_expression;
mod precedence;
mod range_expression;
mod struct_expression;
mod struct_item;
mod trait_item;
mod type_item;
mod type_notation;
mod unary_expressions;
mod unit_struct_item;
mod use_item;
mod value_expressions;

use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{FileId, Span},
    syntax::{
        SyntaxId,
        node::{SyntaxChildren, SyntaxKind::*},
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
        FileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("x = 42;")),
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
            Root.with_single_child(Span::new(0, 25), SyntaxId(11)),
            FnItem.with_children(Span::new(0, 25), SyntaxChildren::new(1, 4)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionSignature.with_children(Span::new(0, 9), SyntaxChildren::new(0, 1)),
            FunctionParameters.with_single_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_single_child(Span::new(10, 25), SyntaxId(9)),
            ExpressionStatement.with_single_child(Span::new(16, 23), SyntaxId(8)),
            AssignmentExpression.with_binary_children(Span::new(16, 22), SyntaxId(6), SyntaxId(7)),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(5)),
            PathSegment.empty(Span::new(16, 17)),
            IntegerExpression.empty(Span::new(20, 22)),
        ]
    );
}

#[test]
fn grouped_expression() {
    let parser = Parser::new(
        FileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("(x + y)")),
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
            Root.with_single_child(Span::new(0, 25), SyntaxId(12)),
            FnItem.with_children(Span::new(0, 25), SyntaxChildren::new(1, 4)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionSignature.with_children(Span::new(0, 9), SyntaxChildren::new(0, 1)),
            FunctionParameters.with_single_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_single_child(Span::new(10, 25), SyntaxId(10)),
            GroupedExpression.with_single_child(Span::new(16, 23), SyntaxId(9)),
            AdditionExpression.with_binary_children(Span::new(17, 22), SyntaxId(6), SyntaxId(8)),
            PathExpression.with_single_child(Span::new(17, 18), SyntaxId(5)),
            PathSegment.empty(Span::new(17, 18)),
            PathExpression.with_single_child(Span::new(21, 22), SyntaxId(7)),
            PathSegment.empty(Span::new(21, 22)),
        ]
    );
}

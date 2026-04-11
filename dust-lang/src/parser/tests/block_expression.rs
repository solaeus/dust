use crate::function_wrapper;
use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{FileId, Span},
    syntax::{
        SyntaxId,
        node::{SyntaxKind::*, SyntaxChildren},
    },
};

#[test]
fn empty() {
    let parser = Parser::new(
        FileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("{}")),
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
            Root.with_single_child(Span::new(0, 20), SyntaxId(7)),
            FunctionItem.with_children(Span::new(0, 20), SyntaxChildren::new(1, 4)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionSignature.with_children(Span::new(0, 9), SyntaxChildren::new(0, 1)),
            FunctionParameters.with_single_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_single_child(Span::new(10, 20), SyntaxId(5)),
            BlockExpression.empty(Span::new(16, 18)),
        ]
    );
}

#[test]
fn item() {
    let parser = Parser::new(
        FileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("{ fn foo() {} }")),
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
            Root.with_single_child(Span::new(0, 33), SyntaxId(13)),
            FunctionItem.with_children(Span::new(0, 33), SyntaxChildren::new(5, 8)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionSignature.with_children(Span::new(0, 9), SyntaxChildren::new(0, 1)),
            FunctionParameters.with_single_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_single_child(Span::new(10, 33), SyntaxId(11)),
            BlockExpression.with_single_child(Span::new(16, 31), SyntaxId(10)),
            FunctionItem.with_children(Span::new(18, 29), SyntaxChildren::new(2, 5)),
            SimplePath.empty(Span::new(21, 24)),
            FunctionSignature.with_children(Span::new(18, 26), SyntaxChildren::new(1, 2)),
            FunctionParameters.with_single_child(Span::new(18, 26), SyntaxId(6)),
            ValueParameters.empty(Span::new(18, 26)),
            BlockExpression.empty(Span::new(27, 29)),
        ]
    );
}

#[test]
fn statement() {
    let parser = Parser::new(
        FileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("{ let x = 42; }")),
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
            Root.with_single_child(Span::new(0, 33), SyntaxId(10)),
            FunctionItem.with_children(Span::new(0, 33), SyntaxChildren::new(1, 4)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionSignature.with_children(Span::new(0, 9), SyntaxChildren::new(0, 1)),
            FunctionParameters.with_single_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_single_child(Span::new(10, 33), SyntaxId(8)),
            BlockExpression.with_single_child(Span::new(16, 31), SyntaxId(7)),
            LetStatement.with_binary_children(Span::new(18, 29), SyntaxId(5), SyntaxId(6)),
            SimplePath.empty(Span::new(22, 23)),
            IntegerExpression.empty(Span::new(26, 28)),
        ]
    );
}

#[test]
fn expression() {
    let parser = Parser::new(
        FileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("{ x + y }")),
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
            Root.with_single_child(Span::new(0, 27), SyntaxId(12)),
            FunctionItem.with_children(Span::new(0, 27), SyntaxChildren::new(1, 4)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionSignature.with_children(Span::new(0, 9), SyntaxChildren::new(0, 1)),
            FunctionParameters.with_single_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_single_child(Span::new(10, 27), SyntaxId(10)),
            BlockExpression.with_single_child(Span::new(16, 25), SyntaxId(9)),
            AdditionExpression.with_binary_children(Span::new(18, 23), SyntaxId(6), SyntaxId(8)),
            PathExpression.with_single_child(Span::new(18, 19), SyntaxId(5)),
            PathSegment.empty(Span::new(18, 19)),
            PathExpression.with_single_child(Span::new(22, 23), SyntaxId(7)),
            PathSegment.empty(Span::new(22, 23)),
        ]
    );
}

#[test]
fn mixed() {
    let parser = Parser::new(
        FileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("{ fn foo() {} let x = 42; x + y }")),
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
            Root.with_single_child(Span::new(0, 51), SyntaxId(21)),
            FunctionItem.with_children(Span::new(0, 51), SyntaxChildren::new(8, 11)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionSignature.with_children(Span::new(0, 9), SyntaxChildren::new(0, 1)),
            FunctionParameters.with_single_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_single_child(Span::new(10, 51), SyntaxId(19)),
            BlockExpression.with_children(Span::new(16, 49), SyntaxChildren::new(5, 8)),
            FunctionItem.with_children(Span::new(18, 29), SyntaxChildren::new(2, 5)),
            SimplePath.empty(Span::new(21, 24)),
            FunctionSignature.with_children(Span::new(18, 26), SyntaxChildren::new(1, 2)),
            FunctionParameters.with_single_child(Span::new(18, 26), SyntaxId(6)),
            ValueParameters.empty(Span::new(18, 26)),
            BlockExpression.empty(Span::new(27, 29)),
            LetStatement.with_binary_children(Span::new(30, 41), SyntaxId(11), SyntaxId(12)),
            SimplePath.empty(Span::new(34, 35)),
            IntegerExpression.empty(Span::new(38, 40)),
            AdditionExpression.with_binary_children(Span::new(42, 47), SyntaxId(15), SyntaxId(17)),
            PathExpression.with_single_child(Span::new(42, 43), SyntaxId(14)),
            PathSegment.empty(Span::new(42, 43)),
            PathExpression.with_single_child(Span::new(46, 47), SyntaxId(16)),
            PathSegment.empty(Span::new(46, 47)),
        ]
    );
}

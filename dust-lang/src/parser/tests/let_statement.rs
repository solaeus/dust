use crate::function_wrapper;
use crate::syntax::node::SyntaxFlag;
use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{FileId, Span},
    syntax::{
        SyntaxId,
        node::{SyntaxChildren, SyntaxKind::*},
    },
};

#[test]
fn let_statement() {
    let parser = Parser::new(
        FileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("let x = 42;")),
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
            Root.with_single_child(Span::new(0, 29), SyntaxId(9)),
            FunctionItem.with_children(Span::new(0, 29), SyntaxChildren::new(1, 4)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionSignature.with_children(Span::new(0, 9), SyntaxChildren::new(0, 1)),
            FunctionParameters.with_single_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_single_child(Span::new(10, 29), SyntaxId(7)),
            LetStatement.with_binary_children(Span::new(16, 27), SyntaxId(5), SyntaxId(6)),
            SimplePath.empty(Span::new(20, 21)),
            IntegerExpression.empty(Span::new(24, 26)),
        ]
    );
}

#[test]
fn let_statement_with_type() {
    let parser = Parser::new(
        FileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("let x: i64 = 42;")),
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
            Root.with_single_child(Span::new(0, 34), SyntaxId(10)),
            FunctionItem.with_children(Span::new(0, 34), SyntaxChildren::new(4, 7)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionSignature.with_children(Span::new(0, 9), SyntaxChildren::new(0, 1)),
            FunctionParameters.with_single_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_single_child(Span::new(10, 34), SyntaxId(8)),
            LetStatement.with_children(Span::new(16, 32), SyntaxChildren::new(1, 4)),
            SimplePath.empty(Span::new(20, 21)),
            IntegerExpression.empty(Span::new(29, 31)),
            I64Type.empty(Span::new(23, 26)),
        ]
    );
}

#[test]
fn let_mut_statement() {
    let parser = Parser::new(
        FileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("let mut x = 42;")),
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
            Root.with_single_child(Span::new(0, 33), SyntaxId(9)),
            FunctionItem.with_children(Span::new(0, 33), SyntaxChildren::new(1, 4)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionSignature.with_children(Span::new(0, 9), SyntaxChildren::new(0, 1)),
            FunctionParameters.with_single_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_single_child(Span::new(10, 33), SyntaxId(7)),
            LetStatement
                .with_binary_children(Span::new(16, 31), SyntaxId(5), SyntaxId(6))
                .with_flag(SyntaxFlag::MUTABLE),
            SimplePath.empty(Span::new(24, 25)),
            IntegerExpression.empty(Span::new(28, 30)),
        ]
    );
}

#[test]
fn let_mut_statement_with_type() {
    let parser = Parser::new(
        FileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("let mut x: i64 = 42;")),
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
            Root.with_single_child(Span::new(0, 38), SyntaxId(10)),
            FunctionItem.with_children(Span::new(0, 38), SyntaxChildren::new(4, 7)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionSignature.with_children(Span::new(0, 9), SyntaxChildren::new(0, 1)),
            FunctionParameters.with_single_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_single_child(Span::new(10, 38), SyntaxId(8)),
            LetStatement
                .with_children(Span::new(16, 36), SyntaxChildren::new(1, 4))
                .with_flag(SyntaxFlag::MUTABLE),
            SimplePath.empty(Span::new(24, 25)),
            IntegerExpression.empty(Span::new(33, 35)),
            I64Type.empty(Span::new(27, 30)),
        ]
    );
}

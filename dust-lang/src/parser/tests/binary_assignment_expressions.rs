use crate::function_wrapper;
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
fn add_assign() {
    let parser = Parser::new(
        FileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("x += 42;")),
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
            Root.with_single_child(Span::new(0, 26), SyntaxId(11)),
            FnItem.with_children(Span::new(0, 26), SyntaxChildren::new(1, 4)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionSignature.with_children(Span::new(0, 9), SyntaxChildren::new(0, 1)),
            FunctionParameters.with_single_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_single_child(Span::new(10, 26), SyntaxId(9)),
            ExpressionStatement.with_single_child(Span::new(16, 24), SyntaxId(8)),
            AdditionAssignmentExpression.with_binary_children(
                Span::new(16, 23),
                SyntaxId(6),
                SyntaxId(7)
            ),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(5)),
            PathSegment.empty(Span::new(16, 17)),
            IntegerExpression.empty(Span::new(21, 23)),
        ]
    );
}

#[test]
fn subtract_assign() {
    let parser = Parser::new(
        FileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("x -= 42;")),
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
            Root.with_single_child(Span::new(0, 26), SyntaxId(11)),
            FnItem.with_children(Span::new(0, 26), SyntaxChildren::new(1, 4)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionSignature.with_children(Span::new(0, 9), SyntaxChildren::new(0, 1)),
            FunctionParameters.with_single_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_single_child(Span::new(10, 26), SyntaxId(9)),
            ExpressionStatement.with_single_child(Span::new(16, 24), SyntaxId(8)),
            SubtractionAssignmentExpression.with_binary_children(
                Span::new(16, 23),
                SyntaxId(6),
                SyntaxId(7)
            ),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(5)),
            PathSegment.empty(Span::new(16, 17)),
            IntegerExpression.empty(Span::new(21, 23)),
        ]
    );
}

#[test]
fn multiply_assign() {
    let parser = Parser::new(
        FileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("x *= 42;")),
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
            Root.with_single_child(Span::new(0, 26), SyntaxId(11)),
            FnItem.with_children(Span::new(0, 26), SyntaxChildren::new(1, 4)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionSignature.with_children(Span::new(0, 9), SyntaxChildren::new(0, 1)),
            FunctionParameters.with_single_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_single_child(Span::new(10, 26), SyntaxId(9)),
            ExpressionStatement.with_single_child(Span::new(16, 24), SyntaxId(8)),
            MultiplicationAssignmentExpression.with_binary_children(
                Span::new(16, 23),
                SyntaxId(6),
                SyntaxId(7)
            ),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(5)),
            PathSegment.empty(Span::new(16, 17)),
            IntegerExpression.empty(Span::new(21, 23)),
        ]
    );
}

#[test]
fn divide_assign() {
    let parser = Parser::new(
        FileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("x /= 42;")),
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
            Root.with_single_child(Span::new(0, 26), SyntaxId(11)),
            FnItem.with_children(Span::new(0, 26), SyntaxChildren::new(1, 4)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionSignature.with_children(Span::new(0, 9), SyntaxChildren::new(0, 1)),
            FunctionParameters.with_single_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_single_child(Span::new(10, 26), SyntaxId(9)),
            ExpressionStatement.with_single_child(Span::new(16, 24), SyntaxId(8)),
            DivisionAssignmentExpression.with_binary_children(
                Span::new(16, 23),
                SyntaxId(6),
                SyntaxId(7)
            ),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(5)),
            PathSegment.empty(Span::new(16, 17)),
            IntegerExpression.empty(Span::new(21, 23)),
        ]
    );
}

#[test]
fn modulo_assign() {
    let parser = Parser::new(
        FileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("x %= 42;")),
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
            Root.with_single_child(Span::new(0, 26), SyntaxId(11)),
            FnItem.with_children(Span::new(0, 26), SyntaxChildren::new(1, 4)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionSignature.with_children(Span::new(0, 9), SyntaxChildren::new(0, 1)),
            FunctionParameters.with_single_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_single_child(Span::new(10, 26), SyntaxId(9)),
            ExpressionStatement.with_single_child(Span::new(16, 24), SyntaxId(8)),
            ModuloAssignmentExpression.with_binary_children(
                Span::new(16, 23),
                SyntaxId(6),
                SyntaxId(7)
            ),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(5)),
            PathSegment.empty(Span::new(16, 17)),
            IntegerExpression.empty(Span::new(21, 23)),
        ]
    );
}

#[test]
fn power_assign() {
    let parser = Parser::new(
        FileId::MAIN,
        Lexer::with_unvalidated_source(function_wrapper!("x ^= 42;")),
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
            Root.with_single_child(Span::new(0, 26), SyntaxId(11)),
            FnItem.with_children(Span::new(0, 26), SyntaxChildren::new(1, 4)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionSignature.with_children(Span::new(0, 9), SyntaxChildren::new(0, 1)),
            FunctionParameters.with_single_child(Span::new(0, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(0, 9)),
            BlockExpression.with_single_child(Span::new(10, 26), SyntaxId(9)),
            ExpressionStatement.with_single_child(Span::new(16, 24), SyntaxId(8)),
            ExponentAssignmentExpression.with_binary_children(
                Span::new(16, 23),
                SyntaxId(6),
                SyntaxId(7)
            ),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(5)),
            PathSegment.empty(Span::new(16, 17)),
            IntegerExpression.empty(Span::new(21, 23)),
        ]
    );
}

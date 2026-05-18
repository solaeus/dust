use crate::{
    function_wrapper,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::Span,
    syntax::{SyntaxId, node::SyntaxKind::*},
};

#[test]
fn addition() {
    let parser = Parser::new_standalone(Lexer::unvalidated(function_wrapper!("x + y")));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 23), SyntaxId(8)),
            FunctionItem.with_binary_children(Span::new(0, 23), SyntaxId(1), SyntaxId(7)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 23), SyntaxId(6)),
            AdditionExpression.with_binary_children(Span::new(16, 21), SyntaxId(3), SyntaxId(5)),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 17)),
            PathExpression.with_single_child(Span::new(20, 21), SyntaxId(4)),
            PathSegment.empty(Span::new(20, 21)),
        ]
    );
}

#[test]
fn subtraction() {
    let parser = Parser::new_standalone(Lexer::unvalidated(function_wrapper!("x - y")));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 23), SyntaxId(8)),
            FunctionItem.with_binary_children(Span::new(0, 23), SyntaxId(1), SyntaxId(7)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 23), SyntaxId(6)),
            SubtractionExpression.with_binary_children(Span::new(16, 21), SyntaxId(3), SyntaxId(5)),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 17)),
            PathExpression.with_single_child(Span::new(20, 21), SyntaxId(4)),
            PathSegment.empty(Span::new(20, 21)),
        ]
    );
}

#[test]
fn multiplication() {
    let parser = Parser::new_standalone(Lexer::unvalidated(function_wrapper!("x * y")));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 23), SyntaxId(8)),
            FunctionItem.with_binary_children(Span::new(0, 23), SyntaxId(1), SyntaxId(7)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 23), SyntaxId(6)),
            MultiplicationExpression.with_binary_children(
                Span::new(16, 21),
                SyntaxId(3),
                SyntaxId(5),
            ),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 17)),
            PathExpression.with_single_child(Span::new(20, 21), SyntaxId(4)),
            PathSegment.empty(Span::new(20, 21)),
        ]
    );
}

#[test]
fn division() {
    let parser = Parser::new_standalone(Lexer::unvalidated(function_wrapper!("x / y")));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 23), SyntaxId(8)),
            FunctionItem.with_binary_children(Span::new(0, 23), SyntaxId(1), SyntaxId(7)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 23), SyntaxId(6)),
            DivisionExpression.with_binary_children(Span::new(16, 21), SyntaxId(3), SyntaxId(5)),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 17)),
            PathExpression.with_single_child(Span::new(20, 21), SyntaxId(4)),
            PathSegment.empty(Span::new(20, 21)),
        ]
    );
}

#[test]
fn modulo() {
    let parser = Parser::new_standalone(Lexer::unvalidated(function_wrapper!("x % y")));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 23), SyntaxId(8)),
            FunctionItem.with_binary_children(Span::new(0, 23), SyntaxId(1), SyntaxId(7)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 23), SyntaxId(6)),
            ModuloExpression.with_binary_children(Span::new(16, 21), SyntaxId(3), SyntaxId(5)),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 17)),
            PathExpression.with_single_child(Span::new(20, 21), SyntaxId(4)),
            PathSegment.empty(Span::new(20, 21)),
        ]
    );
}

#[test]
fn power() {
    let parser = Parser::new_standalone(Lexer::unvalidated(function_wrapper!("x ^ y")));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 23), SyntaxId(8)),
            FunctionItem.with_binary_children(Span::new(0, 23), SyntaxId(1), SyntaxId(7)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 23), SyntaxId(6)),
            ExponentExpression.with_binary_children(Span::new(16, 21), SyntaxId(3), SyntaxId(5)),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 17)),
            PathExpression.with_single_child(Span::new(20, 21), SyntaxId(4)),
            PathSegment.empty(Span::new(20, 21)),
        ]
    );
}

#[test]
fn equal() {
    let parser =
        Parser::new_standalone(Lexer::unvalidated(function_wrapper!("x == y")));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 24), SyntaxId(8)),
            FunctionItem.with_binary_children(Span::new(0, 24), SyntaxId(1), SyntaxId(7)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 24), SyntaxId(6)),
            EqualExpression.with_binary_children(Span::new(16, 22), SyntaxId(3), SyntaxId(5)),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 17)),
            PathExpression.with_single_child(Span::new(21, 22), SyntaxId(4)),
            PathSegment.empty(Span::new(21, 22)),
        ]
    );
}

#[test]
fn not_equal() {
    let parser =
        Parser::new_standalone(Lexer::unvalidated(function_wrapper!("x != y")));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 24), SyntaxId(8)),
            FunctionItem.with_binary_children(Span::new(0, 24), SyntaxId(1), SyntaxId(7)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 24), SyntaxId(6)),
            NotEqualExpression.with_binary_children(Span::new(16, 22), SyntaxId(3), SyntaxId(5)),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 17)),
            PathExpression.with_single_child(Span::new(21, 22), SyntaxId(4)),
            PathSegment.empty(Span::new(21, 22)),
        ]
    );
}

#[test]
fn less_than() {
    let parser = Parser::new_standalone(Lexer::unvalidated(function_wrapper!("x < y")));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 23), SyntaxId(8)),
            FunctionItem.with_binary_children(Span::new(0, 23), SyntaxId(1), SyntaxId(7)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 23), SyntaxId(6)),
            LessThanExpression.with_binary_children(Span::new(16, 21), SyntaxId(3), SyntaxId(5)),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 17)),
            PathExpression.with_single_child(Span::new(20, 21), SyntaxId(4)),
            PathSegment.empty(Span::new(20, 21)),
        ]
    );
}

#[test]
fn less_than_or_equal() {
    let parser =
        Parser::new_standalone(Lexer::unvalidated(function_wrapper!("x <= y")));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 24), SyntaxId(8)),
            FunctionItem.with_binary_children(Span::new(0, 24), SyntaxId(1), SyntaxId(7)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 24), SyntaxId(6)),
            LessThanOrEqualExpression.with_binary_children(
                Span::new(16, 22),
                SyntaxId(3),
                SyntaxId(5),
            ),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 17)),
            PathExpression.with_single_child(Span::new(21, 22), SyntaxId(4)),
            PathSegment.empty(Span::new(21, 22)),
        ]
    );
}

#[test]
fn greater_than() {
    let parser = Parser::new_standalone(Lexer::unvalidated(function_wrapper!("x > y")));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 23), SyntaxId(8)),
            FunctionItem.with_binary_children(Span::new(0, 23), SyntaxId(1), SyntaxId(7)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 23), SyntaxId(6)),
            GreaterThanExpression.with_binary_children(Span::new(16, 21), SyntaxId(3), SyntaxId(5)),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 17)),
            PathExpression.with_single_child(Span::new(20, 21), SyntaxId(4)),
            PathSegment.empty(Span::new(20, 21)),
        ]
    );
}

#[test]
fn greater_than_or_equal() {
    let parser =
        Parser::new_standalone(Lexer::unvalidated(function_wrapper!("x >= y")));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 24), SyntaxId(8)),
            FunctionItem.with_binary_children(Span::new(0, 24), SyntaxId(1), SyntaxId(7)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 24), SyntaxId(6)),
            GreaterThanOrEqualExpression.with_binary_children(
                Span::new(16, 22),
                SyntaxId(3),
                SyntaxId(5),
            ),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 17)),
            PathExpression.with_single_child(Span::new(21, 22), SyntaxId(4)),
            PathSegment.empty(Span::new(21, 22)),
        ]
    );
}

#[test]
fn logical_and() {
    let parser =
        Parser::new_standalone(Lexer::unvalidated(function_wrapper!("x && y")));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 24), SyntaxId(8)),
            FunctionItem.with_binary_children(Span::new(0, 24), SyntaxId(1), SyntaxId(7)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 24), SyntaxId(6)),
            AndExpression.with_binary_children(Span::new(16, 22), SyntaxId(3), SyntaxId(5)),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 17)),
            PathExpression.with_single_child(Span::new(21, 22), SyntaxId(4)),
            PathSegment.empty(Span::new(21, 22)),
        ]
    );
}

#[test]
fn logical_or() {
    let parser =
        Parser::new_standalone(Lexer::unvalidated(function_wrapper!("x || y")));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sort_nodes(),
        [
            Root.with_single_child(Span::new(0, 24), SyntaxId(8)),
            FunctionItem.with_binary_children(Span::new(0, 24), SyntaxId(1), SyntaxId(7)),
            SimplePath.empty(Span::new(3, 7)),
            BlockExpression.with_single_child(Span::new(10, 24), SyntaxId(6)),
            OrExpression.with_binary_children(Span::new(16, 22), SyntaxId(3), SyntaxId(5)),
            PathExpression.with_single_child(Span::new(16, 17), SyntaxId(2)),
            PathSegment.empty(Span::new(16, 17)),
            PathExpression.with_single_child(Span::new(21, 22), SyntaxId(4)),
            PathSegment.empty(Span::new(21, 22)),
        ]
    );
}

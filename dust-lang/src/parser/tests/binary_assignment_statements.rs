use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceFileId, Span},
    syntax::{SyntaxId, SyntaxKind::*, SyntaxPayload},
    tests::binary_assignment_statements::{
        ADD_ASSIGN, DIVIDE_ASSIGN, MODULO_ASSIGN, MULTIPLY_ASSIGN, POWER_ASSIGN, SUBTRACT_ASSIGN,
    },
};

#[test]
fn add_assign() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(ADD_ASSIGN));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 26), SyntaxId(10)),
            FunctionItem.with_binary_children(Span::new(0, 26), SyntaxId(1), SyntaxId(9)),
            SimplePath.empty(Span::new(3, 7)),
            ValueParameters.empty(Span::new(7, 9)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            FunctionExpression.with_binary_children(Span::new(7, 26), SyntaxId(3), SyntaxId(8)),
            BlockExpression.with_child(Span::new(10, 26), SyntaxId(7)),
            PathSegment.empty(Span::new(16, 17)),
            Path.with_child(Span::new(16, 17), SyntaxId(4)),
            AdditionAssignmentStatement.with_binary_children(
                Span::new(16, 24),
                SyntaxId(5),
                SyntaxId(6)
            ),
            IntegerExpression.with_value(Span::new(21, 23), SyntaxPayload::encode_integer(42)),
        ]
    );
}

#[test]
fn subtract_assign() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(SUBTRACT_ASSIGN));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 26), SyntaxId(10)),
            FunctionItem.with_binary_children(Span::new(0, 26), SyntaxId(1), SyntaxId(9)),
            SimplePath.empty(Span::new(3, 7)),
            ValueParameters.empty(Span::new(7, 9)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            FunctionExpression.with_binary_children(Span::new(7, 26), SyntaxId(3), SyntaxId(8)),
            BlockExpression.with_child(Span::new(10, 26), SyntaxId(7)),
            PathSegment.empty(Span::new(16, 17)),
            Path.with_child(Span::new(16, 17), SyntaxId(4)),
            SubtractionAssignmentStatement.with_binary_children(
                Span::new(16, 24),
                SyntaxId(5),
                SyntaxId(6)
            ),
            IntegerExpression.with_value(Span::new(21, 23), SyntaxPayload::encode_integer(42)),
        ]
    );
}

#[test]
fn multiply_assign() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(MULTIPLY_ASSIGN));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 26), SyntaxId(10)),
            FunctionItem.with_binary_children(Span::new(0, 26), SyntaxId(1), SyntaxId(9)),
            SimplePath.empty(Span::new(3, 7)),
            ValueParameters.empty(Span::new(7, 9)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            FunctionExpression.with_binary_children(Span::new(7, 26), SyntaxId(3), SyntaxId(8)),
            BlockExpression.with_child(Span::new(10, 26), SyntaxId(7)),
            PathSegment.empty(Span::new(16, 17)),
            Path.with_child(Span::new(16, 17), SyntaxId(4)),
            MultiplicationAssignmentStatement.with_binary_children(
                Span::new(16, 24),
                SyntaxId(5),
                SyntaxId(6)
            ),
            IntegerExpression.with_value(Span::new(21, 23), SyntaxPayload::encode_integer(42)),
        ]
    );
}

#[test]
fn divide_assign() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(DIVIDE_ASSIGN));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 26), SyntaxId(10)),
            FunctionItem.with_binary_children(Span::new(0, 26), SyntaxId(1), SyntaxId(9)),
            SimplePath.empty(Span::new(3, 7)),
            ValueParameters.empty(Span::new(7, 9)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            FunctionExpression.with_binary_children(Span::new(7, 26), SyntaxId(3), SyntaxId(8)),
            BlockExpression.with_child(Span::new(10, 26), SyntaxId(7)),
            PathSegment.empty(Span::new(16, 17)),
            Path.with_child(Span::new(16, 17), SyntaxId(4)),
            DivisionAssignmentStatement.with_binary_children(
                Span::new(16, 24),
                SyntaxId(5),
                SyntaxId(6)
            ),
            IntegerExpression.with_value(Span::new(21, 23), SyntaxPayload::encode_integer(42)),
        ]
    );
}

#[test]
fn modulo_assign() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(MODULO_ASSIGN));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 26), SyntaxId(10)),
            FunctionItem.with_binary_children(Span::new(0, 26), SyntaxId(1), SyntaxId(9)),
            SimplePath.empty(Span::new(3, 7)),
            ValueParameters.empty(Span::new(7, 9)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            FunctionExpression.with_binary_children(Span::new(7, 26), SyntaxId(3), SyntaxId(8)),
            BlockExpression.with_child(Span::new(10, 26), SyntaxId(7)),
            PathSegment.empty(Span::new(16, 17)),
            Path.with_child(Span::new(16, 17), SyntaxId(4)),
            ModuloAssignmentStatement.with_binary_children(
                Span::new(16, 24),
                SyntaxId(5),
                SyntaxId(6)
            ),
            IntegerExpression.with_value(Span::new(21, 23), SyntaxPayload::encode_integer(42)),
        ]
    );
}

#[test]
fn power_assign() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(POWER_ASSIGN));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 26), SyntaxId(10)),
            FunctionItem.with_binary_children(Span::new(0, 26), SyntaxId(1), SyntaxId(9)),
            SimplePath.empty(Span::new(3, 7)),
            ValueParameters.empty(Span::new(7, 9)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            FunctionExpression.with_binary_children(Span::new(7, 26), SyntaxId(3), SyntaxId(8)),
            BlockExpression.with_child(Span::new(10, 26), SyntaxId(7)),
            PathSegment.empty(Span::new(16, 17)),
            Path.with_child(Span::new(16, 17), SyntaxId(4)),
            ExponentAssignmentStatement.with_binary_children(
                Span::new(16, 24),
                SyntaxId(5),
                SyntaxId(6)
            ),
            IntegerExpression.with_value(Span::new(21, 23), SyntaxPayload::encode_integer(42)),
        ]
    );
}

use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceFileId, Span},
    syntax::{SyntaxId, SyntaxKind::*, SyntaxPayload},
    tests::let_statement::{WITH_TYPE, WITH_TYPE_MUT, WITHOUT_TYPE, WITHOUT_TYPE_MUT},
};

#[test]
fn let_statement() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(WITHOUT_TYPE));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 29), SyntaxId(10)),
            FunctionItem.with_binary_children(Span::new(0, 29), SyntaxId(1), SyntaxId(9)),
            SimplePath.empty(Span::new(3, 7)),
            ValueParameters.empty(Span::new(7, 9)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            FunctionExpression.with_binary_children(Span::new(7, 29), SyntaxId(3), SyntaxId(8)),
            BlockExpression.with_child(Span::new(10, 29), SyntaxId(7)),
            LetStatement.with_binary_children(Span::new(16, 27), SyntaxId(5), SyntaxId(6)),
            PathSegment.empty(Span::new(20, 21)),
            Path.with_child(Span::new(20, 21), SyntaxId(4)),
            IntegerExpression.with_value(Span::new(24, 26), SyntaxPayload::encode_integer(42)),
        ]
    );
}

#[test]
fn let_statement_with_type() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(WITH_TYPE));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 34), SyntaxId(11)),
            FunctionItem.with_binary_children(Span::new(0, 34), SyntaxId(1), SyntaxId(10)),
            SimplePath.empty(Span::new(3, 7)),
            ValueParameters.empty(Span::new(7, 9)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            FunctionExpression.with_binary_children(Span::new(7, 34), SyntaxId(3), SyntaxId(9)),
            BlockExpression.with_child(Span::new(10, 34), SyntaxId(8)),
            LetStatement.with_multiple_children(Span::new(16, 32), 0, 3),
            PathSegment.empty(Span::new(20, 21)),
            Path.with_child(Span::new(20, 21), SyntaxId(4)),
            IntegerType.empty(Span::new(23, 26)),
            IntegerExpression.with_value(Span::new(29, 31), SyntaxPayload::encode_integer(42)),
        ]
    );
}

#[test]
fn let_mut_statement() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(WITHOUT_TYPE_MUT));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 33), SyntaxId(10)),
            FunctionItem.with_binary_children(Span::new(0, 33), SyntaxId(1), SyntaxId(9)),
            SimplePath.empty(Span::new(3, 7)),
            ValueParameters.empty(Span::new(7, 9)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            FunctionExpression.with_binary_children(Span::new(7, 33), SyntaxId(3), SyntaxId(8)),
            BlockExpression.with_child(Span::new(10, 33), SyntaxId(7)),
            LetMutStatement.with_binary_children(Span::new(16, 31), SyntaxId(5), SyntaxId(6)),
            PathSegment.empty(Span::new(24, 25)),
            Path.with_child(Span::new(24, 25), SyntaxId(4)),
            IntegerExpression.with_value(Span::new(28, 30), SyntaxPayload::encode_integer(42)),
        ]
    );
}

#[test]
fn let_mut_statement_with_type() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(WITH_TYPE_MUT));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 38), SyntaxId(11)),
            FunctionItem.with_binary_children(Span::new(0, 38), SyntaxId(1), SyntaxId(10)),
            SimplePath.empty(Span::new(3, 7)),
            ValueParameters.empty(Span::new(7, 9)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            FunctionExpression.with_binary_children(Span::new(7, 38), SyntaxId(3), SyntaxId(9)),
            BlockExpression.with_child(Span::new(10, 38), SyntaxId(8)),
            LetMutStatement.with_multiple_children(Span::new(16, 36), 0, 3),
            PathSegment.empty(Span::new(24, 25)),
            Path.with_child(Span::new(24, 25), SyntaxId(4)),
            IntegerType.empty(Span::new(27, 30)),
            IntegerExpression.with_value(Span::new(33, 35), SyntaxPayload::encode_integer(42)),
        ]
    );
}

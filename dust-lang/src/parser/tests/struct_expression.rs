use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceFileId, Span},
    syntax::{SyntaxId, SyntaxKind::*, SyntaxPayload},
    tests::struct_expression::{EMPTY_FIELDS, FIELDS},
};

#[test]
fn empty_fields() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(EMPTY_FIELDS));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 24), SyntaxId(10)),
            FunctionItem.with_binary_children(Span::new(0, 24), SyntaxId(1), SyntaxId(9)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 24), SyntaxId(3), SyntaxId(8)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 24), SyntaxId(7)),
            StructExpression.with_binary_children(Span::new(16, 22), SyntaxId(5), SyntaxId(6)),
            Path.with_child(Span::new(16, 19), SyntaxId(4)),
            PathSegment.empty(Span::new(16, 19)),
            StructFields.empty(Span::new(21, 22)),
        ]
    );
}

#[test]
fn fields() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(FIELDS));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 39), SyntaxId(16)),
            FunctionItem.with_binary_children(Span::new(0, 39), SyntaxId(1), SyntaxId(15)),
            SimplePath.empty(Span::new(3, 7)),
            FunctionExpression.with_binary_children(Span::new(7, 39), SyntaxId(3), SyntaxId(14)),
            FunctionSignature.with_child(Span::new(7, 9), SyntaxId(2)),
            ValueParameters.empty(Span::new(7, 9)),
            BlockExpression.with_child(Span::new(10, 39), SyntaxId(13)),
            StructExpression.with_binary_children(Span::new(16, 37), SyntaxId(5), SyntaxId(12)),
            Path.with_child(Span::new(16, 19), SyntaxId(4)),
            PathSegment.empty(Span::new(16, 19)),
            StructFields.with_binary_children(Span::new(22, 37), SyntaxId(8), SyntaxId(11)),
            StructField.with_binary_children(Span::new(22, 28), SyntaxId(6), SyntaxId(7)),
            SimplePath.empty(Span::new(22, 23)),
            IntegerExpression.with_value(Span::new(25, 27), SyntaxPayload::encode_integer(42)),
            StructField.with_binary_children(Span::new(22, 35), SyntaxId(9), SyntaxId(10)),
            SimplePath.empty(Span::new(29, 30)),
            IntegerExpression.with_value(Span::new(32, 35), SyntaxPayload::encode_integer(666)),
        ]
    );
}

use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceFileId, Span},
    syntax::{SyntaxId, SyntaxKind::*},
    tests::struct_item::{EMPYT, FIELDS, TUPLE, TYPE_PARAMETERS},
};

#[test]
fn empty() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(EMPYT));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 13), SyntaxId(3)),
            StructItem.with_binary_children(Span::new(0, 13), SyntaxId(1), SyntaxId(2)),
            SimplePath.empty(Span::new(7, 10)),
            StructFields.empty(Span::new(11, 13)),
        ]
    );
}

#[test]
fn tuple() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(TUPLE));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 21), SyntaxId(5)),
            StructItem.with_binary_children(Span::new(0, 21), SyntaxId(1), SyntaxId(4)),
            SimplePath.empty(Span::new(7, 10)),
            TupleFields.with_binary_children(Span::new(10, 20), SyntaxId(2), SyntaxId(3)),
            IntegerType.empty(Span::new(11, 14)),
            IntegerType.empty(Span::new(16, 19)),
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
            Root.with_child(Span::new(0, 29), SyntaxId(9)),
            StructItem.with_binary_children(Span::new(0, 29), SyntaxId(1), SyntaxId(8)),
            SimplePath.empty(Span::new(7, 10)),
            StructFields.with_binary_children(Span::new(11, 29), SyntaxId(4), SyntaxId(7)),
            StructField.with_binary_children(Span::new(11, 20), SyntaxId(2), SyntaxId(3)),
            SimplePath.empty(Span::new(13, 14)),
            IntegerType.empty(Span::new(16, 19)),
            StructField.with_binary_children(Span::new(11, 27), SyntaxId(5), SyntaxId(6)),
            SimplePath.empty(Span::new(21, 22)),
            IntegerType.empty(Span::new(24, 27)),
        ]
    );
}

#[test]
fn type_parameters() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(TYPE_PARAMETERS));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 22), SyntaxId(7)),
            StructItem.with_multiple_children(Span::new(0, 22), 3, 3),
            SimplePath.empty(Span::new(7, 10)),
            TypeParameters.with_multiple_children(Span::new(10, 19), 0, 3),
            SimplePath.empty(Span::new(11, 12)),
            SimplePath.empty(Span::new(14, 15)),
            SimplePath.empty(Span::new(17, 18)),
            StructFields.empty(Span::new(20, 22)),
        ]
    );
}

use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceFileId, Span},
    syntax::{SyntaxId, SyntaxKind::*},
    tests::enum_item::{EMPTY_VARIANT, FIELDS_VARIANT, MIXED_VARIANTS, TUPLE_VARIANT, TYPE_PARAMETERS},
};

#[test]
fn empty_variant() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(EMPTY_VARIANT));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 16), SyntaxId(5)),
            EnumItem.with_binary_children(Span::new(0, 16), SyntaxId(1), SyntaxId(4)),
            SimplePath.empty(Span::new(5, 8)),
            EnumVariants.with_child(Span::new(0, 16), SyntaxId(3)),
            EnumVariant.with_child(Span::new(11, 16), SyntaxId(2)),
            SimplePath.empty(Span::new(11, 14)),
        ]
    );
}

#[test]
fn tuple_variant() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(TUPLE_VARIANT));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 26), SyntaxId(9)),
            EnumItem.with_binary_children(Span::new(0, 26), SyntaxId(1), SyntaxId(8)),
            SimplePath.empty(Span::new(5, 8)),
            EnumVariants.with_child(Span::new(0, 26), SyntaxId(7)),
            EnumVariant.with_binary_children(Span::new(11, 24), SyntaxId(2), SyntaxId(6)),
            SimplePath.empty(Span::new(11, 14)),
            EnumVariant.with_child(Span::new(11, 24), SyntaxId(5)),
            TupleFields.with_binary_children(Span::new(14, 24), SyntaxId(3), SyntaxId(4)),
            IntegerType.empty(Span::new(15, 18)),
            IntegerType.empty(Span::new(20, 23)),
        ]
    );
}

#[test]
fn fields_variant() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(FIELDS_VARIANT));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 35), SyntaxId(13)),
            EnumItem.with_binary_children(Span::new(0, 35), SyntaxId(1), SyntaxId(12)),
            SimplePath.empty(Span::new(5, 8)),
            EnumVariants.with_child(Span::new(0, 35), SyntaxId(11)),
            EnumVariant.with_binary_children(Span::new(11, 33), SyntaxId(2), SyntaxId(10)),
            SimplePath.empty(Span::new(11, 14)),
            EnumVariant.with_child(Span::new(11, 33), SyntaxId(9)),
            StructFields.with_binary_children(Span::new(15, 33), SyntaxId(5), SyntaxId(8)),
            StructField.with_binary_children(Span::new(15, 24), SyntaxId(3), SyntaxId(4)),
            SimplePath.empty(Span::new(17, 18)),
            IntegerType.empty(Span::new(20, 23)),
            StructField.with_binary_children(Span::new(15, 31), SyntaxId(6), SyntaxId(7)),
            SimplePath.empty(Span::new(25, 26)),
            IntegerType.empty(Span::new(28, 31)),
        ]
    );
}

#[test]
fn mixed_variants() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(MIXED_VARIANTS));
    let ParseResult {
        syntax_tree,
        errors,
        ..
    } = parser.parse();

    assert!(errors.is_empty(), "{errors:#?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        [
            Root.with_child(Span::new(0, 42), SyntaxId(17)),
            EnumItem.with_binary_children(Span::new(0, 42), SyntaxId(1), SyntaxId(16)),
            SimplePath.empty(Span::new(5, 8)),
            EnumVariants.with_multiple_children(Span::new(0, 42), 0, 3),
            EnumVariant.with_child(Span::new(11, 15), SyntaxId(2)),
            SimplePath.empty(Span::new(11, 14)),
            EnumVariant.with_binary_children(Span::new(16, 24), SyntaxId(4), SyntaxId(7)),
            SimplePath.empty(Span::new(16, 19)),
            EnumVariant.with_child(Span::new(16, 24), SyntaxId(6)),
            TupleFields.with_child(Span::new(19, 24), SyntaxId(5)),
            IntegerType.empty(Span::new(20, 23)),
            EnumVariant.with_binary_children(Span::new(26, 40), SyntaxId(9), SyntaxId(14)),
            SimplePath.empty(Span::new(26, 29)),
            EnumVariant.with_child(Span::new(26, 40), SyntaxId(13)),
            StructFields.with_child(Span::new(30, 40), SyntaxId(12)),
            StructField.with_binary_children(Span::new(30, 38), SyntaxId(10), SyntaxId(11)),
            SimplePath.empty(Span::new(32, 33)),
            IntegerType.empty(Span::new(35, 38)),
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
            Root.with_child(Span::new(0, 25), SyntaxId(9)),
            EnumItem.with_multiple_children(Span::new(0, 25), 3, 3),
            SimplePath.empty(Span::new(5, 8)),
            EnumVariants.with_child(Span::new(0, 25), SyntaxId(7)),
            EnumVariant.with_child(Span::new(20, 25), SyntaxId(6)),
            SimplePath.empty(Span::new(20, 23)),
            TypeParameters.with_multiple_children(Span::new(8, 17), 0, 3),
            SimplePath.empty(Span::new(9, 10)),
            SimplePath.empty(Span::new(12, 13)),
            SimplePath.empty(Span::new(15, 16)),
        ]
    );
}

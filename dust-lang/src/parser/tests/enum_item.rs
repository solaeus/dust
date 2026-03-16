use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceFileId, Span},
    syntax::{
        SyntaxId,
        node::{SyntaxKind::*, SyntaxPayload},
    },
};

#[test]
fn empty_variant() {
    let parser = Parser::new(SourceFileId::MAIN, Lexer::from_bytes(b"enum Foo { Bar }"));
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
            EnumUnitVariant.with_child(Span::new(11, 16), SyntaxId(2)),
            SimplePath.empty(Span::new(11, 14)),
        ]
    );
}

#[test]
fn tuple_variant() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(b"enum Foo { Bar(i64, i64) }"),
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
            Root.with_child(Span::new(0, 26), SyntaxId(9)),
            EnumItem.with_binary_children(Span::new(0, 26), SyntaxId(1), SyntaxId(8)),
            SimplePath.empty(Span::new(5, 8)),
            EnumVariants.with_child(Span::new(0, 26), SyntaxId(7)),
            EnumUnitVariant.with_binary_children(Span::new(11, 24), SyntaxId(2), SyntaxId(6)),
            SimplePath.empty(Span::new(11, 14)),
            EnumUnitVariant.with_child(Span::new(11, 24), SyntaxId(5)),
            StructItemTupleFields.with_binary_children(Span::new(14, 24), SyntaxId(3), SyntaxId(4)),
            I64Type.empty(Span::new(15, 18)),
            I64Type.empty(Span::new(20, 23)),
        ]
    );
}

#[test]
fn fields_variant() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(b"enum Foo { Bar { x: i64, y: i64 } }"),
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
            Root.with_child(Span::new(0, 35), SyntaxId(11)),
            EnumItem.with_binary_children(Span::new(0, 35), SyntaxId(1), SyntaxId(10)),
            SimplePath.empty(Span::new(5, 8)),
            EnumVariants.with_child(Span::new(0, 35), SyntaxId(9)),
            EnumUnitVariant.with_binary_children(Span::new(11, 33), SyntaxId(2), SyntaxId(8)),
            SimplePath.empty(Span::new(11, 14)),
            EnumUnitVariant.with_child(Span::new(11, 33), SyntaxId(7)),
            StructItemStructFields
                .with_multiple_children(Span::new(15, 33), SyntaxPayload::child_indices(0, 4)),
            SimplePath.empty(Span::new(17, 18)),
            I64Type.empty(Span::new(20, 23)),
            SimplePath.empty(Span::new(25, 26)),
            I64Type.empty(Span::new(28, 31)),
        ]
    );
}

#[test]
fn mixed_variants() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(b"enum Foo { Bar, Baz(i64), Qux { x: i64 } }"),
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
            Root.with_child(Span::new(0, 42), SyntaxId(16)),
            EnumItem.with_binary_children(Span::new(0, 42), SyntaxId(1), SyntaxId(15)),
            SimplePath.empty(Span::new(5, 8)),
            EnumVariants
                .with_multiple_children(Span::new(0, 42), SyntaxPayload::child_indices(0, 3)),
            EnumUnitVariant.with_child(Span::new(11, 15), SyntaxId(2)),
            SimplePath.empty(Span::new(11, 14)),
            EnumUnitVariant.with_binary_children(Span::new(16, 24), SyntaxId(4), SyntaxId(7)),
            SimplePath.empty(Span::new(16, 19)),
            EnumUnitVariant.with_child(Span::new(16, 24), SyntaxId(6)),
            StructItemTupleFields.with_child(Span::new(19, 24), SyntaxId(5)),
            I64Type.empty(Span::new(20, 23)),
            EnumUnitVariant.with_binary_children(Span::new(26, 40), SyntaxId(9), SyntaxId(13)),
            SimplePath.empty(Span::new(26, 29)),
            EnumUnitVariant.with_child(Span::new(26, 40), SyntaxId(12)),
            StructItemStructFields.with_binary_children(
                Span::new(30, 40),
                SyntaxId(10),
                SyntaxId(11)
            ),
            SimplePath.empty(Span::new(32, 33)),
            I64Type.empty(Span::new(35, 38)),
        ]
    );
}

#[test]
fn type_parameters() {
    let parser = Parser::new(
        SourceFileId::MAIN,
        Lexer::from_bytes(b"enum Foo<A, B, C> { Bar }"),
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
            Root.with_child(Span::new(0, 25), SyntaxId(9)),
            EnumItem.with_multiple_children(Span::new(0, 25), SyntaxPayload::child_indices(3, 3)),
            SimplePath.empty(Span::new(5, 8)),
            EnumVariants.with_child(Span::new(0, 25), SyntaxId(7)),
            EnumUnitVariant.with_child(Span::new(20, 25), SyntaxId(6)),
            SimplePath.empty(Span::new(20, 23)),
            TypeParameters
                .with_multiple_children(Span::new(8, 17), SyntaxPayload::child_indices(0, 3)),
            SimplePath.empty(Span::new(9, 10)),
            SimplePath.empty(Span::new(12, 13)),
            SimplePath.empty(Span::new(15, 16)),
        ]
    );
}

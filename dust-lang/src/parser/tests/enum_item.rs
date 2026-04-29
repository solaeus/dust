use crate::{
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::{SourceCodeId, Span},
    syntax::{
        SyntaxId,
        node::{SyntaxChildren, SyntaxFlags, SyntaxKind::*},
    },
};

#[test]
fn empty_variant() {
    let parser = Parser::new(
        SourceCodeId::MAIN,
        Lexer::with_unvalidated_source(b"enum Foo { Bar }"),
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
            Root.with_single_child(Span::new(0, 16), SyntaxId(4)),
            EnumItem.with_binary_children(Span::new(0, 16), SyntaxId(1), SyntaxId(3)),
            SimplePath.empty(Span::new(5, 8)),
            EnumVariants.with_single_child(Span::new(9, 16), SyntaxId(2)),
            EnumUnitVariant.empty(Span::new(11, 14)),
        ]
    );
}

#[test]
fn tuple_variant() {
    let parser = Parser::new(
        SourceCodeId::MAIN,
        Lexer::with_unvalidated_source(b"enum Foo { Bar(i64, i64) }"),
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
            Root.with_single_child(Span::new(0, 26), SyntaxId(8)),
            EnumItem.with_binary_children(Span::new(0, 26), SyntaxId(1), SyntaxId(7)),
            SimplePath.empty(Span::new(5, 8)),
            EnumVariants.with_single_child(Span::new(9, 26), SyntaxId(6)),
            EnumTupleFieldsVariant.with_binary_children(
                Span::new(11, 24),
                SyntaxId(2),
                SyntaxId(5),
            ),
            SimplePath.empty(Span::new(11, 14)),
            TupleFields.with_binary_children(Span::new(14, 24), SyntaxId(3), SyntaxId(4)),
            I64Type.empty(Span::new(15, 18)),
            I64Type.empty(Span::new(20, 23)),
        ]
    );
}

#[test]
fn fields_variant() {
    let parser = Parser::new(
        SourceCodeId::MAIN,
        Lexer::with_unvalidated_source(b"enum Foo { Bar { x: i64, y: i64 } }"),
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
            Root.with_single_child(Span::new(0, 35), SyntaxId(10)),
            EnumItem.with_binary_children(Span::new(0, 35), SyntaxId(1), SyntaxId(9)),
            SimplePath.empty(Span::new(5, 8)),
            EnumVariants.with_single_child(Span::new(9, 35), SyntaxId(8)),
            EnumNamedFieldsVariant.with_binary_children(
                Span::new(11, 33),
                SyntaxId(2),
                SyntaxId(7),
            ),
            SimplePath.empty(Span::new(11, 14)),
            NamedFields.with_children(Span::new(15, 33), SyntaxChildren::new(0, 4)),
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
        SourceCodeId::MAIN,
        Lexer::with_unvalidated_source(b"enum Foo { Bar, Baz(i64), Qux { x: i64 } }"),
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
            Root.with_single_child(Span::new(0, 42), SyntaxId(13)),
            EnumItem.with_binary_children(Span::new(0, 42), SyntaxId(1), SyntaxId(12)),
            SimplePath.empty(Span::new(5, 8)),
            EnumVariants.with_children(Span::new(9, 42), SyntaxChildren::new(0, 3)),
            EnumUnitVariant.empty(Span::new(11, 14)),
            EnumTupleFieldsVariant.with_binary_children(
                Span::new(16, 24),
                SyntaxId(3),
                SyntaxId(5),
            ),
            SimplePath.empty(Span::new(16, 19)),
            TupleFields.with_single_child(Span::new(19, 24), SyntaxId(4)),
            I64Type.empty(Span::new(20, 23)),
            EnumNamedFieldsVariant.with_binary_children(
                Span::new(26, 40),
                SyntaxId(7),
                SyntaxId(10),
            ),
            SimplePath.empty(Span::new(26, 29)),
            NamedFields.with_binary_children(Span::new(30, 40), SyntaxId(8), SyntaxId(9)),
            SimplePath.empty(Span::new(32, 33)),
            I64Type.empty(Span::new(35, 38)),
        ]
    );
}

#[test]
fn type_parameters() {
    let parser = Parser::new(
        SourceCodeId::MAIN,
        Lexer::with_unvalidated_source(b"enum Foo<A, B, C> { Bar }"),
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
            Root.with_single_child(Span::new(0, 25), SyntaxId(11)),
            EnumItem
                .with_children(Span::new(0, 25), SyntaxChildren::new(3, 6))
                .with_flags(SyntaxFlags::TYPE_PARAMETERS),
            SimplePath.empty(Span::new(5, 8)),
            TypeParameters.with_children(Span::new(8, 17), SyntaxChildren::new(0, 3)),
            TypeParameter.with_single_child(Span::new(9, 10), SyntaxId(2)),
            SimplePath.empty(Span::new(9, 10)),
            TypeParameter.with_single_child(Span::new(12, 13), SyntaxId(4)),
            SimplePath.empty(Span::new(12, 13)),
            TypeParameter.with_single_child(Span::new(15, 16), SyntaxId(6)),
            SimplePath.empty(Span::new(15, 16)),
            EnumVariants.with_single_child(Span::new(18, 25), SyntaxId(9)),
            EnumUnitVariant.empty(Span::new(20, 23)),
        ]
    );
}

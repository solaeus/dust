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
fn empty() {
    let parser = Parser::new(
        SourceCodeId::MAIN,
        Lexer::with_unvalidated_source(b"trait Foo {}"),
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
            Root.with_single_child(Span::new(0, 12), SyntaxId(3)),
            TraitItem.with_binary_children(Span::new(0, 12), SyntaxId(1), SyntaxId(2)),
            SimplePath.empty(Span::new(6, 9)),
            TraitBody.empty(Span::new(10, 12)),
        ]
    );
}

#[test]
fn with_supertraits() {
    let parser = Parser::new(
        SourceCodeId::MAIN,
        Lexer::with_unvalidated_source(b"trait Foo: Bar + Baz {}"),
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
            Root.with_single_child(Span::new(0, 23), SyntaxId(8)),
            TraitItem
                .with_children(Span::new(0, 23), SyntaxChildren::new(0, 3))
                .with_flags(SyntaxFlags::SUPERTRAITS),
            SimplePath.empty(Span::new(6, 9)),
            TraitBody.empty(Span::new(21, 23)),
            TraitBounds.with_binary_children(Span::new(11, 20), SyntaxId(3), SyntaxId(5)),
            Path.with_single_child(Span::new(11, 14), SyntaxId(2)),
            PathSegment.empty(Span::new(11, 14)),
            Path.with_single_child(Span::new(17, 20), SyntaxId(4)),
            PathSegment.empty(Span::new(17, 20)),
        ]
    );
}

#[test]
fn with_type_parameters_and_supertraits() {
    let parser = Parser::new(
        SourceCodeId::MAIN,
        Lexer::with_unvalidated_source(b"trait Foo<T>: Bar + Baz {}"),
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
            TraitItem
                .with_children(Span::new(0, 26), SyntaxChildren::new(0, 4))
                .with_flags(SyntaxFlags::TYPE_PARAMETERS.and(SyntaxFlags::SUPERTRAITS)),
            SimplePath.empty(Span::new(6, 9)),
            TraitBody.empty(Span::new(24, 26)),
            TypeParameters.with_single_child(Span::new(9, 12), SyntaxId(3)),
            TypeParameter.with_single_child(Span::new(10, 11), SyntaxId(2)),
            SimplePath.empty(Span::new(10, 11)),
            TraitBounds.with_binary_children(Span::new(14, 23), SyntaxId(6), SyntaxId(8)),
            Path.with_single_child(Span::new(14, 17), SyntaxId(5)),
            PathSegment.empty(Span::new(14, 17)),
            Path.with_single_child(Span::new(20, 23), SyntaxId(7)),
            PathSegment.empty(Span::new(20, 23)),
        ]
    );
}

#[test]
fn with_where_clause() {
    let parser = Parser::new(
        SourceCodeId::MAIN,
        Lexer::with_unvalidated_source(b"trait Foo<T> where T: Bar {}"),
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
            Root.with_single_child(Span::new(0, 28), SyntaxId(13)),
            TraitItem
                .with_children(Span::new(0, 28), SyntaxChildren::new(0, 4))
                .with_flags(SyntaxFlags::TYPE_PARAMETERS.and(SyntaxFlags::WHERE_CLAUSE)),
            SimplePath.empty(Span::new(6, 9)),
            TraitBody.empty(Span::new(26, 28)),
            TypeParameters.with_single_child(Span::new(9, 12), SyntaxId(3)),
            TypeParameter.with_single_child(Span::new(10, 11), SyntaxId(2)),
            SimplePath.empty(Span::new(10, 11)),
            WhereClause.with_single_child(Span::new(13, 25), SyntaxId(10)),
            WherePredicate.with_binary_children(Span::new(19, 25), SyntaxId(6), SyntaxId(9)),
            TypePath.with_single_child(Span::new(19, 20), SyntaxId(5)),
            PathSegment.empty(Span::new(19, 20)),
            TraitBounds.with_single_child(Span::new(22, 25), SyntaxId(8)),
            Path.with_single_child(Span::new(22, 25), SyntaxId(7)),
            PathSegment.empty(Span::new(22, 25)),
        ]
    );
}

#[test]
fn with_const_member() {
    let parser = Parser::new(
        SourceCodeId::MAIN,
        Lexer::with_unvalidated_source(b"trait Foo { const X: i64; }"),
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
            Root.with_single_child(Span::new(0, 27), SyntaxId(6)),
            TraitItem.with_binary_children(Span::new(0, 27), SyntaxId(1), SyntaxId(5)),
            SimplePath.empty(Span::new(6, 9)),
            TraitBody.with_single_child(Span::new(10, 27), SyntaxId(4)),
            ConstItem.with_binary_children(Span::new(12, 25), SyntaxId(2), SyntaxId(3)),
            SimplePath.empty(Span::new(18, 19)),
            I64Type.empty(Span::new(21, 24)),
        ]
    );
}

#[test]
fn with_const_member_default() {
    let parser = Parser::new(
        SourceCodeId::MAIN,
        Lexer::with_unvalidated_source(b"trait Foo { const X: i64 = 42; }"),
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
            Root.with_single_child(Span::new(0, 32), SyntaxId(7)),
            TraitItem.with_binary_children(Span::new(0, 32), SyntaxId(1), SyntaxId(6)),
            SimplePath.empty(Span::new(6, 9)),
            TraitBody.with_single_child(Span::new(10, 32), SyntaxId(5)),
            ConstItem.with_children(Span::new(12, 30), SyntaxChildren::new(0, 3)),
            SimplePath.empty(Span::new(18, 19)),
            I64Type.empty(Span::new(21, 24)),
            IntegerExpression.empty(Span::new(27, 29)),
        ]
    );
}

#[test]
fn with_method_signature() {
    let parser = Parser::new(
        SourceCodeId::MAIN,
        Lexer::with_unvalidated_source(b"trait Foo { fn bar(self); }"),
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
            Root.with_single_child(Span::new(0, 27), SyntaxId(6)),
            TraitItem.with_binary_children(Span::new(0, 27), SyntaxId(1), SyntaxId(5)),
            SimplePath.empty(Span::new(6, 9)),
            TraitBody.with_single_child(Span::new(10, 27), SyntaxId(4)),
            FnItem
                .with_binary_children(Span::new(12, 25), SyntaxId(2), SyntaxId(3))
                .with_flags(SyntaxFlags::VALUE_PARAMETERS),
            SimplePath.empty(Span::new(15, 18)),
            ValueParameters
                .empty(Span::new(18, 24))
                .with_flags(SyntaxFlags::SELF_VALUE),
        ]
    );
}

#[test]
fn with_default_method() {
    let parser = Parser::new(
        SourceCodeId::MAIN,
        Lexer::with_unvalidated_source(b"trait Foo { fn bar(self) {} }"),
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
            Root.with_single_child(Span::new(0, 29), SyntaxId(7)),
            TraitItem.with_binary_children(Span::new(0, 29), SyntaxId(1), SyntaxId(6)),
            SimplePath.empty(Span::new(6, 9)),
            TraitBody.with_single_child(Span::new(10, 29), SyntaxId(5)),
            FnItem
                .with_children(Span::new(12, 27), SyntaxChildren::new(0, 3))
                .with_flags(SyntaxFlags::VALUE_PARAMETERS),
            SimplePath.empty(Span::new(15, 18)),
            ValueParameters
                .empty(Span::new(18, 24))
                .with_flags(SyntaxFlags::SELF_VALUE),
            BlockExpression.empty(Span::new(25, 27)),
        ]
    );
}

#[test]
fn with_method_signature_and_return_type() {
    let parser = Parser::new(
        SourceCodeId::MAIN,
        Lexer::with_unvalidated_source(b"trait Foo { fn bar(self) -> i64; }"),
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
            Root.with_single_child(Span::new(0, 34), SyntaxId(7)),
            TraitItem.with_binary_children(Span::new(0, 34), SyntaxId(1), SyntaxId(6)),
            SimplePath.empty(Span::new(6, 9)),
            TraitBody.with_single_child(Span::new(10, 34), SyntaxId(5)),
            FnItem
                .with_children(Span::new(12, 32), SyntaxChildren::new(0, 3))
                .with_flags(SyntaxFlags::VALUE_PARAMETERS.and(SyntaxFlags::RETURN_TYPE)),
            SimplePath.empty(Span::new(15, 18)),
            ValueParameters
                .empty(Span::new(18, 24))
                .with_flags(SyntaxFlags::SELF_VALUE),
            I64Type.empty(Span::new(28, 31)),
        ]
    );
}

#[test]
fn with_type_member() {
    let parser = Parser::new(
        SourceCodeId::MAIN,
        Lexer::with_unvalidated_source(b"trait Foo { type Bar; }"),
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
            Root.with_single_child(Span::new(0, 23), SyntaxId(5)),
            TraitItem.with_binary_children(Span::new(0, 23), SyntaxId(1), SyntaxId(4)),
            SimplePath.empty(Span::new(6, 9)),
            TraitBody.with_single_child(Span::new(10, 23), SyntaxId(3)),
            TypeItem.with_single_child(Span::new(12, 21), SyntaxId(2)),
            SimplePath.empty(Span::new(17, 20)),
        ]
    );
}

#[test]
fn with_type_member_default() {
    let parser = Parser::new(
        SourceCodeId::MAIN,
        Lexer::with_unvalidated_source(b"trait Foo { type Bar = i64; }"),
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
            Root.with_single_child(Span::new(0, 29), SyntaxId(6)),
            TraitItem.with_binary_children(Span::new(0, 29), SyntaxId(1), SyntaxId(5)),
            SimplePath.empty(Span::new(6, 9)),
            TraitBody.with_single_child(Span::new(10, 29), SyntaxId(4)),
            TypeItem.with_binary_children(Span::new(12, 27), SyntaxId(2), SyntaxId(3)),
            SimplePath.empty(Span::new(17, 20)),
            I64Type.empty(Span::new(23, 26)),
        ]
    );
}

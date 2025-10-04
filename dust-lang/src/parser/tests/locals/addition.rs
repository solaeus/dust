use crate::{
    parser::parse_main,
    source::Span,
    syntax_tree::{SyntaxKind, SyntaxNode},
    tests::local_cases,
};

#[test]
fn local_byte_addition() {
    let source = local_cases::LOCAL_BYTE_ADDITION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (4, 3),
                span: Span(0, 47)
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (2, 5),
                span: Span(1, 20)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(5, 6)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (0, 1),
                span: Span(5, 6)
            },
            SyntaxNode {
                kind: SyntaxKind::ByteType,
                children: (0, 0),
                span: Span(8, 12)
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (40, 0),
                span: Span(15, 19)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (4, 0),
                span: Span(15, 20)
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (8, 11),
                span: Span(21, 40)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(25, 26)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (1, 1),
                span: Span(25, 26)
            },
            SyntaxNode {
                kind: SyntaxKind::ByteType,
                children: (0, 0),
                span: Span(28, 32)
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (2, 0),
                span: Span(35, 39)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (10, 0),
                span: Span(35, 40)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(41, 42)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (2, 1),
                span: Span(41, 42)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (14, 0),
                span: Span(41, 42)
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (15, 18),
                span: Span(41, 46)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(45, 46)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (3, 1),
                span: Span(45, 46)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (17, 0),
                span: Span(45, 46)
            },
        ]
    );
}

#[test]
fn local_float_addition() {
    let source = local_cases::LOCAL_FLOAT_ADDITION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (4, 3),
                span: Span(0, 48)
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (2, 5),
                span: Span(1, 21)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(5, 6)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (0, 1),
                span: Span(5, 6)
            },
            SyntaxNode {
                kind: SyntaxKind::FloatType,
                children: (0, 0),
                span: Span(8, 13)
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(40.0),
                span: Span(16, 20)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (4, 0),
                span: Span(16, 21)
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (8, 11),
                span: Span(22, 41)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(26, 27)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (1, 1),
                span: Span(26, 27)
            },
            SyntaxNode {
                kind: SyntaxKind::FloatType,
                children: (0, 0),
                span: Span(29, 34)
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(2.0),
                span: Span(37, 40)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (10, 0),
                span: Span(37, 41)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(42, 43)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (2, 1),
                span: Span(42, 43)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (14, 0),
                span: Span(42, 43)
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (15, 18),
                span: Span(42, 47)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(46, 47)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (3, 1),
                span: Span(46, 47)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (17, 0),
                span: Span(46, 47)
            },
        ]
    );
}

#[test]
fn local_integer_addition() {
    let source = local_cases::LOCAL_INTEGER_ADDITION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (4, 3),
                span: Span(0, 40)
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (2, 5),
                span: Span(1, 17)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(5, 6)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (0, 1),
                span: Span(5, 6)
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                span: Span(8, 11)
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (40, 0),
                span: Span(14, 16)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (4, 0),
                span: Span(14, 17)
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (8, 11),
                span: Span(18, 33)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(22, 23)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (1, 1),
                span: Span(22, 23)
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                span: Span(25, 28)
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (2, 0),
                span: Span(31, 32)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (10, 0),
                span: Span(31, 33)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(34, 35)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (2, 1),
                span: Span(34, 35)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (14, 0),
                span: Span(34, 35)
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (15, 18),
                span: Span(34, 39)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(38, 39)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (3, 1),
                span: Span(38, 39)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (17, 0),
                span: Span(38, 39)
            },
        ]
    );
}

#[test]
fn local_string_concatenation() {
    let source = local_cases::LOCAL_STRING_CONCATENATION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    println!("sorted_nodes: {:?}", syntax_tree.sorted_nodes());
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (4, 3),
                span: Span(0, 47)
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (2, 5),
                span: Span(1, 20)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(5, 6)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (0, 1),
                span: Span(5, 6)
            },
            SyntaxNode {
                kind: SyntaxKind::StringType,
                children: (0, 0),
                span: Span(8, 11)
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(14, 19)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (4, 0),
                span: Span(14, 20)
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (8, 11),
                span: Span(21, 40)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(25, 26)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (1, 1),
                span: Span(25, 26)
            },
            SyntaxNode {
                kind: SyntaxKind::StringType,
                children: (0, 0),
                span: Span(28, 31)
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(34, 39)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (10, 0),
                span: Span(34, 40)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(41, 42)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (2, 1),
                span: Span(41, 42)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (14, 0),
                span: Span(41, 42)
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (15, 18),
                span: Span(41, 46)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(45, 46)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (3, 1),
                span: Span(45, 46)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (17, 0),
                span: Span(45, 46)
            },
        ]
    );
}

#[test]
fn local_character_concatenation() {
    let source = local_cases::LOCAL_CHARACTER_CONCATENATION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (4, 3),
                span: Span(0, 45)
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (2, 5),
                span: Span(1, 19)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(5, 6)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (0, 1),
                span: Span(5, 6)
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterType,
                children: (0, 0),
                span: Span(8, 12)
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: ('q' as u32, 0),
                span: Span(15, 18)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (4, 0),
                span: Span(15, 19)
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (8, 11),
                span: Span(20, 38)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(24, 25)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (1, 1),
                span: Span(24, 25)
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterType,
                children: (0, 0),
                span: Span(27, 31)
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: ('q' as u32, 0),
                span: Span(34, 37)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (10, 0),
                span: Span(34, 38)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(39, 40)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (2, 1),
                span: Span(39, 40)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (14, 0),
                span: Span(39, 40)
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (15, 18),
                span: Span(39, 44)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(43, 44)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (3, 1),
                span: Span(43, 44)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (17, 0),
                span: Span(43, 44)
            },
        ]
    );
}

#[test]
fn local_string_character_concatenation() {
    let source = local_cases::LOCAL_STRING_CHARACTER_CONCATENATION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    println!("sorted_nodes: {:?}", syntax_tree.sorted_nodes());
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (4, 3),
                span: Span(0, 46)
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (2, 5),
                span: Span(1, 20)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(5, 6)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (0, 1),
                span: Span(5, 6)
            },
            SyntaxNode {
                kind: SyntaxKind::StringType,
                children: (0, 0),
                span: Span(8, 11)
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(14, 19)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (4, 0),
                span: Span(14, 20)
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (8, 11),
                span: Span(21, 39)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(25, 26)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (1, 1),
                span: Span(25, 26)
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterType,
                children: (0, 0),
                span: Span(28, 32)
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: ('q' as u32, 0),
                span: Span(35, 38)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (10, 0),
                span: Span(35, 39)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(40, 41)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (2, 1),
                span: Span(40, 41)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (14, 0),
                span: Span(40, 41)
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (15, 18),
                span: Span(40, 45)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(44, 45)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (3, 1),
                span: Span(44, 45)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (17, 0),
                span: Span(44, 45)
            },
        ]
    );
}

#[test]
fn local_character_string_concatenation() {
    let source = local_cases::LOCAL_CHARACTER_STRING_CONCATENATION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (4, 3),
                span: Span(0, 46)
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (2, 5),
                span: Span(1, 19)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(5, 6)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (0, 1),
                span: Span(5, 6)
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterType,
                children: (0, 0),
                span: Span(8, 12)
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: ('q' as u32, 0),
                span: Span(15, 18)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (4, 0),
                span: Span(15, 19)
            },
            SyntaxNode {
                kind: SyntaxKind::LetStatement,
                children: (8, 11),
                span: Span(20, 39)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(24, 25)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (1, 1),
                span: Span(24, 25)
            },
            SyntaxNode {
                kind: SyntaxKind::StringType,
                children: (0, 0),
                span: Span(27, 30)
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(33, 38)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (10, 0),
                span: Span(33, 39)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(40, 41)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (2, 1),
                span: Span(40, 41)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (14, 0),
                span: Span(40, 41)
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionExpression,
                children: (15, 18),
                span: Span(40, 45)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(44, 45)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (3, 1),
                span: Span(44, 45)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (17, 0),
                span: Span(44, 45)
            },
        ]
    );
}

#[test]
fn local_mut_byte_addition() {
    let source = local_cases::LOCAL_MUT_BYTE_ADDITION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (3, 3),
                span: Span(0, 38)
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                children: (2, 5),
                span: Span(1, 24)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(9, 10)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (0, 1),
                span: Span(9, 10)
            },
            SyntaxNode {
                kind: SyntaxKind::ByteType,
                children: (0, 0),
                span: Span(12, 16)
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (40, 0),
                span: Span(19, 23)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (4, 0),
                span: Span(19, 24)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(25, 26)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (1, 1),
                span: Span(25, 26)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (8, 0),
                span: Span(25, 26)
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionAssignmentExpression,
                children: (9, 10),
                span: Span(25, 34)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (11, 0),
                span: Span(25, 35)
            },
            SyntaxNode {
                kind: SyntaxKind::ByteExpression,
                children: (2, 0),
                span: Span(30, 34)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(36, 37)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (2, 1),
                span: Span(36, 37)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (14, 0),
                span: Span(36, 37)
            },
        ]
    );
}

#[test]
fn local_mut_float_addition() {
    let source = local_cases::LOCAL_MUT_FLOAT_ADDITION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (3, 3),
                span: Span(0, 38)
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                children: (2, 5),
                span: Span(1, 25)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(9, 10)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (0, 1),
                span: Span(9, 10)
            },
            SyntaxNode {
                kind: SyntaxKind::FloatType,
                children: (0, 0),
                span: Span(12, 17)
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(40.0),
                span: Span(20, 24)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (4, 0),
                span: Span(20, 25)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(26, 27)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (1, 1),
                span: Span(26, 27)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (8, 0),
                span: Span(26, 27)
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionAssignmentExpression,
                children: (9, 10),
                span: Span(26, 34)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (11, 0),
                span: Span(26, 35)
            },
            SyntaxNode {
                kind: SyntaxKind::FloatExpression,
                children: SyntaxNode::encode_float(2.0),
                span: Span(31, 34)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(36, 37)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (2, 1),
                span: Span(36, 37)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (14, 0),
                span: Span(36, 37)
            },
        ]
    );
}

#[test]
fn local_mut_integer_addition() {
    let source = local_cases::LOCAL_MUT_INTEGER_ADDITION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (3, 3),
                span: Span(0, 32)
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                children: (2, 5),
                span: Span(1, 21)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(9, 10)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (0, 1),
                span: Span(9, 10)
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerType,
                children: (0, 0),
                span: Span(12, 15)
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (40, 0),
                span: Span(18, 20)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (4, 0),
                span: Span(18, 21)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(22, 23)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (1, 1),
                span: Span(22, 23)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (8, 0),
                span: Span(22, 23)
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionAssignmentExpression,
                children: (9, 10),
                span: Span(22, 28)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (11, 0),
                span: Span(22, 29)
            },
            SyntaxNode {
                kind: SyntaxKind::IntegerExpression,
                children: (2, 0),
                span: Span(27, 28)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(30, 31)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (2, 1),
                span: Span(30, 31)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (14, 0),
                span: Span(30, 31)
            },
        ]
    );
}

#[test]
fn local_mut_string_concatenation() {
    let source = local_cases::LOCAL_MUT_STRING_CONCATENATION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    println!("sorted_nodes: {:?}", syntax_tree.sorted_nodes());
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (3, 3),
                span: Span(0, 39)
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                children: (2, 5),
                span: Span(1, 24)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(9, 10)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (0, 1),
                span: Span(9, 10)
            },
            SyntaxNode {
                kind: SyntaxKind::StringType,
                children: (0, 0),
                span: Span(12, 15)
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(18, 23)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (4, 0),
                span: Span(18, 24)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(25, 26)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (1, 1),
                span: Span(25, 26)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (8, 0),
                span: Span(25, 26)
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionAssignmentExpression,
                children: (9, 10),
                span: Span(25, 35)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (11, 0),
                span: Span(25, 36)
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(30, 35)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(37, 38)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (2, 1),
                span: Span(37, 38)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (14, 0),
                span: Span(37, 38)
            },
        ]
    );
}

#[test]
fn local_mut_string_character_concatenation() {
    let source = local_cases::LOCAL_MUT_STRING_CHARACTER_CONCATENATION.to_string();
    let (syntax_tree, error) = parse_main(source);

    assert!(error.is_none(), "{error:?}");
    println!("sorted_nodes: {:?}", syntax_tree.sorted_nodes());
    assert_eq!(
        syntax_tree.sorted_nodes(),
        vec![
            SyntaxNode {
                kind: SyntaxKind::MainFunctionItem,
                children: (3, 3),
                span: Span(0, 37)
            },
            SyntaxNode {
                kind: SyntaxKind::LetMutStatement,
                children: (2, 5),
                span: Span(1, 24)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(9, 10)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (0, 1),
                span: Span(9, 10)
            },
            SyntaxNode {
                kind: SyntaxKind::StringType,
                children: (0, 0),
                span: Span(12, 15)
            },
            SyntaxNode {
                kind: SyntaxKind::StringExpression,
                children: (0, 0),
                span: Span(18, 23)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (4, 0),
                span: Span(18, 24)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(25, 26)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (1, 1),
                span: Span(25, 26)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (8, 0),
                span: Span(25, 26)
            },
            SyntaxNode {
                kind: SyntaxKind::AdditionAssignmentExpression,
                children: (9, 10),
                span: Span(25, 33)
            },
            SyntaxNode {
                kind: SyntaxKind::ExpressionStatement,
                children: (11, 0),
                span: Span(25, 34)
            },
            SyntaxNode {
                kind: SyntaxKind::CharacterExpression,
                children: ('q' as u32, 0),
                span: Span(30, 33)
            },
            SyntaxNode {
                kind: SyntaxKind::PathSegment,
                children: (0, 0),
                span: Span(35, 36)
            },
            SyntaxNode {
                kind: SyntaxKind::Path,
                children: (2, 1),
                span: Span(35, 36)
            },
            SyntaxNode {
                kind: SyntaxKind::PathExpression,
                children: (14, 0),
                span: Span(35, 36)
            },
        ]
    );
}

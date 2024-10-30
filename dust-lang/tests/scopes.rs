use dust_lang::*;

#[test]
fn allow_access_to_parent_scope() {
    let source = r#"
        let x = 1;
        {
            x
        }
    "#;

    assert_eq!(run(source), Ok(Some(Value::integer(1))));
}

// #[test]
// fn disallow_access_to_child_scope() {
//     let source = r#"
//         {
//             let x = 1;
//         }
//         x
//     "#;

//     assert_eq!(
//         run(source),
//         Err(DustError::Parse {
//             error: ParseError::Chunk(ChunkError::LocalOutOfScope {
//                 identifier: Identifier::new("x"),
//                 position: Span(52, 53)
//             }),
//             source
//         })
//     );
// }

// #[test]
// fn disallow_access_to_sibling_scope() {
//     let source = r#"
//         {
//             let x = 1;
//         }
//         {
//             x
//         }
//     "#;

//     assert_eq!(
//         run(source),
//         Err(DustError::Parse {
//             error: ParseError::Chunk(ChunkError::LocalOutOfScope {
//                 identifier: Identifier::new("x"),
//                 position: Span(52, 53)
//             }),
//             source
//         })
//     );
// }

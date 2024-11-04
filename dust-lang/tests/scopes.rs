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

#[test]
fn block_scope() {
    let source = "
        let a = 0;
        {
            let b = 42;
            {
                let c = 1;
            }
            let d = 2;
        }
        let e = 1;
    ";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(17, 18)),
                (Instruction::define_local(0, 0, false), Span(13, 14)),
                (Instruction::load_constant(1, 1, false), Span(50, 52)),
                (Instruction::define_local(1, 1, false), Span(46, 47)),
                (Instruction::load_constant(2, 2, false), Span(92, 93)),
                (Instruction::define_local(2, 2, false), Span(88, 89)),
                (Instruction::load_constant(3, 3, false), Span(129, 130)),
                (Instruction::define_local(3, 3, false), Span(125, 126)),
                (Instruction::load_constant(4, 2, false), Span(158, 159)),
                (Instruction::define_local(4, 4, false), Span(154, 155)),
                (Instruction::r#return(false), Span(165, 165))
            ],
            vec![
                Value::integer(0),
                Value::integer(42),
                Value::integer(1),
                Value::integer(2),
            ],
            vec![
                Local::new(Identifier::new("a"), None, false, 0, 0),
                Local::new(Identifier::new("b"), None, false, 1, 1),
                Local::new(Identifier::new("c"), None, false, 2, 2),
                Local::new(Identifier::new("d"), None, false, 1, 3),
                Local::new(Identifier::new("e"), None, false, 0, 4),
            ]
        )),
    );

    assert_eq!(run(source), Ok(None));
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

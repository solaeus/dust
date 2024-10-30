use dust_lang::*;

#[test]
fn equality_assignment_long() {
    let source = "let a = if 4 == 4 { true } else { false }; a";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::equal(true, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(13, 15)
                ),
                (Instruction::jump(1, true), Span(13, 15)),
                (Instruction::load_boolean(0, true, true), Span(20, 24)),
                (Instruction::load_boolean(0, false, false), Span(34, 39)),
                (Instruction::define_local(0, 0, false), Span(4, 5)),
                (Instruction::get_local(1, 0), Span(43, 44)),
                (Instruction::r#return(true), Span(44, 44)),
            ],
            vec![Value::integer(4), Value::integer(4)],
            vec![Local::new(Identifier::new("a"), None, false, 0, 0)]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::boolean(true))));
}

#[test]
fn equality_assignment_short() {
    let source = "let a = 4 == 4 a";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::equal(true, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(10, 12)
                ),
                (Instruction::jump(1, true), Span(10, 12)),
                (Instruction::load_boolean(0, true, true), Span(10, 12)),
                (Instruction::load_boolean(0, false, false), Span(10, 12)),
                (Instruction::define_local(0, 0, false), Span(4, 5)),
                (Instruction::get_local(1, 0), Span(15, 16)),
                (Instruction::r#return(true), Span(16, 16)),
            ],
            vec![Value::integer(4), Value::integer(4)],
            vec![Local::new(Identifier::new("a"), None, false, 0, 0)]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::boolean(true))));
}

#[test]
fn if_else_complex() {
    let source = "
        if 1 == 1 {
            1; 2; 3; 4;
        } else {
            1; 2; 3; 4;
        }";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::equal(true, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(14, 16)
                ),
                (Instruction::jump(5, true), Span(14, 16)),
                (Instruction::load_constant(0, 2, false), Span(33, 34)),
                (Instruction::load_constant(1, 3, false), Span(36, 37)),
                (Instruction::load_constant(2, 4, false), Span(39, 40)),
                (Instruction::load_constant(3, 5, false), Span(42, 43)),
                (Instruction::jump(5, true), Span(95, 95)),
                (Instruction::jump(4, true), Span(95, 95)),
                (Instruction::load_constant(4, 6, false), Span(74, 75)),
                (Instruction::load_constant(5, 7, false), Span(77, 78)),
                (Instruction::load_constant(6, 8, false), Span(80, 81)),
                (Instruction::load_constant(7, 9, false), Span(83, 84)),
                (Instruction::r#return(true), Span(95, 95)),
            ],
            vec![
                Value::integer(1),
                Value::integer(1),
                Value::integer(1),
                Value::integer(2),
                Value::integer(3),
                Value::integer(4),
                Value::integer(1),
                Value::integer(2),
                Value::integer(3),
                Value::integer(4)
            ],
            vec![]
        ))
    )
}

// #[test]
// fn if_else_nested() {
//     let source = r#"
//         if 0 == 1 {
//             if 0 == 2 {
//                 1;
//             } else {
//                 2;
//             }
//         } else {
//             if 0 == 3 {
//                 3;
//             } else {
//                 4;
//             }
//         }"#;

//     assert_eq!(
//         parse(source),
//         Ok(Chunk::with_data(
//             None,
//             vec![
//                 (
//                     *Instruction::equal(true, 0, 1)
//                         .set_b_is_constant()
//                         .set_c_is_constant(),
//                     Span(14, 16)
//                 ),
//                 (Instruction::jump(7, true), Span(14, 16)),
//                 (
//                     *Instruction::equal(true, 0, 2)
//                         .set_b_is_constant()
//                         .set_c_is_constant(),
//                     Span(38, 41)
//                 ),
//                 (Instruction::jump(3, true), Span(38, 41)),
//                 (Instruction::load_constant(0, 1, false), Span(61, 62)),
//                 (Instruction::jump(1, true1), Span(95, 95)),
//                 (
//                     *Instruction::equal(true, 0, 3)
//                         .set_b_is_constant()
//                         .set_c_is_constant(),
//                     Span(77, 79)
//                 ),
//                 (Instruction::jump(3, true), Span(77, 79)),
//                 (Instruction::load_constant(0, 2, false), Span(94, 95)),
//                 (Instruction::jump(1, true1), Span(95, 95)),
//                 (Instruction::load_constant(0, 3, false), Span(114, 115)),
//                 (Instruction::jump(1, true1), Span(95, 95)),
//                 (Instruction::load_constant(0, 4, false), Span(134, 135)),
//                 (Instruction::r#return(true), Span(146, 146)),
//             ],
//             vec![
//                 Value::integer(0),
//                 Value::integer(1),
//                 Value::integer(0),
//                 Value::integer(2),
//                 Value::integer(1),
//                 Value::integer(0),
//                 Value::integer(3),
//                 Value::integer(3),
//                 Value::integer(4)
//             ],
//             vec![]
//         ))
//     );

//     assert_eq!(run(source), Ok(Some(Value::integer(4))));
// }

#[test]
fn if_else_false() {
    let source = "if 1 == 2 { panic() } else { 42 }";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::equal(true, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(5, 7)
                ),
                (Instruction::jump(1, true), Span(5, 7)),
                (
                    Instruction::call_native(0, NativeFunction::Panic, 0),
                    Span(12, 19)
                ),
                (Instruction::load_constant(0, 2, true), Span(29, 31)),
                (Instruction::r#return(true), Span(33, 33)),
            ],
            vec![Value::integer(1), Value::integer(2), Value::integer(42)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::integer(42))));
}

#[test]
fn if_else_true() {
    let source = "if 1 == 1 { 42 } else { panic() }";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::equal(true, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(5, 7)
                ),
                (Instruction::jump(1, true), Span(5, 7)),
                (Instruction::load_constant(0, 2, true), Span(12, 14)),
                (
                    Instruction::call_native(1, NativeFunction::Panic, 0),
                    Span(24, 31)
                ),
                (Instruction::r#return(true), Span(33, 33))
            ],
            vec![Value::integer(1), Value::integer(1), Value::integer(42)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::integer(42))));
}

#[test]
fn if_expression_false() {
    let source = "if 1 == 2 { 2 }";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::equal(true, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(5, 7)
                ),
                (Instruction::jump(1, true), Span(5, 7)),
                (Instruction::load_constant(0, 2, false), Span(12, 13)),
            ],
            vec![Value::integer(1), Value::integer(2), Value::integer(2)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(None));
}

#[test]
fn if_expression_true() {
    let source = "if 1 == 1 { 2 }";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::equal(true, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(5, 7)
                ),
                (Instruction::jump(1, true), Span(5, 7)),
                (Instruction::load_constant(0, 2, false), Span(12, 13)),
            ],
            vec![Value::integer(1), Value::integer(1), Value::integer(2)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(None));
}

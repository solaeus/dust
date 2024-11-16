use dust_lang::*;

#[test]
fn equality_assignment_long() {
    let source = "let a = if 4 == 4 { true } else { false }; a";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::equal(true, 0, 0)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(13, 15)
                ),
                (Instruction::jump(1, true), Span(18, 19)),
                (Instruction::load_boolean(0, true, true), Span(20, 24)),
                (Instruction::load_boolean(0, false, false), Span(34, 39)),
                (Instruction::define_local(0, 0, false), Span(4, 5)),
                (Instruction::get_local(1, 0), Span(43, 44)),
                (Instruction::r#return(true), Span(44, 44)),
            ],
            vec![ConcreteValue::Integer(4), ConcreteValue::string("a")],
            vec![Local::new(1, Type::Boolean, false, Scope::default(),)]
        )),
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Boolean(true))));
}

#[test]
fn equality_assignment_short() {
    let source = "let a = 4 == 4 a";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::equal(true, 0, 0)
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
            vec![ConcreteValue::Integer(4), ConcreteValue::string("a")],
            vec![Local::new(1, Type::Boolean, false, Scope::default())]
        )),
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Boolean(true))));
}

#[test]
fn if_else_assigment_false() {
    let source = r#"
        let a = if 4 == 3 {
            1; 2; 3; 4;
            panic()
        } else {
            1; 2; 3; 4;
            42
        };
        a"#;

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::equal(true, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(22, 24)
                ),
                (Instruction::jump(6, true), Span(27, 28)),
                (Instruction::load_constant(0, 2, false), Span(41, 42)),
                (Instruction::load_constant(1, 3, false), Span(44, 45)),
                (Instruction::load_constant(2, 1, false), Span(47, 48)),
                (Instruction::load_constant(3, 0, false), Span(50, 51)),
                (
                    Instruction::call_native(4, NativeFunction::Panic, 0),
                    Span(65, 72)
                ),
                (Instruction::jump(5, true), Span(138, 139)),
                (Instruction::load_constant(5, 2, false), Span(102, 103)),
                (Instruction::load_constant(6, 3, false), Span(105, 106)),
                (Instruction::load_constant(7, 1, false), Span(108, 109)),
                (Instruction::load_constant(8, 0, false), Span(111, 112)),
                (Instruction::load_constant(9, 4, false), Span(126, 128)),
                (Instruction::r#move(9, 4), Span(138, 139)),
                (Instruction::define_local(9, 0, false), Span(13, 14)),
                (Instruction::get_local(10, 0), Span(148, 149)),
                (Instruction::r#return(true), Span(149, 149)),
            ],
            vec![
                ConcreteValue::Integer(4),
                ConcreteValue::Integer(3),
                ConcreteValue::Integer(1),
                ConcreteValue::Integer(2),
                ConcreteValue::Integer(42),
                ConcreteValue::string("a")
            ],
            vec![Local::new(5, Type::Integer, false, Scope::default())]
        )),
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(42))));
}

#[test]
fn if_else_assigment_true() {
    let source = r#"
        let a = if 4 == 4 {
            1; 2; 3; 4;
            42
        } else {
            1; 2; 3; 4;
            panic()
        };
        a"#;

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::equal(true, 0, 0)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(22, 24)
                ),
                (Instruction::jump(6, true), Span(27, 28)),
                (Instruction::load_constant(0, 1, false), Span(41, 42)),
                (Instruction::load_constant(1, 2, false), Span(44, 45)),
                (Instruction::load_constant(2, 3, false), Span(47, 48)),
                (Instruction::load_constant(3, 0, false), Span(50, 51)),
                (Instruction::load_constant(4, 4, false), Span(65, 67)),
                (Instruction::jump(5, true), Span(138, 139)),
                (Instruction::load_constant(5, 1, false), Span(97, 98)),
                (Instruction::load_constant(6, 2, false), Span(100, 101)),
                (Instruction::load_constant(7, 3, false), Span(103, 104)),
                (Instruction::load_constant(8, 0, false), Span(106, 107)),
                (
                    Instruction::call_native(9, NativeFunction::Panic, 0),
                    Span(121, 128)
                ),
                (Instruction::r#move(9, 4), Span(138, 139)),
                (Instruction::define_local(9, 0, false), Span(13, 14)),
                (Instruction::get_local(10, 0), Span(148, 149)),
                (Instruction::r#return(true), Span(149, 149)),
            ],
            vec![
                ConcreteValue::Integer(4),
                ConcreteValue::Integer(1),
                ConcreteValue::Integer(2),
                ConcreteValue::Integer(3),
                ConcreteValue::Integer(42),
                ConcreteValue::string("a")
            ],
            vec![Local::new(5, Type::Integer, false, Scope::default())]
        )),
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(42))));
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
        compile(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::equal(true, 0, 0)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(14, 16)
                ),
                (Instruction::jump(5, true), Span(19, 20)),
                (Instruction::load_constant(0, 0, false), Span(33, 34)),
                (Instruction::load_constant(1, 1, false), Span(36, 37)),
                (Instruction::load_constant(2, 2, false), Span(39, 40)),
                (Instruction::load_constant(3, 3, false), Span(42, 43)),
                (Instruction::jump(4, true), Span(95, 95)),
                (Instruction::load_constant(4, 0, false), Span(74, 75)),
                (Instruction::load_constant(5, 1, false), Span(77, 78)),
                (Instruction::load_constant(6, 2, false), Span(80, 81)),
                (Instruction::load_constant(7, 3, false), Span(83, 84)),
                (Instruction::r#move(7, 3), Span(95, 95)),
                (Instruction::r#return(false), Span(95, 95)),
            ],
            vec![
                ConcreteValue::Integer(1),
                ConcreteValue::Integer(2),
                ConcreteValue::Integer(3),
                ConcreteValue::Integer(4),
            ],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(None));
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
//                 ConcreteValue::integer(0),
//                 ConcreteValue::integer(1),
//                 ConcreteValue::integer(0),
//                 ConcreteValue::integer(2),
//                 ConcreteValue::integer(1),
//                 ConcreteValue::integer(0),
//                 ConcreteValue::integer(3),
//                 ConcreteValue::integer(3),
//                 ConcreteValue::integer(4)
//             ],
//             vec![]
//         ))
//     );

//     assert_eq!(run(source), Ok(Some(ConcreteValue::integer(4))));
// }

#[test]
fn if_else_false() {
    let source = "if 1 == 2 { panic(); 0 } else { 42 }";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::equal(true, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(5, 7)
                ),
                (Instruction::jump(1, true), Span(10, 11)),
                (
                    Instruction::call_native(0, NativeFunction::Panic, 0),
                    Span(12, 19)
                ),
                (Instruction::load_constant(1, 2, true), Span(29, 31)),
                (Instruction::r#move(1, 0), Span(33, 33)),
                (Instruction::r#return(true), Span(33, 33)),
            ],
            vec![
                ConcreteValue::Integer(1),
                ConcreteValue::Integer(2),
                ConcreteValue::Integer(42)
            ],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(42))));
}

#[test]
fn if_else_true() {
    let source = "if 1 == 1 { 42 } else { panic(); 0 }";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::equal(true, 0, 0)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(5, 7)
                ),
                (Instruction::jump(1, true), Span(10, 11)),
                (Instruction::load_constant(0, 1, true), Span(12, 14)),
                (
                    Instruction::call_native(1, NativeFunction::Panic, 0),
                    Span(24, 31)
                ),
                (Instruction::r#move(1, 0), Span(33, 33)),
                (Instruction::r#return(true), Span(33, 33))
            ],
            vec![ConcreteValue::Integer(1), ConcreteValue::Integer(42)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(42))));
}

#[test]
fn if_false() {
    let source = "if 1 == 2 { panic() }";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::equal(true, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(5, 7)
                ),
                (Instruction::jump(1, true), Span(10, 11)),
                (
                    Instruction::call_native(0, NativeFunction::Panic, 0),
                    Span(12, 19)
                ),
                (Instruction::r#return(false), Span(21, 21))
            ],
            vec![ConcreteValue::Integer(1), ConcreteValue::Integer(2)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(None));
}

#[test]
fn if_true() {
    let source = "if 1 == 1 { panic() }";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::equal(true, 0, 0)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(5, 7)
                ),
                (Instruction::jump(1, true), Span(10, 11)),
                (
                    Instruction::call_native(0, NativeFunction::Panic, 0),
                    Span(12, 19)
                ),
                (Instruction::r#return(false), Span(21, 21))
            ],
            vec![ConcreteValue::Integer(1)],
            vec![]
        )),
    );

    assert_eq!(
        run(source),
        Err(DustError::Runtime {
            error: VmError::NativeFunction(NativeFunctionError::Panic {
                message: None,
                position: Span(12, 19)
            }),
            source
        })
    );
}

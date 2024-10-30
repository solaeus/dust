use dust_lang::*;

#[test]
fn add() {
    let source = "1 + 2";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::add(0, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(2, 3)
                ),
                (Instruction::r#return(true), Span(5, 5))
            ],
            vec![Value::integer(1), Value::integer(2)],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::integer(3))));
}

#[test]
fn add_assign() {
    let source = "let mut a = 1; a += 2; a";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(12, 13)),
                (Instruction::define_local(0, 0, true), Span(8, 9)),
                (*Instruction::add(0, 0, 1).set_c_is_constant(), Span(17, 19)),
                (Instruction::get_local(1, 0), Span(23, 24)),
                (Instruction::r#return(true), Span(24, 24))
            ],
            vec![Value::integer(1), Value::integer(2)],
            vec![Local::new(Identifier::new("a"), None, true, 0, 0)]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::integer(3))));
}

#[test]
fn and() {
    let source = "true && false";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_boolean(0, true, false), Span(0, 4)),
                (Instruction::test(0, false), Span(5, 7)),
                (Instruction::jump(4), Span(5, 7)),
                (Instruction::load_boolean(1, false, false), Span(8, 13)),
                (Instruction::r#return(true), Span(13, 13)),
            ],
            vec![],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::boolean(false))));
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
                (Instruction::load_constant(4, 4, false), Span(158, 159)),
                (Instruction::define_local(4, 4, false), Span(154, 155)),
            ],
            vec![
                Value::integer(0),
                Value::integer(42),
                Value::integer(1),
                Value::integer(2),
                Value::integer(1)
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

#[test]
fn constant() {
    let source = "42";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(0, 2)),
                (Instruction::r#return(true), Span(2, 2))
            ],
            vec![Value::integer(42)],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::integer(42))));
}

#[test]
fn define_local() {
    let source = "let x = 42;";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(8, 10)),
                (Instruction::define_local(0, 0, false), Span(4, 5)),
            ],
            vec![Value::integer(42)],
            vec![Local::new(Identifier::new("x"), None, false, 0, 0)]
        )),
    );

    assert_eq!(run(source), Ok(None));
}

#[test]
fn divide() {
    let source = "2 / 2";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::divide(0, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(2, 3)
                ),
                (Instruction::r#return(true), Span(5, 5))
            ],
            vec![Value::integer(2), Value::integer(2)],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::integer(1))));
}

#[test]
fn divide_assign() {
    let source = "let mut a = 2; a /= 2; a";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(12, 13)),
                (Instruction::define_local(0, 0, true), Span(8, 9)),
                (
                    *Instruction::divide(0, 0, 1).set_c_is_constant(),
                    Span(17, 19)
                ),
                (Instruction::get_local(1, 0), Span(23, 24)),
                (Instruction::r#return(true), Span(24, 24))
            ],
            vec![Value::integer(2), Value::integer(2)],
            vec![Local::new(Identifier::new("a"), None, true, 0, 0)]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::integer(1))));
}

#[test]
fn empty() {
    let source = "";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(None, vec![], vec![], vec![]))
    );
    assert_eq!(run(source), Ok(None));
}

#[test]
fn empty_list() {
    let source = "[]";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_list(0, 0, 0), Span(0, 2)),
                (Instruction::r#return(true), Span(2, 2)),
            ],
            vec![],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::list(0, 0, Type::Any))));
}

#[test]
fn equal() {
    let source = "1 == 2";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::equal(true, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(2, 4)
                ),
                (Instruction::jump(3), Span(2, 4)),
                (Instruction::load_boolean(0, true, true), Span(2, 4)),
                (Instruction::load_boolean(0, false, false), Span(2, 4)),
                (Instruction::r#return(true), Span(6, 6)),
            ],
            vec![Value::integer(1), Value::integer(2)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::boolean(false))));
}

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
                (Instruction::jump(3), Span(13, 15)),
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
                (Instruction::jump(3), Span(10, 12)),
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
fn function() {
    let source = "fn(a: int, b: int) -> int { a + b }";

    assert_eq!(
        run(source),
        Ok(Some(Value::function(
            Chunk::with_data(
                None,
                vec![
                    (Instruction::add(2, 0, 1), Span(30, 31)),
                    (Instruction::r#return(true), Span(34, 35)),
                ],
                vec![],
                vec![
                    Local::new(Identifier::new("a"), Some(Type::Integer), false, 0, 0),
                    Local::new(Identifier::new("b"), Some(Type::Integer), false, 0, 1)
                ]
            ),
            FunctionType {
                type_parameters: None,
                value_parameters: Some(vec![
                    (Identifier::new("a"), Type::Integer),
                    (Identifier::new("b"), Type::Integer)
                ]),
                return_type: Some(Box::new(Type::Integer)),
            }
        )))
    );
}

#[test]
fn function_declaration() {
    let source = "fn add (a: int, b: int) -> int { a + b }";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(0, 40)),
                (Instruction::define_local(0, 0, false), Span(3, 6)),
            ],
            vec![Value::function(
                Chunk::with_data(
                    None,
                    vec![
                        (Instruction::add(2, 0, 1), Span(35, 36)),
                        (Instruction::r#return(true), Span(39, 40)),
                    ],
                    vec![],
                    vec![
                        Local::new(Identifier::new("a"), Some(Type::Integer), false, 0, 0),
                        Local::new(Identifier::new("b"), Some(Type::Integer), false, 0, 1)
                    ]
                ),
                FunctionType {
                    type_parameters: None,
                    value_parameters: Some(vec![
                        (Identifier::new("a"), Type::Integer),
                        (Identifier::new("b"), Type::Integer)
                    ]),
                    return_type: Some(Box::new(Type::Integer)),
                },
            )],
            vec![Local::new(
                Identifier::new("add"),
                Some(Type::Function(FunctionType {
                    type_parameters: None,
                    value_parameters: Some(vec![
                        (Identifier::new("a"), Type::Integer),
                        (Identifier::new("b"), Type::Integer)
                    ]),
                    return_type: Some(Box::new(Type::Integer)),
                })),
                false,
                0,
                0
            ),],
        )),
    );

    assert_eq!(run(source), Ok(None));
}

#[test]
fn function_call() {
    let source = "fn(a: int, b: int) -> int { a + b }(1, 2)";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(0, 36)),
                (Instruction::load_constant(1, 1, false), Span(36, 37)),
                (Instruction::load_constant(2, 2, false), Span(39, 40)),
                (Instruction::call(3, 0, 2), Span(35, 41)),
                (Instruction::r#return(true), Span(41, 41)),
            ],
            vec![
                Value::function(
                    Chunk::with_data(
                        None,
                        vec![
                            (Instruction::add(2, 0, 1), Span(30, 31)),
                            (Instruction::r#return(true), Span(34, 35)),
                        ],
                        vec![],
                        vec![
                            Local::new(Identifier::new("a"), Some(Type::Integer), false, 0, 0),
                            Local::new(Identifier::new("b"), Some(Type::Integer), false, 0, 1)
                        ]
                    ),
                    FunctionType {
                        type_parameters: None,
                        value_parameters: Some(vec![
                            (Identifier::new("a"), Type::Integer),
                            (Identifier::new("b"), Type::Integer)
                        ]),
                        return_type: Some(Box::new(Type::Integer)),
                    }
                ),
                Value::integer(1),
                Value::integer(2)
            ],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::integer(3))));
}

#[test]
fn greater() {
    let source = "1 > 2";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::less_equal(false, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(2, 3)
                ),
                (Instruction::jump(3), Span(2, 3)),
                (Instruction::load_boolean(0, true, true), Span(2, 3)),
                (Instruction::load_boolean(0, false, false), Span(2, 3)),
                (Instruction::r#return(true), Span(5, 5)),
            ],
            vec![Value::integer(1), Value::integer(2)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::boolean(false))));
}

#[test]
fn greater_than_or_equal() {
    let source = "1 >= 2";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::less(false, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(2, 4)
                ),
                (Instruction::jump(3), Span(2, 4)),
                (Instruction::load_boolean(0, true, true), Span(2, 4)),
                (Instruction::load_boolean(0, false, false), Span(2, 4)),
                (Instruction::r#return(true), Span(6, 6)),
            ],
            vec![Value::integer(1), Value::integer(2)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::boolean(false))));
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
                (Instruction::jump(7), Span(14, 16)),
                (Instruction::load_constant(0, 2, false), Span(33, 34)),
                (Instruction::load_constant(1, 3, false), Span(36, 37)),
                (Instruction::load_constant(2, 4, false), Span(39, 40)),
                (Instruction::load_constant(3, 5, false), Span(42, 43)),
                (Instruction::jump(11), Span(95, 95)),
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

#[test]
fn if_else_nested() {
    let source = r#"
        if 0 == 1 {
            if 0 == 2 {
                1;
            } else {
                2;
            }
        } else {
            if 0 == 3 {
                3;
            } else {
                4;
            }
        }"#;

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
                (Instruction::jump(7), Span(14, 16)),
                (
                    *Instruction::equal(true, 0, 2)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(38, 41)
                ),
                (Instruction::jump(3), Span(38, 41)),
                (Instruction::load_constant(0, 1, false), Span(61, 62)),
                (Instruction::jump(11), Span(95, 95)),
                (
                    *Instruction::equal(true, 0, 3)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(77, 79)
                ),
                (Instruction::jump(3), Span(77, 79)),
                (Instruction::load_constant(0, 2, false), Span(94, 95)),
                (Instruction::jump(11), Span(95, 95)),
                (Instruction::load_constant(0, 3, false), Span(114, 115)),
                (Instruction::jump(11), Span(95, 95)),
                (Instruction::load_constant(0, 4, false), Span(134, 135)),
                (Instruction::r#return(true), Span(146, 146)),
            ],
            vec![
                Value::integer(0),
                Value::integer(1),
                Value::integer(0),
                Value::integer(2),
                Value::integer(1),
                Value::integer(0),
                Value::integer(3),
                Value::integer(3),
                Value::integer(4)
            ],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::integer(4))));
}

#[test]
fn if_else_simple() {
    let source = "if 1 == 1 { 2 } else { 3 }";

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
                (Instruction::jump(3), Span(5, 7)),
                (Instruction::load_constant(0, 2, true), Span(12, 13)),
                (Instruction::load_constant(0, 3, false), Span(23, 24)),
                (Instruction::r#return(true), Span(26, 26)),
            ],
            vec![
                Value::integer(1),
                Value::integer(1),
                Value::integer(2),
                Value::integer(3)
            ],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::integer(2))));
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
                (Instruction::jump(3), Span(5, 7)),
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
                (Instruction::jump(3), Span(5, 7)),
                (Instruction::load_constant(0, 2, false), Span(12, 13)),
            ],
            vec![Value::integer(1), Value::integer(1), Value::integer(2)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(None));
}

#[test]
fn less_than() {
    let source = "1 < 2";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::less(true, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(2, 3)
                ),
                (Instruction::jump(3), Span(2, 3)),
                (Instruction::load_boolean(0, true, true), Span(2, 3)),
                (Instruction::load_boolean(0, false, false), Span(2, 3)),
                (Instruction::r#return(true), Span(5, 5)),
            ],
            vec![Value::integer(1), Value::integer(2)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::boolean(true))));
}

#[test]
fn less_than_or_equal() {
    let source = "1 <= 2";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::less_equal(true, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(2, 4)
                ),
                (Instruction::jump(3), Span(2, 4)),
                (Instruction::load_boolean(0, true, true), Span(2, 4)),
                (Instruction::load_boolean(0, false, false), Span(2, 4)),
                (Instruction::r#return(true), Span(6, 6)),
            ],
            vec![Value::integer(1), Value::integer(2)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::boolean(true))));
}

#[test]
fn list() {
    let source = "[1, 2, 3]";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(1, 2)),
                (Instruction::load_constant(1, 1, false), Span(4, 5)),
                (Instruction::load_constant(2, 2, false), Span(7, 8)),
                (Instruction::load_list(3, 0, 2), Span(0, 9)),
                (Instruction::r#return(true), Span(9, 9)),
            ],
            vec![Value::integer(1), Value::integer(2), Value::integer(3),],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::list(0, 2, Type::Integer))));
}

#[test]
fn list_with_complex_expression() {
    let source = "[1, 2 + 3 - 4 * 5]";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(1, 2)),
                (
                    *Instruction::add(1, 1, 2)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(6, 7)
                ),
                (
                    *Instruction::multiply(2, 3, 4)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(14, 15)
                ),
                (Instruction::subtract(3, 1, 2), Span(10, 11)),
                (Instruction::close(1, 3), Span(17, 18)),
                (Instruction::load_list(4, 0, 3), Span(0, 18)),
                (Instruction::r#return(true), Span(18, 18)),
            ],
            vec![
                Value::integer(1),
                Value::integer(2),
                Value::integer(3),
                Value::integer(4),
                Value::integer(5)
            ],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::list(0, 3, Type::Integer))));
}

#[test]
fn list_with_simple_expression() {
    let source = "[1, 2 + 3, 4]";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(1, 2)),
                (
                    *Instruction::add(1, 1, 2)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(6, 7)
                ),
                (Instruction::load_constant(2, 3, false), Span(11, 12)),
                (Instruction::load_list(3, 0, 2), Span(0, 13)),
                (Instruction::r#return(true), Span(13, 13)),
            ],
            vec![
                Value::integer(1),
                Value::integer(2),
                Value::integer(3),
                Value::integer(4),
            ],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::list(0, 2, Type::Integer))));
}

#[test]
fn math_operator_precedence() {
    let source = "1 + 2 - 3 * 4 / 5";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::add(0, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(2, 3)
                ),
                (
                    *Instruction::multiply(1, 2, 3)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(10, 11)
                ),
                (
                    *Instruction::divide(2, 1, 4).set_c_is_constant(),
                    Span(14, 15)
                ),
                (Instruction::subtract(3, 0, 2), Span(6, 7)),
                (Instruction::r#return(true), Span(17, 17)),
            ],
            vec![
                Value::integer(1),
                Value::integer(2),
                Value::integer(3),
                Value::integer(4),
                Value::integer(5),
            ],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::integer(1))));
}

#[test]
fn multiply() {
    let source = "1 * 2";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::multiply(0, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(2, 3)
                ),
                (Instruction::r#return(true), Span(5, 5)),
            ],
            vec![Value::integer(1), Value::integer(2)],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::integer(2))));
}

#[test]
fn multiply_assign() {
    let source = "let mut a = 2; a *= 3 a";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(12, 13)),
                (Instruction::define_local(0, 0, true), Span(8, 9)),
                (
                    *Instruction::multiply(0, 0, 1).set_c_is_constant(),
                    Span(17, 19)
                ),
                (Instruction::get_local(1, 0), Span(22, 23)),
                (Instruction::r#return(true), Span(23, 23))
            ],
            vec![Value::integer(2), Value::integer(3)],
            vec![Local::new(Identifier::new("a"), None, true, 0, 0),]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::integer(6))));
}

#[test]
fn negate() {
    let source = "-(42)";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (*Instruction::negate(0, 0).set_b_is_constant(), Span(0, 1)),
                (Instruction::r#return(true), Span(5, 5)),
            ],
            vec![Value::integer(42)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::integer(-42))));
}

#[test]
fn not() {
    let source = "!true";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_boolean(0, true, false), Span(1, 5)),
                (Instruction::not(1, 0), Span(0, 1)),
                (Instruction::r#return(true), Span(5, 5)),
            ],
            vec![],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::boolean(false))));
}

#[test]
fn not_equal() {
    let source = "1 != 2";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::equal(false, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(2, 4)
                ),
                (Instruction::jump(3), Span(2, 4)),
                (Instruction::load_boolean(0, true, true), Span(2, 4)),
                (Instruction::load_boolean(0, false, false), Span(2, 4)),
                (Instruction::r#return(true), Span(6, 6)),
            ],
            vec![Value::integer(1), Value::integer(2)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::boolean(true))));
}

#[test]
fn or() {
    let source = "true || false";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_boolean(0, true, false), Span(0, 4)),
                (Instruction::test(0, true), Span(5, 7)),
                (Instruction::jump(4), Span(5, 7)),
                (Instruction::load_boolean(1, false, false), Span(8, 13)),
                (Instruction::r#return(true), Span(13, 13)),
            ],
            vec![],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::boolean(true))));
}

#[test]
fn parentheses_precedence() {
    let source = "(1 + 2) * 3";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::add(0, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(3, 4)
                ),
                (
                    *Instruction::multiply(1, 0, 2).set_c_is_constant(),
                    Span(8, 9)
                ),
                (Instruction::r#return(true), Span(11, 11)),
            ],
            vec![Value::integer(1), Value::integer(2), Value::integer(3)],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::integer(9))));
}

#[test]
fn set_local() {
    let source = "let mut x = 41; x = 42; x";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(12, 14)),
                (Instruction::define_local(0, 0, true), Span(8, 9)),
                (Instruction::load_constant(1, 1, false), Span(20, 22)),
                (Instruction::set_local(1, 0), Span(16, 17)),
                (Instruction::get_local(2, 0), Span(24, 25)),
                (Instruction::r#return(true), Span(25, 25)),
            ],
            vec![Value::integer(41), Value::integer(42)],
            vec![Local::new(Identifier::new("x"), None, true, 0, 0)]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::integer(42))));
}

#[test]
fn subtract() {
    let source = "1 - 2";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (
                    *Instruction::subtract(0, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(2, 3)
                ),
                (Instruction::r#return(true), Span(5, 5)),
            ],
            vec![Value::integer(1), Value::integer(2)],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::integer(-1))));
}

#[test]
fn subtract_assign() {
    let source = "let mut x = 42; x -= 2; x";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(12, 14)),
                (Instruction::define_local(0, 0, true), Span(8, 9)),
                (
                    *Instruction::subtract(0, 0, 1).set_c_is_constant(),
                    Span(18, 20)
                ),
                (Instruction::get_local(1, 0), Span(24, 25)),
                (Instruction::r#return(true), Span(25, 25)),
            ],
            vec![Value::integer(42), Value::integer(2)],
            vec![Local::new(Identifier::new("x"), None, true, 0, 0)]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::integer(40))));
}

#[test]
fn variable_and() {
    let source = "let a = true; let b = false; a && b";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_boolean(0, true, false), Span(8, 12)),
                (Instruction::define_local(0, 0, false), Span(4, 5)),
                (Instruction::load_boolean(1, false, false), Span(22, 27)),
                (Instruction::define_local(1, 1, false), Span(18, 19)),
                (Instruction::get_local(2, 0), Span(29, 30)),
                (Instruction::test(2, false), Span(31, 33)),
                (Instruction::jump(8), Span(31, 33)),
                (Instruction::get_local(3, 1), Span(34, 35)),
                (Instruction::r#return(true), Span(35, 35)),
            ],
            vec![],
            vec![
                Local::new(Identifier::new("a"), None, false, 0, 0),
                Local::new(Identifier::new("b"), None, false, 0, 1),
            ]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::boolean(false))));
}

#[test]
fn r#while() {
    let source = "let mut x = 0; while x < 5 { x = x + 1 } x";

    assert_eq!(
        parse(source),
        Ok(Chunk::with_data(
            None,
            vec![
                (Instruction::load_constant(0, 0, false), Span(12, 13)),
                (Instruction::define_local(0, 0, true), Span(8, 9)),
                (
                    *Instruction::less(true, 0, 1).set_c_is_constant(),
                    Span(23, 24)
                ),
                (Instruction::jump(7), Span(23, 24)),
                (*Instruction::add(0, 0, 2).set_c_is_constant(), Span(39, 40)),
                (Instruction::jump(2), Span(41, 42)),
                (Instruction::get_local(1, 0), Span(41, 42)),
                (Instruction::r#return(true), Span(42, 42)),
            ],
            vec![Value::integer(0), Value::integer(5), Value::integer(1),],
            vec![Local::new(Identifier::new("x"), None, true, 0, 0),]
        )),
    );

    assert_eq!(run(source), Ok(Some(Value::integer(5))));
}

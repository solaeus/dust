use dust_lang::*;

#[test]
fn add() {
    let source = "1 + 2";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::Integer),
            },
            vec![
                (
                    Instruction::add(
                        Destination::Register(0),
                        Argument::Constant(0),
                        Argument::Constant(1)
                    ),
                    Type::Integer,
                    Span(2, 3)
                ),
                (Instruction::r#return(true), Type::None, Span(5, 5))
            ],
            vec![ConcreteValue::Integer(1), ConcreteValue::Integer(2)],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(3))));
}

#[test]
fn add_assign() {
    let source = "let mut a = 1; a += 2; a";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::Integer),
            },
            vec![
                (
                    Instruction::load_constant(Destination::Register(0), 0, false),
                    Type::Integer,
                    Span(12, 13)
                ),
                (
                    Instruction::define_local(0, 0, true),
                    Type::None,
                    Span(8, 9)
                ),
                (
                    Instruction::add(
                        Destination::Local(0),
                        Argument::Local(0),
                        Argument::Constant(2)
                    ),
                    Type::None,
                    Span(17, 19)
                ),
                (
                    Instruction::get_local(Destination::Register(1), 0),
                    Type::Integer,
                    Span(23, 24)
                ),
                (Instruction::r#return(true), Type::None, Span(24, 24))
            ],
            vec![
                ConcreteValue::Integer(1),
                ConcreteValue::string("a"),
                ConcreteValue::Integer(2)
            ],
            vec![Local::new(1, Type::Integer, true, Scope::default())]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(3))));
}

#[test]
fn add_assign_expects_mutable_variable() {
    let source = "1 += 2";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::ExpectedMutableVariable {
                found: Token::Integer("1").to_owned(),
                position: Span(0, 1)
            },
            source
        })
    );
}

// #[test]
// fn add_expects_integer_float_or_string() {
//     let source = "true + false";

//     assert_eq!(
//         parse(source),
//         Err(DustError::Parse {
//             error: ParseError::ExpectedIntegerFloatOrString {
//                 found: Token::True,
//                 position: Span(0, 3)
//             },
//             source
//         })
//     );
// }

#[test]
fn divide() {
    let source = "2 / 2";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::Integer),
            },
            vec![
                (
                    Instruction::divide(
                        Destination::Register(0),
                        Argument::Constant(0),
                        Argument::Constant(0)
                    ),
                    Type::Integer,
                    Span(2, 3)
                ),
                (Instruction::r#return(true), Type::None, Span(5, 5))
            ],
            vec![ConcreteValue::Integer(2)],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(1))));
}

#[test]
fn divide_assign() {
    let source = "let mut a = 2; a /= 2; a";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::Integer),
            },
            vec![
                (
                    Instruction::load_constant(Destination::Register(0), 0, false),
                    Type::Integer,
                    Span(12, 13)
                ),
                (
                    Instruction::define_local(0, 0, true),
                    Type::None,
                    Span(8, 9)
                ),
                (
                    Instruction::divide(
                        Destination::Local(0),
                        Argument::Local(0),
                        Argument::Constant(0)
                    ),
                    Type::None,
                    Span(17, 19)
                ),
                (
                    Instruction::get_local(Destination::Register(1), 0),
                    Type::Integer,
                    Span(23, 24)
                ),
                (Instruction::r#return(true), Type::None, Span(24, 24))
            ],
            vec![ConcreteValue::Integer(2), ConcreteValue::string("a")],
            vec![Local::new(1, Type::Integer, true, Scope::default())]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(1))));
}

#[test]
fn divide_assign_expects_mutable_variable() {
    let source = "1 -= 2";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::ExpectedMutableVariable {
                found: Token::Integer("1").to_owned(),
                position: Span(0, 1)
            },
            source
        })
    );
}

#[test]
fn math_operator_precedence() {
    let source = "1 + 2 - 3 * 4 / 5";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::Integer),
            },
            vec![
                (
                    Instruction::add(
                        Destination::Register(0),
                        Argument::Constant(0),
                        Argument::Constant(1)
                    ),
                    Type::Integer,
                    Span(2, 3)
                ),
                (
                    Instruction::multiply(
                        Destination::Register(1),
                        Argument::Constant(2),
                        Argument::Constant(3)
                    ),
                    Type::Integer,
                    Span(10, 11)
                ),
                (
                    Instruction::divide(
                        Destination::Register(2),
                        Argument::Register(1),
                        Argument::Constant(4)
                    ),
                    Type::Integer,
                    Span(14, 15)
                ),
                (
                    Instruction::subtract(
                        Destination::Register(3),
                        Argument::Register(0),
                        Argument::Register(2)
                    ),
                    Type::Integer,
                    Span(6, 7)
                ),
                (Instruction::r#return(true), Type::None, Span(17, 17)),
            ],
            vec![
                ConcreteValue::Integer(1),
                ConcreteValue::Integer(2),
                ConcreteValue::Integer(3),
                ConcreteValue::Integer(4),
                ConcreteValue::Integer(5),
            ],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(1))));
}

#[test]
fn multiply() {
    let source = "1 * 2";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::Integer),
            },
            vec![
                (
                    Instruction::multiply(
                        Destination::Register(0),
                        Argument::Constant(0),
                        Argument::Constant(1)
                    ),
                    Type::Integer,
                    Span(2, 3)
                ),
                (Instruction::r#return(true), Type::None, Span(5, 5)),
            ],
            vec![ConcreteValue::Integer(1), ConcreteValue::Integer(2)],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(2))));
}

#[test]
fn multiply_assign() {
    let source = "let mut a = 2; a *= 3 a";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::Integer),
            },
            vec![
                (
                    Instruction::load_constant(Destination::Register(0), 0, false),
                    Type::Integer,
                    Span(12, 13)
                ),
                (
                    Instruction::define_local(0, 0, true),
                    Type::None,
                    Span(8, 9)
                ),
                (
                    Instruction::multiply(
                        Destination::Local(0),
                        Argument::Local(0),
                        Argument::Constant(2)
                    ),
                    Type::None,
                    Span(17, 19)
                ),
                (
                    Instruction::get_local(Destination::Register(1), 0),
                    Type::Integer,
                    Span(22, 23)
                ),
                (Instruction::r#return(true), Type::None, Span(23, 23))
            ],
            vec![
                ConcreteValue::Integer(2),
                ConcreteValue::string("a"),
                ConcreteValue::Integer(3)
            ],
            vec![Local::new(1, Type::Integer, true, Scope::default())]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(6))));
}

#[test]
fn multiply_assign_expects_mutable_variable() {
    let source = "1 *= 2";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::ExpectedMutableVariable {
                found: Token::Integer("1").to_owned(),
                position: Span(0, 1)
            },
            source
        })
    );
}

#[test]
fn subtract() {
    let source = "1 - 2";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::Integer),
            },
            vec![
                (
                    Instruction::subtract(
                        Destination::Register(0),
                        Argument::Constant(0),
                        Argument::Constant(1)
                    ),
                    Type::Integer,
                    Span(2, 3)
                ),
                (Instruction::r#return(true), Type::None, Span(5, 5)),
            ],
            vec![ConcreteValue::Integer(1), ConcreteValue::Integer(2)],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(-1))));
}

#[test]
fn subtract_assign() {
    let source = "let mut x = 42; x -= 2; x";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::Integer),
            },
            vec![
                (
                    Instruction::load_constant(Destination::Register(0), 0, false),
                    Type::Integer,
                    Span(12, 14)
                ),
                (
                    Instruction::define_local(0, 0, true),
                    Type::None,
                    Span(8, 9)
                ),
                (
                    Instruction::subtract(
                        Destination::Local(0),
                        Argument::Local(0),
                        Argument::Constant(2)
                    ),
                    Type::None,
                    Span(18, 20)
                ),
                (
                    Instruction::get_local(Destination::Register(1), 0),
                    Type::Integer,
                    Span(24, 25)
                ),
                (Instruction::r#return(true), Type::None, Span(25, 25)),
            ],
            vec![
                ConcreteValue::Integer(42),
                ConcreteValue::string("x"),
                ConcreteValue::Integer(2)
            ],
            vec![Local::new(1, Type::Integer, true, Scope::default())]
        )),
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(40))));
}

#[test]
fn subtract_assign_expects_mutable_variable() {
    let source = "1 -= 2";

    assert_eq!(
        compile(source),
        Err(DustError::Compile {
            error: CompileError::ExpectedMutableVariable {
                found: Token::Integer("1").to_owned(),
                position: Span(0, 1)
            },
            source
        })
    );
}

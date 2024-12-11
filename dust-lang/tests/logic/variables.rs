use dust_lang::*;

#[test]
fn true_and_true() {
    let source = "let a = true; let b = true; a && b";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Type::Boolean,
            },
            vec![
                (Instruction::load_boolean(0, true, false), Span(8, 12)),
                (Instruction::load_boolean(1, true, false), Span(22, 26)),
                (Instruction::get_local(2, 0), Span(28, 29)),
                (Instruction::test(Argument::Register(0), true), Span(30, 32)),
                (Instruction::jump(1, true), Span(30, 32)),
                (Instruction::get_local(3, 1), Span(33, 34)),
                (Instruction::r#return(true), Span(34, 34)),
            ],
            vec![ConcreteValue::string("a"), ConcreteValue::string("b")],
            vec![
                Local::new(0, 0, false, Scope::default()),
                Local::new(1, 1, false, Scope::default())
            ]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Boolean(true))));
}

#[test]
fn false_and_false() {
    let source = "let a = false; let b = false; a && b";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Type::Boolean,
            },
            vec![
                (Instruction::load_boolean(0, false, false), Span(8, 13)),
                (Instruction::load_boolean(1, false, false), Span(23, 28)),
                (Instruction::get_local(2, 0), Span(30, 31)),
                (Instruction::test(Argument::Register(0), true), Span(32, 34)),
                (Instruction::jump(1, true), Span(32, 34)),
                (Instruction::get_local(3, 1), Span(35, 36)),
                (Instruction::r#return(true), Span(36, 36)),
            ],
            vec![ConcreteValue::string("a"), ConcreteValue::string("b")],
            vec![
                Local::new(0, 0, false, Scope::default()),
                Local::new(1, 0, false, Scope::default())
            ]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Boolean(false))));
}

#[test]
fn true_and_false() {
    let source = "let a = true; let b = false; a && b";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Type::Boolean,
            },
            vec![
                (Instruction::load_boolean(0, true, false), Span(8, 12)),
                (Instruction::load_boolean(1, false, false), Span(22, 27)),
                (Instruction::get_local(2, 0), Span(29, 30)),
                (Instruction::test(Argument::Register(0), true), Span(31, 33)),
                (Instruction::jump(1, true), Span(31, 33)),
                (Instruction::get_local(3, 1), Span(34, 35)),
                (Instruction::r#return(true), Span(35, 35)),
            ],
            vec![ConcreteValue::string("a"), ConcreteValue::string("b")],
            vec![
                Local::new(0, 0, false, Scope::default()),
                Local::new(1, 1, false, Scope::default())
            ]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Boolean(false))));
}

#[test]
fn false_and_true() {
    let source = "let a = false; let b = true; a && b";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Type::Boolean,
            },
            vec![
                (Instruction::load_boolean(0, false, false), Span(8, 13)),
                (Instruction::load_boolean(1, true, false), Span(23, 27)),
                (Instruction::get_local(2, 0), Span(29, 30)),
                (Instruction::test(Argument::Register(0), true), Span(31, 33)),
                (Instruction::jump(1, true), Span(31, 33)),
                (Instruction::get_local(3, 1), Span(34, 35)),
                (Instruction::r#return(true), Span(35, 35)),
            ],
            vec![ConcreteValue::string("a"), ConcreteValue::string("b")],
            vec![
                Local::new(0, 0, false, Scope::default()),
                Local::new(1, 1, false, Scope::default())
            ]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Boolean(false))));
}

#[test]
fn true_and_true_and_true() {
    let source = "let a = true; let b = true; let c = true; a && b && c";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Type::Boolean,
            },
            vec![
                (Instruction::load_boolean(0, true, false), Span(8, 12)),
                (Instruction::load_boolean(1, true, false), Span(22, 26)),
                (Instruction::load_boolean(2, true, false), Span(36, 40)),
                (Instruction::get_local(3, 0), Span(42, 43)),
                (Instruction::test(Argument::Register(0), true), Span(44, 46)),
                (Instruction::jump(1, true), Span(44, 46)),
                (Instruction::get_local(4, 1), Span(47, 48)),
                (Instruction::test(Argument::Register(1), true), Span(49, 51)),
                (Instruction::jump(1, true), Span(49, 51)),
                (Instruction::get_local(5, 2), Span(52, 53)),
                (Instruction::r#return(true), Span(53, 53)),
            ],
            vec![
                ConcreteValue::string("a"),
                ConcreteValue::string("b"),
                ConcreteValue::string("c")
            ],
            vec![
                Local::new(0, 0, false, Scope::default()),
                Local::new(1, 1, false, Scope::default()),
                Local::new(2, 2, false, Scope::default())
            ]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Boolean(true))));
}

#[test]
fn false_and_false_and_false() {
    let source = "let a = false; let b = false; let c = false; a && b && c";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Type::Boolean,
            },
            vec![
                (Instruction::load_boolean(0, false, false), Span(8, 13)),
                (Instruction::load_boolean(1, false, false), Span(23, 28)),
                (Instruction::load_boolean(2, false, false), Span(38, 43)),
                (Instruction::get_local(3, 0), Span(45, 46)),
                (Instruction::test(Argument::Register(0), true), Span(47, 49)),
                (Instruction::jump(1, true), Span(47, 49)),
                (Instruction::get_local(4, 1), Span(50, 51)),
                (Instruction::test(Argument::Register(1), true), Span(52, 54)),
                (Instruction::jump(1, true), Span(52, 54)),
                (Instruction::get_local(5, 2), Span(55, 56)),
                (Instruction::r#return(true), Span(56, 56)),
            ],
            vec![
                ConcreteValue::string("a"),
                ConcreteValue::string("b"),
                ConcreteValue::string("c")
            ],
            vec![
                Local::new(0, 0, false, Scope::default()),
                Local::new(1, 1, false, Scope::default()),
                Local::new(2, 2, false, Scope::default())
            ]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Boolean(false))));
}

#[test]
fn true_and_true_or_false() {
    let source = "let a = true; let b = true; let c = false; a && b || c";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Type::Boolean,
            },
            vec![
                (Instruction::load_boolean(0, true, false), Span(8, 12)),
                (Instruction::load_boolean(1, true, false), Span(22, 26)),
                (Instruction::load_boolean(2, false, false), Span(36, 41)),
                (Instruction::get_local(3, 0), Span(43, 44)),
                (Instruction::test(Argument::Register(0), true), Span(45, 47)),
                (Instruction::jump(1, true), Span(45, 47)),
                (Instruction::get_local(4, 1), Span(48, 49)),
                (
                    Instruction::test(Argument::Register(1), false),
                    Span(50, 52)
                ),
                (Instruction::jump(1, true), Span(50, 52)),
                (Instruction::get_local(5, 2), Span(53, 54)),
                (Instruction::r#return(true), Span(54, 54)),
            ],
            vec![
                ConcreteValue::string("a"),
                ConcreteValue::string("b"),
                ConcreteValue::string("c")
            ],
            vec![
                Local::new(0, 0, false, Scope::default()),
                Local::new(1, 1, false, Scope::default()),
                Local::new(2, 2, false, Scope::default())
            ]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Boolean(true))));
}

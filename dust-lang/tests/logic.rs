use dust_lang::*;

#[test]
fn and() {
    let source = "true && false";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::Boolean),
            },
            vec![
                (
                    Instruction::load_boolean(Destination::Register(0), true, false),
                    Type::Boolean,
                    Span(0, 4)
                ),
                (
                    Instruction::test(Argument::Register(0), true),
                    Type::None,
                    Span(5, 7)
                ),
                (Instruction::jump(1, true), Type::None, Span(5, 7)),
                (
                    Instruction::load_boolean(Destination::Register(1), false, false),
                    Type::Boolean,
                    Span(8, 13)
                ),
                (Instruction::r#return(true), Type::None, Span(13, 13)),
            ],
            vec![],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Boolean(false))));
}

#[test]
fn or() {
    let source = "true || false";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::Boolean),
            },
            vec![
                (
                    Instruction::load_boolean(Destination::Register(0), true, false),
                    Type::Boolean,
                    Span(0, 4)
                ),
                (
                    Instruction::test(Argument::Register(0), false),
                    Type::None,
                    Span(5, 7)
                ),
                (Instruction::jump(1, true), Type::None, Span(5, 7)),
                (
                    Instruction::load_boolean(Destination::Register(1), false, false),
                    Type::Boolean,
                    Span(8, 13)
                ),
                (Instruction::r#return(true), Type::None, Span(13, 13)),
            ],
            vec![],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Boolean(true))));
}

#[test]
fn and_and_or() {
    let source = "let a = true; let b = true; let c = false; a && b || c";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::Boolean),
            },
            vec![
                (
                    Instruction::load_boolean(Destination::Register(0), true, false),
                    Type::Boolean,
                    Span(8, 12)
                ),
                (
                    Instruction::define_local(0, 0, false),
                    Type::None,
                    Span(4, 5)
                ),
                (
                    Instruction::load_boolean(Destination::Register(1), true, false),
                    Type::Boolean,
                    Span(22, 26)
                ),
                (
                    Instruction::define_local(1, 1, false),
                    Type::None,
                    Span(18, 19)
                ),
                (
                    Instruction::load_boolean(Destination::Register(2), false, false),
                    Type::Boolean,
                    Span(36, 41)
                ),
                (
                    Instruction::define_local(2, 2, false),
                    Type::None,
                    Span(32, 33)
                ),
                (
                    Instruction::test(Argument::Local(0), true),
                    Type::None,
                    Span(45, 47)
                ),
                (Instruction::jump(1, true), Type::None, Span(45, 47)),
                (
                    Instruction::test(Argument::Local(1), false),
                    Type::None,
                    Span(50, 52)
                ),
                (Instruction::jump(1, true), Type::None, Span(50, 52)),
                (
                    Instruction::get_local(Destination::Register(3), 2),
                    Type::Boolean,
                    Span(53, 54)
                ),
                (Instruction::r#return(true), Type::None, Span(54, 54)),
            ],
            vec![
                ConcreteValue::string("a"),
                ConcreteValue::string("b"),
                ConcreteValue::string("c")
            ],
            vec![
                Local::new(0, Type::Boolean, false, Scope::default()),
                Local::new(1, Type::Boolean, false, Scope::default()),
                Local::new(2, Type::Boolean, false, Scope::default())
            ]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Boolean(true))));
}

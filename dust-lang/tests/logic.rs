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
                    Instruction::load_boolean(0, true, false),
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
                    Instruction::load_boolean(1, false, false),
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
                    Instruction::load_boolean(0, true, false),
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
                    Instruction::load_boolean(1, false, false),
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
fn variable_and() {
    let source = "let a = true; let b = false; a && b";

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
                    Instruction::load_boolean(0, true, false),
                    Type::Boolean,
                    Span(8, 12)
                ),
                (
                    Instruction::define_local(0, 0, false),
                    Type::None,
                    Span(4, 5)
                ),
                (
                    Instruction::load_boolean(1, false, false),
                    Type::Boolean,
                    Span(22, 27)
                ),
                (
                    Instruction::define_local(1, 1, false),
                    Type::None,
                    Span(18, 19)
                ),
                (Instruction::get_local(2, 0), Type::Boolean, Span(29, 30)),
                (
                    Instruction::test(Argument::Register(2), true),
                    Type::None,
                    Span(31, 33)
                ),
                (Instruction::jump(1, true), Type::None, Span(31, 33)),
                (Instruction::get_local(3, 1), Type::Boolean, Span(34, 35)),
                (Instruction::r#return(true), Type::None, Span(35, 35)),
            ],
            vec![ConcreteValue::string("a"), ConcreteValue::string("b"),],
            vec![
                Local::new(0, Type::Boolean, false, Scope::default()),
                Local::new(1, Type::Boolean, false, Scope::default()),
            ]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Boolean(false))));
}

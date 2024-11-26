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
                (Instruction::load_boolean(0, true, false), Span(0, 4)),
                (Instruction::test(0, true), Span(5, 7)),
                (Instruction::jump(1, true), Span(5, 7)),
                (Instruction::load_boolean(1, false, false), Span(8, 13)),
                (Instruction::r#return(true), Span(13, 13)),
            ],
            vec![],
            vec![]
        ))
    );

    assert_eq!(run_source(source), Ok(Some(ConcreteValue::Boolean(false))));
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
                (Instruction::load_boolean(0, true, false), Span(0, 4)),
                (Instruction::test(0, false), Span(5, 7)),
                (Instruction::jump(1, true), Span(5, 7)),
                (Instruction::load_boolean(1, false, false), Span(8, 13)),
                (Instruction::r#return(true), Span(13, 13)),
            ],
            vec![],
            vec![]
        ))
    );

    assert_eq!(run_source(source), Ok(Some(ConcreteValue::Boolean(true))));
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
                (Instruction::load_boolean(0, true, false), Span(8, 12)),
                (Instruction::define_local(0, 0, false), Span(4, 5)),
                (Instruction::load_boolean(1, false, false), Span(22, 27)),
                (Instruction::define_local(1, 1, false), Span(18, 19)),
                (Instruction::get_local(2, 0), Span(29, 30)),
                (Instruction::test(2, true), Span(31, 33)),
                (Instruction::jump(1, true), Span(31, 33)),
                (Instruction::get_local(3, 1), Span(34, 35)),
                (Instruction::r#return(true), Span(35, 35)),
            ],
            vec![ConcreteValue::string("a"), ConcreteValue::string("b"),],
            vec![
                Local::new(0, Type::Boolean, false, Scope::default()),
                Local::new(1, Type::Boolean, false, Scope::default()),
            ]
        ))
    );

    assert_eq!(run_source(source), Ok(Some(ConcreteValue::Boolean(false))));
}

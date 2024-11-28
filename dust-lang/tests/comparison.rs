use dust_lang::*;

#[test]
fn equal() {
    let source = "1 == 2";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::Boolean)
            },
            vec![
                (
                    Instruction::equal(true, Argument::Constant(0), Argument::Constant(1)),
                    Type::None,
                    Span(2, 4)
                ),
                (Instruction::jump(1, true), Type::None, Span(2, 4)),
                (
                    Instruction::load_boolean(0, true, true),
                    Type::Boolean,
                    Span(2, 4)
                ),
                (
                    Instruction::load_boolean(0, false, false),
                    Type::Boolean,
                    Span(2, 4)
                ),
                (Instruction::r#return(true), Type::None, Span(6, 6)),
            ],
            vec![ConcreteValue::Integer(1), ConcreteValue::Integer(2)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Boolean(false))));
}

#[test]
fn greater() {
    let source = "1 > 2";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::Boolean)
            },
            vec![
                (
                    Instruction::less_equal(false, Argument::Constant(0), Argument::Constant(1)),
                    Type::None,
                    Span(2, 3)
                ),
                (Instruction::jump(1, true), Type::None, Span(2, 3)),
                (
                    Instruction::load_boolean(0, true, true),
                    Type::Boolean,
                    Span(2, 3)
                ),
                (
                    Instruction::load_boolean(0, false, false),
                    Type::Boolean,
                    Span(2, 3)
                ),
                (Instruction::r#return(true), Type::None, Span(5, 5)),
            ],
            vec![ConcreteValue::Integer(1), ConcreteValue::Integer(2)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Boolean(false))));
}

#[test]
fn greater_than_or_equal() {
    let source = "1 >= 2";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::Boolean)
            },
            vec![
                (
                    Instruction::less(false, Argument::Constant(0), Argument::Constant(1)),
                    Type::None,
                    Span(2, 4)
                ),
                (Instruction::jump(1, true), Type::None, Span(2, 4)),
                (
                    Instruction::load_boolean(0, true, true),
                    Type::Boolean,
                    Span(2, 4)
                ),
                (
                    Instruction::load_boolean(0, false, false),
                    Type::Boolean,
                    Span(2, 4)
                ),
                (Instruction::r#return(true), Type::None, Span(6, 6)),
            ],
            vec![ConcreteValue::Integer(1), ConcreteValue::Integer(2)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Boolean(false))));
}

#[test]
fn less_than() {
    let source = "1 < 2";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::Boolean)
            },
            vec![
                (
                    Instruction::less(true, Argument::Constant(0), Argument::Constant(1)),
                    Type::None,
                    Span(2, 3)
                ),
                (Instruction::jump(1, true), Type::None, Span(2, 3)),
                (
                    Instruction::load_boolean(0, true, true),
                    Type::Boolean,
                    Span(2, 3)
                ),
                (
                    Instruction::load_boolean(0, false, false),
                    Type::Boolean,
                    Span(2, 3)
                ),
                (Instruction::r#return(true), Type::None, Span(5, 5)),
            ],
            vec![ConcreteValue::Integer(1), ConcreteValue::Integer(2)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Boolean(true))));
}

#[test]
fn less_than_or_equal() {
    let source = "1 <= 2";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::Boolean)
            },
            vec![
                (
                    Instruction::less_equal(true, Argument::Constant(0), Argument::Constant(1)),
                    Type::None,
                    Span(2, 4)
                ),
                (Instruction::jump(1, true), Type::None, Span(2, 4)),
                (
                    Instruction::load_boolean(0, true, true),
                    Type::Boolean,
                    Span(2, 4)
                ),
                (
                    Instruction::load_boolean(0, false, false),
                    Type::Boolean,
                    Span(2, 4)
                ),
                (Instruction::r#return(true), Type::None, Span(6, 6)),
            ],
            vec![ConcreteValue::Integer(1), ConcreteValue::Integer(2)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Boolean(true))));
}

#[test]
fn not_equal() {
    let source = "1 != 2";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::Boolean)
            },
            vec![
                (
                    Instruction::equal(false, Argument::Constant(0), Argument::Constant(1)),
                    Type::None,
                    Span(2, 4)
                ),
                (Instruction::jump(1, true), Type::None, Span(2, 4)),
                (
                    Instruction::load_boolean(0, true, true),
                    Type::Boolean,
                    Span(2, 4)
                ),
                (
                    Instruction::load_boolean(0, false, false),
                    Type::Boolean,
                    Span(2, 4)
                ),
                (Instruction::r#return(true), Type::None, Span(6, 6)),
            ],
            vec![ConcreteValue::Integer(1), ConcreteValue::Integer(2)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Boolean(true))));
}

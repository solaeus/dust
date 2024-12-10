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
                return_type: Type::Boolean
            },
            vec![
                (
                    Instruction::equal(0, true, Argument::Constant(0), Argument::Constant(1)),
                    Span(2, 4)
                ),
                (Instruction::r#return(true), Span(6, 6)),
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
                return_type: Type::Boolean
            },
            vec![
                (
                    Instruction::less_equal(0, false, Argument::Constant(0), Argument::Constant(1)),
                    Span(2, 3)
                ),
                (Instruction::r#return(true), Span(5, 5)),
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
                return_type: Type::Boolean
            },
            vec![
                (
                    Instruction::less(0, false, Argument::Constant(0), Argument::Constant(1)),
                    Span(2, 4)
                ),
                (Instruction::r#return(true), Span(6, 6)),
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
                return_type: Type::Boolean
            },
            vec![
                (
                    Instruction::less(0, true, Argument::Constant(0), Argument::Constant(1)),
                    Span(2, 3)
                ),
                (Instruction::r#return(true), Span(5, 5)),
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
                return_type: Type::Boolean
            },
            vec![
                (
                    Instruction::less_equal(0, true, Argument::Constant(0), Argument::Constant(1)),
                    Span(2, 4)
                ),
                (Instruction::r#return(true), Span(6, 6)),
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
                return_type: Type::Boolean
            },
            vec![
                (
                    Instruction::equal(0, false, Argument::Constant(0), Argument::Constant(1)),
                    Span(2, 4)
                ),
                (Instruction::r#return(true), Span(6, 6)),
            ],
            vec![ConcreteValue::Integer(1), ConcreteValue::Integer(2)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Boolean(true))));
}

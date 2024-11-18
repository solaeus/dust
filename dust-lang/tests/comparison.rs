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
                    *Instruction::equal(true, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(2, 4)
                ),
                (Instruction::jump(1, true), Span(2, 4)),
                (Instruction::load_boolean(0, true, true), Span(2, 4)),
                (Instruction::load_boolean(0, false, false), Span(2, 4)),
                (Instruction::r#return(true), Span(6, 6)),
            ],
            vec![ConcreteValue::Integer(1), ConcreteValue::Integer(2)],
            vec![]
        )),
    );

    assert_eq!(run_source(source), Ok(Some(ConcreteValue::Boolean(false))));
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
                    *Instruction::less_equal(false, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(2, 3)
                ),
                (Instruction::jump(1, true), Span(2, 3)),
                (Instruction::load_boolean(0, true, true), Span(2, 3)),
                (Instruction::load_boolean(0, false, false), Span(2, 3)),
                (Instruction::r#return(true), Span(5, 5)),
            ],
            vec![ConcreteValue::Integer(1), ConcreteValue::Integer(2)],
            vec![]
        )),
    );

    assert_eq!(run_source(source), Ok(Some(ConcreteValue::Boolean(false))));
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
                    *Instruction::less(false, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(2, 4)
                ),
                (Instruction::jump(1, true), Span(2, 4)),
                (Instruction::load_boolean(0, true, true), Span(2, 4)),
                (Instruction::load_boolean(0, false, false), Span(2, 4)),
                (Instruction::r#return(true), Span(6, 6)),
            ],
            vec![ConcreteValue::Integer(1), ConcreteValue::Integer(2)],
            vec![]
        )),
    );

    assert_eq!(run_source(source), Ok(Some(ConcreteValue::Boolean(false))));
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
                    *Instruction::less(true, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(2, 3)
                ),
                (Instruction::jump(1, true), Span(2, 3)),
                (Instruction::load_boolean(0, true, true), Span(2, 3)),
                (Instruction::load_boolean(0, false, false), Span(2, 3)),
                (Instruction::r#return(true), Span(5, 5)),
            ],
            vec![ConcreteValue::Integer(1), ConcreteValue::Integer(2)],
            vec![]
        )),
    );

    assert_eq!(run_source(source), Ok(Some(ConcreteValue::Boolean(true))));
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
                    *Instruction::less_equal(true, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(2, 4)
                ),
                (Instruction::jump(1, true), Span(2, 4)),
                (Instruction::load_boolean(0, true, true), Span(2, 4)),
                (Instruction::load_boolean(0, false, false), Span(2, 4)),
                (Instruction::r#return(true), Span(6, 6)),
            ],
            vec![ConcreteValue::Integer(1), ConcreteValue::Integer(2)],
            vec![]
        )),
    );

    assert_eq!(run_source(source), Ok(Some(ConcreteValue::Boolean(true))));
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
                    *Instruction::equal(false, 0, 1)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(2, 4)
                ),
                (Instruction::jump(1, true), Span(2, 4)),
                (Instruction::load_boolean(0, true, true), Span(2, 4)),
                (Instruction::load_boolean(0, false, false), Span(2, 4)),
                (Instruction::r#return(true), Span(6, 6)),
            ],
            vec![ConcreteValue::Integer(1), ConcreteValue::Integer(2)],
            vec![]
        )),
    );

    assert_eq!(run_source(source), Ok(Some(ConcreteValue::Boolean(true))));
}

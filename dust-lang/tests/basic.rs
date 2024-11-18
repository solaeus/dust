use dust_lang::*;

#[test]
fn constant() {
    let source = "42";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::Integer)
            },
            vec![
                (Instruction::load_constant(0, 0, false), Span(0, 2)),
                (Instruction::r#return(true), Span(2, 2))
            ],
            vec![ConcreteValue::Integer(42)],
            vec![]
        ))
    );

    assert_eq!(run_source(source), Ok(Some(ConcreteValue::Integer(42))));
}

#[test]
fn empty() {
    let source = "";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::None)
            },
            vec![(Instruction::r#return(false), Span(0, 0))],
            vec![],
            vec![]
        ))
    );
    assert_eq!(run_source(source), Ok(None));
}

#[test]
fn parentheses_precedence() {
    let source = "(1 + 2) * 3";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::Integer)
            },
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
            vec![
                ConcreteValue::Integer(1),
                ConcreteValue::Integer(2),
                ConcreteValue::Integer(3)
            ],
            vec![]
        ))
    );

    assert_eq!(run_source(source), Ok(Some(ConcreteValue::Integer(9))));
}

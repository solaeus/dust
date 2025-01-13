use dust_lang::*;

#[test]
fn negate() {
    let source = "-(42)";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Type::Integer,
            },
            vec![
                (Instruction::negate(0, Operand::Constant(0)), Span(0, 1)),
                (Instruction::r#return(true), Span(5, 5)),
            ],
            vec![ConcreteValue::Integer(42)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(-42))));
}

#[test]
fn not() {
    let source = "!true";

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
                (Instruction::load_boolean(0, true, false), Span(1, 5)),
                (Instruction::not(1, Operand::Register(0)), Span(0, 1)),
                (Instruction::r#return(true), Span(5, 5)),
            ],
            vec![],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Boolean(false))));
}

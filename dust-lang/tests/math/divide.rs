use dust_lang::*;

#[test]
fn divide_bytes() {
    let source = "0xff / 0x01";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Type::Byte,
            },
            vec![
                (
                    Instruction::divide(0, Operand::Constant(0), Operand::Constant(1)),
                    Span(5, 6)
                ),
                (Instruction::r#return(true), Span(11, 11))
            ],
            vec![ConcreteValue::Byte(0xff), ConcreteValue::Byte(0x01)],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Byte(0xff))));
}

#[test]
fn divide_floats() {
    let source = "2.0 / 2.0";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Type::Float,
            },
            vec![
                (
                    Instruction::divide(0, Operand::Constant(0), Operand::Constant(0)),
                    Span(4, 5)
                ),
                (Instruction::r#return(true), Span(9, 9))
            ],
            vec![ConcreteValue::Float(2.0)],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Float(1.0))));
}

#[test]
fn divide_integers() {
    let source = "2 / 2";

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
                (
                    Instruction::divide(0, Operand::Constant(0), Operand::Constant(0)),
                    Span(2, 3)
                ),
                (Instruction::r#return(true), Span(5, 5))
            ],
            vec![ConcreteValue::Integer(2)],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(1))));
}

use dust_lang::*;

#[test]
fn subtract() {
    let source = "1 - 2";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::Integer),
            },
            vec![
                (
                    Instruction::subtract(
                        Destination::Register(0),
                        Argument::Constant(0),
                        Argument::Constant(1)
                    ),
                    Span(2, 3)
                ),
                (Instruction::r#return(true), Span(5, 5)),
            ],
            vec![ConcreteValue::Integer(1), ConcreteValue::Integer(2)],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(-1))));
}

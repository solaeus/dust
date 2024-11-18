#[test]
fn if_true() {
    let source = "if true && true { 42 } else { 0 }";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::None)
            },
            vec![
                (
                    *Instruction::equal(true, 0, 0)
                        .set_b_is_constant()
                        .set_c_is_constant(),
                    Span(5, 7)
                ),
                (Instruction::jump(1, true), Span(10, 11)),
                (
                    Instruction::call_native(0, NativeFunction::Panic, 0),
                    Span(12, 19)
                ),
                (Instruction::r#return(false), Span(21, 21))
            ],
            vec![ConcreteValue::Integer(1)],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(42))),);
}

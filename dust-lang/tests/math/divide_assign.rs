use dust_lang::*;

#[test]
fn divide_assign() {
    let source = "let mut a = 2; a /= 2; a";

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
                (Instruction::load_constant(0, 0, false), Span(12, 13)),
                (
                    Instruction::divide(0, Argument::Register(0), Argument::Constant(0)),
                    Span(17, 19)
                ),
                (Instruction::get_local(1, 0), Span(23, 24)),
                (Instruction::r#return(true), Span(24, 24))
            ],
            vec![ConcreteValue::Integer(2), ConcreteValue::string("a")],
            vec![Local::new(1, 0, true, Scope::default())]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(1))));
}

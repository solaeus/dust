use dust_lang::*;

#[test]
fn multiply_assign() {
    let source = "let mut a = 2; a *= 3 a";

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
                    Instruction::load_constant(Destination::Register(0), 0, false),
                    Span(12, 13)
                ),
                (Instruction::define_local(0, 0, true), Span(8, 9)),
                (
                    Instruction::multiply(
                        Destination::Local(0),
                        Argument::Local(0),
                        Argument::Constant(2)
                    ),
                    Span(17, 19)
                ),
                (
                    Instruction::get_local(Destination::Register(1), 0),
                    Span(22, 23)
                ),
                (Instruction::r#return(true), Span(23, 23))
            ],
            vec![
                ConcreteValue::Integer(2),
                ConcreteValue::string("a"),
                ConcreteValue::Integer(3)
            ],
            vec![Local::new(1, Type::Integer, true, Scope::default())]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(6))));
}

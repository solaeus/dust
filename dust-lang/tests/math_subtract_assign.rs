use dust_lang::*;

#[test]
fn subtract_assign() {
    let source = "let mut x = 42; x -= 2; x";

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
                    Span(12, 14)
                ),
                (Instruction::define_local(0, 0, true), Span(8, 9)),
                (
                    Instruction::subtract(
                        Destination::Local(0),
                        Argument::Local(0),
                        Argument::Constant(2)
                    ),
                    Span(18, 20)
                ),
                (
                    Instruction::get_local(Destination::Register(1), 0),
                    Span(24, 25)
                ),
                (Instruction::r#return(true), Span(25, 25)),
            ],
            vec![
                ConcreteValue::Integer(42),
                ConcreteValue::string("x"),
                ConcreteValue::Integer(2)
            ],
            vec![Local::new(1, Type::Integer, true, Scope::default())]
        )),
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(40))));
}

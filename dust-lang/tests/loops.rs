use dust_lang::*;

#[test]
fn r#while() {
    let source = "let mut x = 0; while x < 5 { x = x + 1 } x";

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
                    Type::Integer,
                    Span(12, 13)
                ),
                (
                    Instruction::define_local(0, 0, true),
                    Type::None,
                    Span(8, 9)
                ),
                (
                    Instruction::less(true, Argument::Local(0), Argument::Constant(2)),
                    Type::None,
                    Span(23, 24)
                ),
                (Instruction::jump(2, true), Type::None, Span(41, 42)),
                (
                    Instruction::add(
                        Destination::Local(0),
                        Argument::Local(0),
                        Argument::Constant(3)
                    ),
                    Type::Integer,
                    Span(35, 36)
                ),
                (Instruction::jump(3, false), Type::None, Span(41, 42)),
                (
                    Instruction::get_local(Destination::Register(1), 0),
                    Type::Integer,
                    Span(41, 42)
                ),
                (Instruction::r#return(true), Type::None, Span(42, 42)),
            ],
            vec![
                ConcreteValue::Integer(0),
                ConcreteValue::string("x"),
                ConcreteValue::Integer(5),
                ConcreteValue::Integer(1),
            ],
            vec![Local::new(1, Type::Integer, true, Scope::default())]
        )),
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(5))));
}

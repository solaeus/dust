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
                return_type: Type::Integer,
            },
            vec![
                (Instruction::load_constant(0, 0, false), Span(12, 14)),
                (
                    Instruction::subtract(0, Operand::Register(0), Operand::Constant(2)),
                    Span(18, 20)
                ),
                (Instruction::get_local(1, 0), Span(24, 25)),
                (Instruction::r#return(true), Span(25, 25)),
            ],
            vec![
                ConcreteValue::Integer(42),
                ConcreteValue::string("x"),
                ConcreteValue::Integer(2)
            ],
            vec![Local::new(1, 0, true, Scope::default())]
        )),
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(40))));
}

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
                return_type: Type::Integer,
            },
            vec![
                (Instruction::load_constant(0, 0, false), Span(12, 13)),
                (
                    Instruction::multiply(0, Operand::Register(0), Operand::Constant(2)),
                    Span(17, 19)
                ),
                (Instruction::get_local(1, 0), Span(22, 23)),
                (Instruction::r#return(true), Span(23, 23))
            ],
            vec![
                ConcreteValue::Integer(2),
                ConcreteValue::string("a"),
                ConcreteValue::Integer(3)
            ],
            vec![Local::new(1, 0, true, Scope::default())]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(6))));
}

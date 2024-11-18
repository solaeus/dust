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
                (Instruction::load_constant(0, 0, false), Span(12, 13)),
                (
                    *Instruction::less(true, 0, 2).set_c_is_constant(),
                    Span(23, 24)
                ),
                (Instruction::jump(2, true), Span(41, 42)),
                (*Instruction::add(0, 0, 3).set_c_is_constant(), Span(35, 36)),
                (Instruction::jump(3, false), Span(41, 42)),
                (Instruction::get_local(1, 0), Span(41, 42)),
                (Instruction::r#return(true), Span(42, 42)),
            ],
            vec![
                ConcreteValue::Integer(0),
                ConcreteValue::string("x"),
                ConcreteValue::Integer(5),
                ConcreteValue::Integer(1),
            ],
            vec![Local::new(1, Type::Integer, true, Scope::default(), 0)]
        )),
    );

    assert_eq!(run_source(source), Ok(Some(ConcreteValue::Integer(5))));
}

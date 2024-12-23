use dust_lang::*;

#[test]
fn constant() {
    let source = "42";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::Integer)
            },
            vec![
                (
                    Instruction::load_constant(Destination::Register(0), 0, false),
                    Type::Integer,
                    Span(0, 2)
                ),
                (Instruction::r#return(true), Type::None, Span(2, 2))
            ],
            vec![ConcreteValue::Integer(42)],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(42))));
}

#[test]
fn empty() {
    let source = "";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::None)
            },
            vec![(Instruction::r#return(false), Type::None, Span(0, 0))],
            vec![],
            vec![]
        ))
    );
    assert_eq!(run(source), Ok(None));
}

#[test]
fn parentheses_precedence() {
    let source = "(1 + 2) * 3";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::Integer)
            },
            vec![
                (
                    Instruction::add(
                        Destination::Register(0),
                        Argument::Constant(0),
                        Argument::Constant(1)
                    ),
                    Type::Integer,
                    Span(3, 4)
                ),
                (
                    Instruction::multiply(
                        Destination::Register(1),
                        Argument::Register(0),
                        Argument::Constant(2)
                    ),
                    Type::Integer,
                    Span(8, 9)
                ),
                (Instruction::r#return(true), Type::None, Span(11, 11)),
            ],
            vec![
                ConcreteValue::Integer(1),
                ConcreteValue::Integer(2),
                ConcreteValue::Integer(3)
            ],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(9))));
}

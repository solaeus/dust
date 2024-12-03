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
                    Span(0, 2)
                ),
                (Instruction::r#return(true), Span(2, 2))
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
            vec![(Instruction::r#return(false), Span(0, 0))],
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
                    Span(3, 4)
                ),
                (
                    Instruction::multiply(
                        Destination::Register(1),
                        Argument::Register(0),
                        Argument::Constant(2)
                    ),
                    Span(8, 9)
                ),
                (Instruction::r#return(true), Span(11, 11)),
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

#[test]
fn math_operator_precedence() {
    let source = "1 + 2 - 3 * 4 / 5";

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
                    Instruction::add(
                        Destination::Register(0),
                        Argument::Constant(0),
                        Argument::Constant(1)
                    ),
                    Span(2, 3)
                ),
                (
                    Instruction::multiply(
                        Destination::Register(1),
                        Argument::Constant(2),
                        Argument::Constant(3)
                    ),
                    Span(10, 11)
                ),
                (
                    Instruction::divide(
                        Destination::Register(2),
                        Argument::Register(1),
                        Argument::Constant(4)
                    ),
                    Span(14, 15)
                ),
                (
                    Instruction::subtract(
                        Destination::Register(3),
                        Argument::Register(0),
                        Argument::Register(2)
                    ),
                    Span(6, 7)
                ),
                (Instruction::r#return(true), Span(17, 17)),
            ],
            vec![
                ConcreteValue::Integer(1),
                ConcreteValue::Integer(2),
                ConcreteValue::Integer(3),
                ConcreteValue::Integer(4),
                ConcreteValue::Integer(5),
            ],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::Integer(1))));
}

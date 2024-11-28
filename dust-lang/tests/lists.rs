use dust_lang::*;

#[test]
fn empty_list() {
    let source = "[]";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::List(Box::new(Type::Any))),
            },
            vec![
                (
                    Instruction::load_list(0, 0),
                    Type::List(Box::new(Type::Any)),
                    Span(0, 2)
                ),
                (Instruction::r#return(true), Type::None, Span(2, 2)),
            ],
            vec![],
            vec![]
        )),
    );

    assert_eq!(run(source), Ok(Some(ConcreteValue::list([]))));
}

#[test]
fn list() {
    let source = "[1, 2, 3]";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::List(Box::new(Type::Integer))),
            },
            vec![
                (
                    Instruction::load_constant(0, 0, false),
                    Type::Integer,
                    Span(1, 2)
                ),
                (
                    Instruction::load_constant(1, 1, false),
                    Type::Integer,
                    Span(4, 5)
                ),
                (
                    Instruction::load_constant(2, 2, false),
                    Type::Integer,
                    Span(7, 8)
                ),
                (
                    Instruction::load_list(3, 0),
                    Type::List(Box::new(Type::Integer)),
                    Span(0, 9)
                ),
                (Instruction::r#return(true), Type::None, Span(9, 9)),
            ],
            vec![
                ConcreteValue::Integer(1),
                ConcreteValue::Integer(2),
                ConcreteValue::Integer(3)
            ],
            vec![]
        )),
    );

    assert_eq!(
        run(source),
        Ok(Some(ConcreteValue::list([
            ConcreteValue::Integer(1),
            ConcreteValue::Integer(2),
            ConcreteValue::Integer(3)
        ])))
    );
}

#[test]
fn list_with_complex_expression() {
    let source = "[1, 2 + 3 - 4 * 5]";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::List(Box::new(Type::Integer))),
            },
            vec![
                (
                    Instruction::load_constant(0, 0, false),
                    Type::Integer,
                    Span(1, 2)
                ),
                (
                    Instruction::add(1, Argument::Constant(1), Argument::Constant(2)),
                    Type::Integer,
                    Span(6, 7)
                ),
                (
                    Instruction::multiply(2, Argument::Constant(3), Argument::Constant(4)),
                    Type::Integer,
                    Span(14, 15)
                ),
                (
                    Instruction::subtract(3, Argument::Register(1), Argument::Register(2)),
                    Type::Integer,
                    Span(10, 11)
                ),
                (Instruction::close(1, 3), Type::None, Span(17, 18)),
                (
                    Instruction::load_list(4, 0),
                    Type::List(Box::new(Type::Integer)),
                    Span(0, 18)
                ),
                (Instruction::r#return(true), Type::None, Span(18, 18)),
            ],
            vec![
                ConcreteValue::Integer(1),
                ConcreteValue::Integer(2),
                ConcreteValue::Integer(3),
                ConcreteValue::Integer(4),
                ConcreteValue::Integer(5)
            ],
            vec![]
        )),
    );

    assert_eq!(
        run(source),
        Ok(Some(ConcreteValue::list([
            ConcreteValue::Integer(1),
            ConcreteValue::Integer(-15)
        ])))
    );
}

#[test]
fn list_with_simple_expression() {
    let source = "[1, 2 + 3, 4]";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Box::new(Type::List(Box::new(Type::Integer))),
            },
            vec![
                (
                    Instruction::load_constant(0, 0, false),
                    Type::Integer,
                    Span(1, 2)
                ),
                (
                    Instruction::add(1, Argument::Constant(1), Argument::Constant(2)),
                    Type::Integer,
                    Span(6, 7)
                ),
                (
                    Instruction::load_constant(2, 3, false),
                    Type::Integer,
                    Span(11, 12)
                ),
                (
                    Instruction::load_list(3, 0),
                    Type::List(Box::new(Type::Integer)),
                    Span(0, 13)
                ),
                (Instruction::r#return(true), Type::None, Span(13, 13)),
            ],
            vec![
                ConcreteValue::Integer(1),
                ConcreteValue::Integer(2),
                ConcreteValue::Integer(3),
                ConcreteValue::Integer(4),
            ],
            vec![]
        )),
    );

    assert_eq!(
        run(source),
        Ok(Some(ConcreteValue::list([
            ConcreteValue::Integer(1),
            ConcreteValue::Integer(5),
            ConcreteValue::Integer(4),
        ])))
    );
}

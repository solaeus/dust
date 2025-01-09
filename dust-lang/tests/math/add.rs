use dust_lang::*;
use smallvec::smallvec;

#[test]
fn add_bytes() {
    let source = "0xfe + 0x01";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Type::Byte,
            },
            smallvec![
                Instruction::add(0, Argument::Constant(0), Argument::Constant(1)),
                Instruction::r#return(true),
            ],
            smallvec![Span(5, 6), Span(11, 11),],
            smallvec![Value::byte(0xfe), Value::byte(0x01)],
            smallvec![],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::byte(0xff))));
}

#[test]
fn add_bytes_saturate() {
    let source = "0xff + 0x01";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Type::Byte,
            },
            smallvec![
                Instruction::add(0, Argument::Constant(0), Argument::Constant(1)),
                Instruction::r#return(true),
            ],
            smallvec![Span(5, 6), Span(11, 11)],
            smallvec![Value::byte(0xff), Value::byte(0x01)],
            smallvec![],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::byte(0xff))));
}

#[test]
fn add_characters() {
    let source = "'a' + 'b'";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Type::String,
            },
            smallvec![
                Instruction::add(0, Argument::Constant(0), Argument::Constant(1)),
                Instruction::r#return(true)
            ],
            smallvec![Span(4, 5), Span(11, 11)],
            smallvec![Value::character('a'), Value::character('b')],
            smallvec![],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::string("ab"))));
}

#[test]
fn add_character_and_string() {
    let source = "'a' + \"b\"";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Type::String,
            },
            smallvec![
                Instruction::add(0, Argument::Constant(0), Argument::Constant(1)),
                Instruction::r#return(true),
            ],
            smallvec![Span(4, 5), Span(9, 9),],
            smallvec![Value::character('a'), Value::string("b")],
            smallvec![],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::string("ab"))));
}

#[test]
fn add_floats() {
    let source = "1.0 + 2.0";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Type::Float,
            },
            smallvec![
                Instruction::add(0, Argument::Constant(0), Argument::Constant(1)),
                Instruction::r#return(true),
            ],
            smallvec![Span(4, 5), Span(9, 9),],
            smallvec![Value::float(1.0), Value::float(2.0)],
            smallvec![],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::float(3.0))));
}

#[test]
fn add_floats_saturatate() {
    let source = "1.7976931348623157E+308 + 0.00000001";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Type::Float,
            },
            smallvec![
                Instruction::add(0, Argument::Constant(0), Argument::Constant(1)),
                Instruction::r#return(true),
            ],
            smallvec![Span(24, 25), Span(36, 36),],
            smallvec![Value::float(f64::MAX), Value::float(0.00000001)],
            smallvec![],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::float(f64::MAX))));
}

#[test]
fn add_integers() {
    let source = "1 + 2";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Type::Integer,
            },
            smallvec![
                Instruction::add(0, Argument::Constant(0), Argument::Constant(1)),
                Instruction::r#return(true)
            ],
            smallvec![Span(2, 3), Span(5, 5),],
            smallvec![Value::integer(1), Value::integer(2)],
            smallvec![],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::integer(3))));
}

#[test]
fn add_integers_saturate() {
    let source = "9223372036854775807 + 1";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Type::Integer,
            },
            smallvec![
                Instruction::add(0, Argument::Constant(0), Argument::Constant(1)),
                Instruction::r#return(true)
            ],
            smallvec![Span(20, 21), Span(23, 23),],
            smallvec![Value::integer(i64::MAX), Value::integer(1)],
            smallvec![],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::integer(i64::MAX))));
}

#[test]
fn add_strings() {
    let source = "\"Hello, \" + \"world!\"";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Type::String,
            },
            smallvec![
                Instruction::add(0, Argument::Constant(0), Argument::Constant(1)),
                Instruction::r#return(true),
            ],
            smallvec![Span(10, 11), Span(20, 20)],
            smallvec![Value::string("Hello, "), Value::string("world!")],
            smallvec![],
            vec![]
        ))
    );
}

#[test]
fn add_string_and_character() {
    let source = "\"a\" + 'b'";

    assert_eq!(
        compile(source),
        Ok(Chunk::with_data(
            None,
            FunctionType {
                type_parameters: None,
                value_parameters: None,
                return_type: Type::String,
            },
            smallvec![
                Instruction::add(0, Argument::Constant(0), Argument::Constant(1)),
                Instruction::r#return(true),
            ],
            smallvec![Span(4, 5), Span(9, 9),],
            smallvec![Value::string("a"), Value::character('b')],
            smallvec![],
            vec![]
        ))
    );

    assert_eq!(run(source), Ok(Some(Value::string("ab"))));
}

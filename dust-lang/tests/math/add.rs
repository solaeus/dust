use dust_lang::{
    compile, instruction::TypeCode, run, Chunk, DustString, FunctionType, Instruction, Operand,
    Span, Type, Value,
};

#[test]
fn add_bytes() {
    let source = "0x28 + 0x02";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Byte),
        instructions: vec![
            Instruction::load_encoded(0, 40, TypeCode::BYTE, false),
            Instruction::load_encoded(1, 2, TypeCode::BYTE, false),
            Instruction::add(
                2,
                Operand::Register(0, TypeCode::BYTE),
                Operand::Register(1, TypeCode::BYTE),
            ),
            Instruction::r#return(true, 2, TypeCode::BYTE),
        ],
        positions: vec![Span(0, 4), Span(7, 11), Span(0, 11), Span(11, 11)],
        ..Chunk::default()
    };
    let return_value = Some(Value::byte(0x2A));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn add_many_bytes() {
    let source = "0x28 + 0x02 + 0x02 + 0x02";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Byte),
        instructions: vec![
            Instruction::load_encoded(0, 40, TypeCode::BYTE, false),
            Instruction::load_encoded(1, 2, TypeCode::BYTE, false),
            Instruction::add(
                2,
                Operand::Register(0, TypeCode::BYTE),
                Operand::Register(1, TypeCode::BYTE),
            ),
            Instruction::load_encoded(3, 2, TypeCode::BYTE, false),
            Instruction::add(
                4,
                Operand::Register(2, TypeCode::BYTE),
                Operand::Register(3, TypeCode::BYTE),
            ),
            Instruction::load_encoded(5, 2, TypeCode::BYTE, false),
            Instruction::add(
                6,
                Operand::Register(4, TypeCode::BYTE),
                Operand::Register(5, TypeCode::BYTE),
            ),
            Instruction::r#return(true, 6, TypeCode::BYTE),
        ],
        positions: vec![
            Span(0, 4),
            Span(7, 11),
            Span(0, 11),
            Span(14, 18),
            Span(0, 18),
            Span(21, 25),
            Span(0, 25),
            Span(25, 25),
        ],
        ..Chunk::default()
    };
    let return_value = Some(Value::byte(46));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn add_characters() {
    let source = "'a' + 'b'";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::String),
        instructions: vec![
            Instruction::add(
                0,
                Operand::Constant(0, TypeCode::CHARACTER),
                Operand::Constant(1, TypeCode::CHARACTER),
            ),
            Instruction::r#return(true, 0, TypeCode::STRING),
        ],
        positions: vec![Span(0, 9), Span(9, 9)],
        character_constants: vec!['a', 'b'],
        ..Chunk::default()
    };
    let return_value = Some(Value::string("ab"));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn add_many_characters() {
    let source = "'a' + 'b' + 'c' + 'd'";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::String),
        instructions: vec![
            Instruction::add(
                0,
                Operand::Constant(0, TypeCode::CHARACTER),
                Operand::Constant(1, TypeCode::CHARACTER),
            ),
            Instruction::add(
                1,
                Operand::Register(0, TypeCode::STRING),
                Operand::Constant(2, TypeCode::CHARACTER),
            ),
            Instruction::add(
                2,
                Operand::Register(1, TypeCode::STRING),
                Operand::Constant(3, TypeCode::CHARACTER),
            ),
            Instruction::r#return(true, 2, TypeCode::STRING),
        ],
        positions: vec![Span(0, 9), Span(0, 15), Span(0, 21), Span(21, 21)],
        character_constants: vec!['a', 'b', 'c', 'd'],
        ..Chunk::default()
    };
    let return_value = Some(Value::string("abcd"));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn add_floats() {
    let source = "2.40 + 40.02";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Float),
        instructions: vec![
            Instruction::add(
                0,
                Operand::Constant(0, TypeCode::FLOAT),
                Operand::Constant(1, TypeCode::FLOAT),
            ),
            Instruction::r#return(true, 0, TypeCode::FLOAT),
        ],
        positions: vec![Span(0, 12), Span(12, 12)],
        float_constants: vec![2.40, 40.02],
        ..Chunk::default()
    };
    let return_value = Some(Value::float(42.42));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn add_many_floats() {
    let source = "2.40 + 40.02 + 2.40 + 40.02";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Float),
        instructions: vec![
            Instruction::add(
                0,
                Operand::Constant(0, TypeCode::FLOAT),
                Operand::Constant(1, TypeCode::FLOAT),
            ),
            Instruction::add(
                1,
                Operand::Register(0, TypeCode::FLOAT),
                Operand::Constant(0, TypeCode::FLOAT),
            ),
            Instruction::add(
                2,
                Operand::Register(1, TypeCode::FLOAT),
                Operand::Constant(1, TypeCode::FLOAT),
            ),
            Instruction::r#return(true, 2, TypeCode::FLOAT),
        ],
        positions: vec![Span(0, 12), Span(0, 19), Span(0, 27), Span(27, 27)],
        float_constants: vec![2.40, 40.02],
        ..Chunk::default()
    };
    let return_value = Some(Value::float(84.84));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn add_integers() {
    let source = "40 + 2";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Integer),
        instructions: vec![
            Instruction::add(
                0,
                Operand::Constant(0, TypeCode::INTEGER),
                Operand::Constant(1, TypeCode::INTEGER),
            ),
            Instruction::r#return(true, 0, TypeCode::INTEGER),
        ],
        positions: vec![Span(0, 6), Span(6, 6)],
        integer_constants: vec![40, 2],
        ..Chunk::default()
    };
    let return_value = Some(Value::integer(42));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn add_many_integers() {
    let source = "40 + 2 + 40 + 2";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Integer),
        instructions: vec![
            Instruction::add(
                0,
                Operand::Constant(0, TypeCode::INTEGER),
                Operand::Constant(1, TypeCode::INTEGER),
            ),
            Instruction::add(
                1,
                Operand::Register(0, TypeCode::INTEGER),
                Operand::Constant(0, TypeCode::INTEGER),
            ),
            Instruction::add(
                2,
                Operand::Register(1, TypeCode::INTEGER),
                Operand::Constant(1, TypeCode::INTEGER),
            ),
            Instruction::r#return(true, 2, TypeCode::INTEGER),
        ],
        positions: vec![Span(0, 6), Span(0, 11), Span(0, 15), Span(15, 15)],
        integer_constants: vec![40, 2],
        ..Chunk::default()
    };
    let return_value = Some(Value::integer(84));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn add_strings() {
    let source = "\"Hello, \" + \"World!\"";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::String),
        instructions: vec![
            Instruction::add(
                0,
                Operand::Constant(0, TypeCode::STRING),
                Operand::Constant(1, TypeCode::STRING),
            ),
            Instruction::r#return(true, 0, TypeCode::STRING),
        ],
        positions: vec![Span(0, 20), Span(20, 20)],
        string_constants: vec![DustString::from("Hello, "), DustString::from("World!")],
        ..Chunk::default()
    };
    let return_value = Some(Value::string("Hello, World!"));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn add_many_strings() {
    let source = "\"foo\" + \"bar\" + \"baz\" + \"buzz\"";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::String),
        instructions: vec![
            Instruction::add(
                0,
                Operand::Constant(0, TypeCode::STRING),
                Operand::Constant(1, TypeCode::STRING),
            ),
            Instruction::add(
                1,
                Operand::Register(0, TypeCode::STRING),
                Operand::Constant(2, TypeCode::STRING),
            ),
            Instruction::add(
                2,
                Operand::Register(1, TypeCode::STRING),
                Operand::Constant(3, TypeCode::STRING),
            ),
            Instruction::r#return(true, 2, TypeCode::STRING),
        ],
        positions: vec![Span(0, 13), Span(0, 21), Span(0, 30), Span(30, 30)],
        string_constants: vec![
            DustString::from("foo"),
            DustString::from("bar"),
            DustString::from("baz"),
            DustString::from("buzz"),
        ],
        ..Chunk::default()
    };
    let return_value = Some(Value::string("foobarbazbuzz"));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

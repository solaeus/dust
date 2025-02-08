use dust_lang::{
    Chunk, ConcreteValue, DustString, FunctionType, Instruction, Operand, Span, Type, Value,
    compile, instruction::TypeCode, run,
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
        positions: vec![Span(0, 4), Span(7, 11), Span(5, 6), Span(11, 11)],
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
            Span(5, 6),
            Span(14, 18),
            Span(12, 13),
            Span(21, 25),
            Span(19, 20),
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
        positions: vec![Span(4, 5), Span(9, 9)],
        constants: vec![ConcreteValue::Character('a'), ConcreteValue::Character('b')],
        ..Chunk::default()
    };
    let return_value = Some(Value::string("ab"));

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
        positions: vec![Span(5, 6), Span(12, 12)],
        constants: vec![ConcreteValue::Float(2.40), ConcreteValue::Float(40.02)],
        ..Chunk::default()
    };
    let return_value = Some(Value::float(42.42));

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
        positions: vec![Span(3, 4), Span(6, 6)],
        constants: vec![ConcreteValue::Integer(40), ConcreteValue::Integer(2)],
        ..Chunk::default()
    };
    let return_value = Some(Value::integer(42));

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
        positions: vec![Span(10, 11), Span(20, 20)],
        constants: vec![
            ConcreteValue::String(DustString::from("Hello, ")),
            ConcreteValue::String(DustString::from("World!")),
        ],
        ..Chunk::default()
    };
    let return_value = Some(Value::string("Hello, World!"));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

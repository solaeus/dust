use dust_lang::{
    Chunk, ConcreteValue, DustString, FunctionType, Instruction, Operand, Span, Type, Value,
    compile, instruction::TypeCode, run,
};

#[test]
fn greater_bytes() {
    let source = "0x0A > 0x03";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::load_encoded(0, 0x0A, TypeCode::BYTE, false),
            Instruction::load_encoded(1, 0x03, TypeCode::BYTE, false),
            Instruction::less_equal(
                false,
                Operand::Register(0, TypeCode::BYTE),
                Operand::Register(1, TypeCode::BYTE),
            ),
            Instruction::jump(1, true),
            Instruction::load_encoded(0, true as u8, TypeCode::BOOLEAN, true),
            Instruction::load_encoded(0, false as u8, TypeCode::BOOLEAN, false),
            Instruction::r#return(true, 0, TypeCode::BOOLEAN),
        ],
        positions: vec![
            Span(0, 4),
            Span(7, 11),
            Span(0, 11),
            Span(0, 11),
            Span(0, 11),
            Span(0, 11),
            Span(11, 11),
        ],
        ..Chunk::default()
    };
    let return_value = Some(Value::boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn greater_characters() {
    let source = "'a' > 'b'";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::less_equal(
                false,
                Operand::Constant(0, TypeCode::CHARACTER),
                Operand::Constant(1, TypeCode::CHARACTER),
            ),
            Instruction::jump(1, true),
            Instruction::load_encoded(0, true as u8, TypeCode::BOOLEAN, true),
            Instruction::load_encoded(0, false as u8, TypeCode::BOOLEAN, false),
            Instruction::r#return(true, 0, TypeCode::BOOLEAN),
        ],
        positions: vec![Span(0, 9), Span(0, 9), Span(0, 9), Span(0, 9), Span(9, 9)],
        constants: vec![ConcreteValue::Character('a'), ConcreteValue::Character('b')],
        ..Chunk::default()
    };
    let return_value = Some(Value::boolean(false));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn greater_floats() {
    let source = "10.0 > 3.0";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::less_equal(
                false,
                Operand::Constant(0, TypeCode::FLOAT),
                Operand::Constant(1, TypeCode::FLOAT),
            ),
            Instruction::jump(1, true),
            Instruction::load_encoded(0, true as u8, TypeCode::BOOLEAN, true),
            Instruction::load_encoded(0, false as u8, TypeCode::BOOLEAN, false),
            Instruction::r#return(true, 0, TypeCode::BOOLEAN),
        ],
        positions: vec![
            Span(0, 10),
            Span(0, 10),
            Span(0, 10),
            Span(0, 10),
            Span(10, 10),
        ],
        constants: vec![ConcreteValue::Float(10.0), ConcreteValue::Float(3.0)],
        ..Chunk::default()
    };
    let return_value = Some(Value::boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn greater_integers() {
    let source = "10 > 3";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::less_equal(
                false,
                Operand::Constant(0, TypeCode::INTEGER),
                Operand::Constant(1, TypeCode::INTEGER),
            ),
            Instruction::jump(1, true),
            Instruction::load_encoded(0, true as u8, TypeCode::BOOLEAN, true),
            Instruction::load_encoded(0, false as u8, TypeCode::BOOLEAN, false),
            Instruction::r#return(true, 0, TypeCode::BOOLEAN),
        ],
        positions: vec![Span(0, 6), Span(0, 6), Span(0, 6), Span(0, 6), Span(6, 6)],
        constants: vec![ConcreteValue::Integer(10), ConcreteValue::Integer(3)],
        ..Chunk::default()
    };
    let return_value = Some(Value::boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn greater_strings() {
    let source = "\"abc\" > \"def\"";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::less_equal(
                false,
                Operand::Constant(0, TypeCode::STRING),
                Operand::Constant(1, TypeCode::STRING),
            ),
            Instruction::jump(1, true),
            Instruction::load_encoded(0, true as u8, TypeCode::BOOLEAN, true),
            Instruction::load_encoded(0, false as u8, TypeCode::BOOLEAN, false),
            Instruction::r#return(true, 0, TypeCode::BOOLEAN),
        ],
        positions: vec![
            Span(0, 13),
            Span(0, 13),
            Span(0, 13),
            Span(0, 13),
            Span(13, 13),
        ],
        constants: vec![
            ConcreteValue::String(DustString::from("abc")),
            ConcreteValue::String(DustString::from("def")),
        ],
        ..Chunk::default()
    };
    let return_value = Some(Value::boolean(false));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

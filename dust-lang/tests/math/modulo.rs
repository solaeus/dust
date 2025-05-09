use dust_lang::{
    Address, Chunk, FunctionType, Instruction, Span, Type, Value, compile, inst
   ruction::TypeCode, run,
};

#[test]
fn modulo_bytes() {
    let source = "0x0A % 0x03";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Byte),
        instructions: vec![
            Instruction::load_encoded(0, 10, TypeCode::BYTE, false),
            Instruction::load_encoded(1, 3, TypeCode::BYTE, false),
            Instruction::modulo(
                2,
                Address::Register(0, TypeCode::BYTE),
                Address::Register(1, TypeCode::BYTE),
            ),
            Instruction::r#return(true, 2, TypeCode::BYTE),
        ],
        positions: vec![Span(0, 4), Span(7, 11), Span(0, 11), Span(11, 11)],
        ..Chunk::default()
    };
    let return_value = Some(Value::byte(0x01));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn modulo_many_bytes() {
    let source = "0x0F % 0x04 % 0x02";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Byte),
        instructions: vec![
            Instruction::load_encoded(0, 15, TypeCode::BYTE, false),
            Instruction::load_encoded(1, 4, TypeCode::BYTE, false),
            Instruction::modulo(
                2,
                Address::Register(0, TypeCode::BYTE),
                Address::Register(1, TypeCode::BYTE),
            ),
            Instruction::load_encoded(3, 2, TypeCode::BYTE, false),
            Instruction::modulo(
                4,
                Address::Register(2, TypeCode::BYTE),
                Address::Register(3, TypeCode::BYTE),
            ),
            Instruction::r#return(true, 4, TypeCode::BYTE),
        ],
        positions: vec![
            Span(0, 4),
            Span(7, 11),
            Span(0, 11),
            Span(14, 18),
            Span(0, 18),
            Span(18, 18),
        ],
        ..Chunk::default()
    };
    let return_value = Some(Value::byte(0x01));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn modulo_integers() {
    let source = "10 % 3";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Integer),
        instructions: vec![
            Instruction::modulo(
                0,
                Address::Constant(0, TypeCode::INTEGER),
                Address::Constant(1, TypeCode::INTEGER),
            ),
            Instruction::r#return(true, 0, TypeCode::INTEGER),
        ],
        positions: vec![Span(0, 6), Span(6, 6)],
        integer_constants: vec![10, 3],
        ..Chunk::default()
    };
    let return_value = Some(Value::integer(1));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn modulo_many_integers() {
    let source = "10 % 5 % 3";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Integer),
        instructions: vec![
            Instruction::modulo(
                0,
                Address::Constant(0, TypeCode::INTEGER),
                Address::Constant(1, TypeCode::INTEGER),
            ),
            Instruction::modulo(
                1,
                Address::Register(0, TypeCode::INTEGER),
                Address::Constant(2, TypeCode::INTEGER),
            ),
            Instruction::r#return(true, 1, TypeCode::INTEGER),
        ],
        positions: vec![Span(0, 6), Span(0, 10), Span(10, 10)],
        integer_constants: vec![10, 5, 3],
        ..Chunk::default()
    };
    let return_value = Some(Value::integer(0));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

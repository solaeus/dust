use dust_lang::{
    Chunk, FunctionType, Instruction, Span, Type, Value, compile, instruction::TypeCode, run,
};

#[test]
fn not_true() {
    let source = "!true";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::load_encoded(0, true as u8, TypeCode::BOOLEAN, false),
            Instruction::not(1, dust_lang::Operand::Register(0, TypeCode::BOOLEAN)),
            Instruction::r#return(true, 1, TypeCode::BOOLEAN),
        ],
        positions: vec![Span(1, 5), Span(0, 1), Span(5, 5)],
        ..Chunk::default()
    };
    let return_value = Some(Value::boolean(false));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn not_false() {
    let source = "!false";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::load_encoded(0, false as u8, TypeCode::BOOLEAN, false),
            Instruction::not(1, dust_lang::Operand::Register(0, TypeCode::BOOLEAN)),
            Instruction::r#return(true, 1, TypeCode::BOOLEAN),
        ],
        positions: vec![Span(1, 6), Span(0, 1), Span(6, 6)],
        ..Chunk::default()
    };
    let return_value = Some(Value::boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn not_not_true() {
    let source = "!!true";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::load_encoded(0, true as u8, TypeCode::BOOLEAN, false),
            Instruction::not(1, dust_lang::Operand::Register(0, TypeCode::BOOLEAN)),
            Instruction::not(2, dust_lang::Operand::Register(1, TypeCode::BOOLEAN)),
            Instruction::r#return(true, 2, TypeCode::BOOLEAN),
        ],
        positions: vec![Span(2, 6), Span(1, 2), Span(0, 1), Span(6, 6)],
        ..Chunk::default()
    };
    let return_value = Some(Value::boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn not_not_false() {
    let source = "!!false";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::load_encoded(0, false as u8, TypeCode::BOOLEAN, false),
            Instruction::not(1, dust_lang::Operand::Register(0, TypeCode::BOOLEAN)),
            Instruction::not(2, dust_lang::Operand::Register(1, TypeCode::BOOLEAN)),
            Instruction::r#return(true, 2, TypeCode::BOOLEAN),
        ],
        positions: vec![Span(2, 7), Span(1, 2), Span(0, 1), Span(7, 7)],
        ..Chunk::default()
    };
    let return_value = Some(Value::boolean(false));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

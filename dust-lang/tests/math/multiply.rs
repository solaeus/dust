use dust_lang::{
    Chunk, ConcreteValue, FunctionType, Instruction, Operand, Span, Type, Value, compile,
    instruction::TypeCode, run,
};

#[test]
fn multiply_bytes() {
    let source = "0x0A * 0x02";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Byte),
        instructions: vec![
            Instruction::load_encoded(0, 10, TypeCode::BYTE, false),
            Instruction::load_encoded(1, 2, TypeCode::BYTE, false),
            Instruction::multiply(
                2,
                Operand::Register(0, TypeCode::BYTE),
                Operand::Register(1, TypeCode::BYTE),
            ),
            Instruction::r#return(true, 2, TypeCode::BYTE),
        ],
        positions: vec![Span(0, 4), Span(7, 11), Span(0, 11), Span(11, 11)],
        ..Chunk::default()
    };
    let return_value = Some(Value::byte(0x14));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn multiply_many_bytes() {
    let source = "0x0A * 0x02 * 0x02";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Byte),
        instructions: vec![
            Instruction::load_encoded(0, 10, TypeCode::BYTE, false),
            Instruction::load_encoded(1, 2, TypeCode::BYTE, false),
            Instruction::multiply(
                2,
                Operand::Register(0, TypeCode::BYTE),
                Operand::Register(1, TypeCode::BYTE),
            ),
            Instruction::load_encoded(3, 2, TypeCode::BYTE, false),
            Instruction::multiply(
                4,
                Operand::Register(2, TypeCode::BYTE),
                Operand::Register(3, TypeCode::BYTE),
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
    let return_value = Some(Value::byte(0x28));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn multiply_floats() {
    let source = "0.5 * 2.0";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Float),
        instructions: vec![
            Instruction::multiply(
                0,
                Operand::Constant(0, TypeCode::FLOAT),
                Operand::Constant(1, TypeCode::FLOAT),
            ),
            Instruction::r#return(true, 0, TypeCode::FLOAT),
        ],
        positions: vec![Span(0, 9), Span(9, 9)],
        constants: vec![ConcreteValue::Float(0.5), ConcreteValue::Float(2.0)],
        ..Chunk::default()
    };
    let return_value = Some(Value::float(1.0));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn multiply_many_floats() {
    let source = "0.5 * 2.0 * 0.5";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Float),
        instructions: vec![
            Instruction::multiply(
                0,
                Operand::Constant(0, TypeCode::FLOAT),
                Operand::Constant(1, TypeCode::FLOAT),
            ),
            Instruction::multiply(
                1,
                Operand::Register(0, TypeCode::FLOAT),
                Operand::Constant(0, TypeCode::FLOAT),
            ),
            Instruction::r#return(true, 1, TypeCode::FLOAT),
        ],
        positions: vec![Span(0, 9), Span(0, 15), Span(15, 15)],
        constants: vec![ConcreteValue::Float(0.5), ConcreteValue::Float(2.0)],
        ..Chunk::default()
    };
    let return_value = Some(Value::float(0.5));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn multiply_integers() {
    let source = "10 * 5";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Integer),
        instructions: vec![
            Instruction::multiply(
                0,
                Operand::Constant(0, TypeCode::INTEGER),
                Operand::Constant(1, TypeCode::INTEGER),
            ),
            Instruction::r#return(true, 0, TypeCode::INTEGER),
        ],
        positions: vec![Span(0, 6), Span(6, 6)],
        constants: vec![ConcreteValue::Integer(10), ConcreteValue::Integer(5)],
        ..Chunk::default()
    };
    let return_value = Some(Value::integer(50));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn multiply_many_integers() {
    let source = "10 * 5 * 2";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Integer),
        instructions: vec![
            Instruction::multiply(
                0,
                Operand::Constant(0, TypeCode::INTEGER),
                Operand::Constant(1, TypeCode::INTEGER),
            ),
            Instruction::multiply(
                1,
                Operand::Register(0, TypeCode::INTEGER),
                Operand::Constant(2, TypeCode::INTEGER),
            ),
            Instruction::r#return(true, 1, TypeCode::INTEGER),
        ],
        positions: vec![Span(0, 6), Span(0, 10), Span(10, 10)],
        constants: vec![
            ConcreteValue::Integer(10),
            ConcreteValue::Integer(5),
            ConcreteValue::Integer(2),
        ],
        ..Chunk::default()
    };
    let return_value = Some(Value::integer(100));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

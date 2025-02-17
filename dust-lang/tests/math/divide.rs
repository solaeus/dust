use dust_lang::{
    compile, instruction::TypeCode, run, Chunk, FunctionType, Instruction, Operand, Span, Type,
    Value,
};

#[test]
fn divide_bytes() {
    let source = "0x0A / 0x02";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Byte),
        instructions: vec![
            Instruction::load_encoded(0, 10, TypeCode::BYTE, false),
            Instruction::load_encoded(1, 2, TypeCode::BYTE, false),
            Instruction::divide(
                2,
                Operand::Register(0, TypeCode::BYTE),
                Operand::Register(1, TypeCode::BYTE),
            ),
            Instruction::r#return(true, 2, TypeCode::BYTE),
        ],
        positions: vec![Span(0, 4), Span(7, 11), Span(0, 11), Span(11, 11)],
        ..Chunk::default()
    };
    let return_value = Some(Value::byte(0x05));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn divide_many_bytes() {
    let source = "0x0A / 0x02 / 0x02";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Byte),
        instructions: vec![
            Instruction::load_encoded(0, 10, TypeCode::BYTE, false),
            Instruction::load_encoded(1, 2, TypeCode::BYTE, false),
            Instruction::divide(
                2,
                Operand::Register(0, TypeCode::BYTE),
                Operand::Register(1, TypeCode::BYTE),
            ),
            Instruction::load_encoded(3, 2, TypeCode::BYTE, false),
            Instruction::divide(
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
    let return_value = Some(Value::byte(0x02));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn divide_floats() {
    let source = "1.0 / 0.25";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Float),
        instructions: vec![
            Instruction::divide(
                0,
                Operand::Constant(0, TypeCode::FLOAT),
                Operand::Constant(1, TypeCode::FLOAT),
            ),
            Instruction::r#return(true, 0, TypeCode::FLOAT),
        ],
        positions: vec![Span(0, 10), Span(10, 10)],
        float_constants: vec![1.0, 0.25],
        ..Chunk::default()
    };
    let return_value = Some(Value::float(4.0));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn divide_many_floats() {
    let source = "1.0 / 0.25 / 0.5";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Float),
        instructions: vec![
            Instruction::divide(
                0,
                Operand::Constant(0, TypeCode::FLOAT),
                Operand::Constant(1, TypeCode::FLOAT),
            ),
            Instruction::divide(
                1,
                Operand::Register(0, TypeCode::FLOAT),
                Operand::Constant(2, TypeCode::FLOAT),
            ),
            Instruction::r#return(true, 1, TypeCode::FLOAT),
        ],
        positions: vec![Span(0, 10), Span(0, 16), Span(16, 16)],
        float_constants: vec![1.0, 0.25, 0.5],
        ..Chunk::default()
    };
    let return_value = Some(Value::float(8.0));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn divide_integers() {
    let source = "10 / 2";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Integer),
        instructions: vec![
            Instruction::divide(
                0,
                Operand::Constant(0, TypeCode::INTEGER),
                Operand::Constant(1, TypeCode::INTEGER),
            ),
            Instruction::r#return(true, 0, TypeCode::INTEGER),
        ],
        positions: vec![Span(0, 6), Span(6, 6)],
        integer_constants: vec![10, 2],
        ..Chunk::default()
    };
    let return_value = Some(Value::integer(5));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn divide_many_integers() {
    let source = "10 / 2 / 2";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Integer),
        instructions: vec![
            Instruction::divide(
                0,
                Operand::Constant(0, TypeCode::INTEGER),
                Operand::Constant(1, TypeCode::INTEGER),
            ),
            Instruction::divide(
                1,
                Operand::Register(0, TypeCode::INTEGER),
                Operand::Constant(1, TypeCode::INTEGER),
            ),
            Instruction::r#return(true, 1, TypeCode::INTEGER),
        ],
        positions: vec![Span(0, 6), Span(0, 10), Span(10, 10)],
        integer_constants: vec![10, 2],
        ..Chunk::default()
    };
    let return_value = Some(Value::integer(2));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

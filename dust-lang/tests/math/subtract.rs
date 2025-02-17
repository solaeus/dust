use dust_lang::{
    compile, instruction::TypeCode, run, Chunk, FunctionType, Instruction, Operand, Span, Type,
    Value,
};

#[test]
fn subtract_bytes() {
    let source = "0x28 - 0x02";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Byte),
        instructions: vec![
            Instruction::load_encoded(0, 40, TypeCode::BYTE, false),
            Instruction::load_encoded(1, 2, TypeCode::BYTE, false),
            Instruction::subtract(
                2,
                Operand::Register(0, TypeCode::BYTE),
                Operand::Register(1, TypeCode::BYTE),
            ),
            Instruction::r#return(true, 2, TypeCode::BYTE),
        ],
        positions: vec![Span(0, 4), Span(7, 11), Span(0, 11), Span(11, 11)],
        ..Chunk::default()
    };
    let return_value = Some(Value::byte(0x26));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn subtract_many_bytes() {
    let source = "0x28 - 0x02 - 0x02";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Byte),
        instructions: vec![
            Instruction::load_encoded(0, 40, TypeCode::BYTE, false),
            Instruction::load_encoded(1, 2, TypeCode::BYTE, false),
            Instruction::subtract(
                2,
                Operand::Register(0, TypeCode::BYTE),
                Operand::Register(1, TypeCode::BYTE),
            ),
            Instruction::load_encoded(3, 2, TypeCode::BYTE, false),
            Instruction::subtract(
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
    let return_value = Some(Value::byte(0x24));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn add_floats() {
    let source = "0.5 - 0.25";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Float),
        instructions: vec![
            Instruction::subtract(
                0,
                Operand::Constant(0, TypeCode::FLOAT),
                Operand::Constant(1, TypeCode::FLOAT),
            ),
            Instruction::r#return(true, 0, TypeCode::FLOAT),
        ],
        positions: vec![Span(0, 10), Span(10, 10)],
        float_constants: vec![0.5, 0.25],
        ..Chunk::default()
    };
    let return_value = Some(Value::float(0.25));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn add_many_floats() {
    let source = "0.5 - 0.25 - 0.25";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Float),
        instructions: vec![
            Instruction::subtract(
                0,
                Operand::Constant(0, TypeCode::FLOAT),
                Operand::Constant(1, TypeCode::FLOAT),
            ),
            Instruction::subtract(
                1,
                Operand::Register(0, TypeCode::FLOAT),
                Operand::Constant(1, TypeCode::FLOAT),
            ),
            Instruction::r#return(true, 1, TypeCode::FLOAT),
        ],
        positions: vec![Span(0, 10), Span(0, 17), Span(17, 17)],
        float_constants: vec![0.5, 0.25],
        ..Chunk::default()
    };
    let return_value = Some(Value::float(0.0));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn subtract_integers() {
    let source = "10 - 5";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Integer),
        instructions: vec![
            Instruction::subtract(
                0,
                Operand::Constant(0, TypeCode::INTEGER),
                Operand::Constant(1, TypeCode::INTEGER),
            ),
            Instruction::r#return(true, 0, TypeCode::INTEGER),
        ],
        positions: vec![Span(0, 6), Span(6, 6)],
        integer_constants: vec![10, 5],
        ..Chunk::default()
    };
    let return_value = Some(Value::integer(5));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn subtract_many_integers() {
    let source = "10 - 5 - 5";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Integer),
        instructions: vec![
            Instruction::subtract(
                0,
                Operand::Constant(0, TypeCode::INTEGER),
                Operand::Constant(1, TypeCode::INTEGER),
            ),
            Instruction::subtract(
                1,
                Operand::Register(0, TypeCode::INTEGER),
                Operand::Constant(1, TypeCode::INTEGER),
            ),
            Instruction::r#return(true, 1, TypeCode::INTEGER),
        ],
        positions: vec![Span(0, 6), Span(0, 10), Span(10, 10)],
        integer_constants: vec![10, 5],
        ..Chunk::default()
    };
    let return_value = Some(Value::integer(0));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

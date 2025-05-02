use dust_lang::{
    Chunk, FunctionType, Instruction, Operand, Span, Type, TypeCode, Value, compile, run,
};

#[test]
fn if_else() {
    let source = "if 42 == 42 { 42 } else { 0 }";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Integer),
        instructions: vec![
            Instruction::equal(
                true,
                Operand::Constant(0, TypeCode::INTEGER),
                Operand::Constant(0, TypeCode::INTEGER),
            ),
            Instruction::jump(1, true),
            Instruction::load_constant(0, 0, TypeCode::INTEGER, true),
            Instruction::load_constant(0, 1, TypeCode::INTEGER, false),
            Instruction::r#return(true, 0, TypeCode::INTEGER),
        ],
        positions: vec![
            Span(3, 11),
            Span(12, 13),
            Span(14, 16),
            Span(26, 27),
            Span(29, 29),
        ],
        integer_constants: vec![42, 0],
        ..Chunk::default()
    };
    let return_value = Some(Value::integer(42));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn if_else_if_else() {
    let source = "if 42 != 42 { 0 } else if 0 > 0 { 0 } else { 42 }";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Integer),
        instructions: vec![
            Instruction::equal(
                false,
                Operand::Constant(0, TypeCode::INTEGER),
                Operand::Constant(0, TypeCode::INTEGER),
            ),
            Instruction::load_constant(0, 1, TypeCode::INTEGER, false),
            Instruction::less_equal(
                false,
                Operand::Constant(1, TypeCode::INTEGER),
                Operand::Constant(1, TypeCode::INTEGER),
            ),
            Instruction::jump(1, true),
            Instruction::load_constant(1, 1, TypeCode::INTEGER, true),
            Instruction::load_constant(1, 0, TypeCode::INTEGER, false),
            Instruction::r#return(true, 1, TypeCode::INTEGER),
        ],
        positions: vec![
            Span(3, 11),
            Span(14, 15),
            Span(26, 31),
            Span(32, 33),
            Span(34, 35),
            Span(45, 47),
            Span(49, 49),
        ],
        integer_constants: vec![42, 0],
        ..Chunk::default()
    };
    let return_value = Some(Value::integer(42));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn if_else_if_else_if_else() {
    let source = "if 0 > 42 { 1 } else if 0 > 42 { 2 } else if 100 < 50 { 3 } else { 42 }";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Integer),
        instructions: vec![
            Instruction::less_equal(
                false,
                Operand::Constant(0, TypeCode::INTEGER),
                Operand::Constant(1, TypeCode::INTEGER),
            ),
            Instruction::load_constant(0, 2, TypeCode::INTEGER, false),
            Instruction::less_equal(
                false,
                Operand::Constant(0, TypeCode::INTEGER),
                Operand::Constant(1, TypeCode::INTEGER),
            ),
            Instruction::load_constant(1, 3, TypeCode::INTEGER, false),
            Instruction::less(
                true,
                Operand::Constant(4, TypeCode::INTEGER),
                Operand::Constant(5, TypeCode::INTEGER),
            ),
            Instruction::jump(1, true),
            Instruction::load_constant(2, 6, TypeCode::INTEGER, true),
            Instruction::load_constant(2, 1, TypeCode::INTEGER, false),
            Instruction::r#return(true, 2, TypeCode::INTEGER),
        ],
        positions: vec![
            Span(3, 9),
            Span(12, 13),
            Span(24, 30),
            Span(33, 34),
            Span(45, 53),
            Span(54, 55),
            Span(56, 57),
            Span(67, 69),
            Span(71, 71),
        ],
        integer_constants: vec![0, 42, 1, 2, 100, 50, 3],
        ..Chunk::default()
    };
    let return_value = Some(Value::integer(42));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

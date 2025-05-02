use dust_lang::{
    Chunk, FunctionType, Instruction, Operand, Span, Type, TypeCode, Value, compile, run,
};

#[test]
fn if_equal() {
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

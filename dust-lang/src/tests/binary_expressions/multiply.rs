use crate::{
    Address, Chunk, FunctionType, Instruction, Span, Type, Value, compile,
    instruction::{AddressKind, Destination},
    run,
};

#[test]
fn multiply_bytes() {
    let source = "0x28 * 0x02";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Byte),
        instructions: vec![
            Instruction::load_encoded(
                Destination::register(0),
                40,
                AddressKind::BYTE_MEMORY,
                false,
            ),
            Instruction::load_encoded(Destination::register(1), 2, AddressKind::BYTE_MEMORY, false),
            Instruction::multiply(
                Destination::register(2),
                Address::new(0, AddressKind::BYTE_REGISTER),
                Address::new(1, AddressKind::BYTE_REGISTER),
            ),
            Instruction::r#return(true, Address::new(2, AddressKind::BYTE_REGISTER)),
        ],
        positions: vec![Span(0, 4), Span(7, 11), Span(0, 11), Span(11, 11)],
        ..Chunk::default()
    };
    let return_value = Some(Value::byte(0x50));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn multiply_floats() {
    let source = "0.5 * 0.25";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Float),
        instructions: vec![
            Instruction::multiply(
                Destination::register(0),
                Address::new(0, AddressKind::FLOAT_CONSTANT),
                Address::new(1, AddressKind::FLOAT_CONSTANT),
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::FLOAT_REGISTER)),
        ],
        positions: vec![Span(0, 10), Span(10, 10)],
        float_constants: vec![0.5, 0.25],
        ..Chunk::default()
    };
    let return_value = Some(Value::float(0.125));

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
                Destination::register(0),
                Address::new(0, AddressKind::INTEGER_CONSTANT),
                Address::new(1, AddressKind::INTEGER_CONSTANT),
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::INTEGER_REGISTER)),
        ],
        positions: vec![Span(0, 6), Span(6, 6)],
        integer_constants: vec![10, 5],
        ..Chunk::default()
    };
    let return_value = Some(Value::integer(50));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

use crate::{
    Address, Chunk, DustString, FunctionType, Instruction, Span, Type, Value, compile,
    instruction::{AddressKind, Destination},
    run,
};

#[test]
fn add_bytes() {
    let source = "0x28 + 0x02";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Byte),
        instructions: vec![
            Instruction::load_encoded(
                Destination::stack(0),
                40,
                AddressKind::BYTE_MEMORY,
                false,
            ),
            Instruction::load_encoded(Destination::stack(1), 2, AddressKind::BYTE_MEMORY, false),
            Instruction::add(
                Destination::stack(2),
                Address::new(0, AddressKind::BYTE_REGISTER),
                Address::new(1, AddressKind::BYTE_REGISTER),
            ),
            Instruction::r#return(true, Address::new(2, AddressKind::BYTE_REGISTER)),
        ],
        positions: vec![Span(0, 4), Span(7, 11), Span(0, 11), Span(11, 11)],
        ..Chunk::default()
    };
    let return_value = Some(Value::byte(0x2A));

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
                Destination::stack(0),
                Address::new(0, AddressKind::CHARACTER_CONSTANT),
                Address::new(1, AddressKind::CHARACTER_CONSTANT),
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::STRING_REGISTER)),
        ],
        positions: vec![Span(0, 9), Span(9, 9)],
        character_constants: vec!['a', 'b'],
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
                Destination::stack(0),
                Address::new(0, AddressKind::FLOAT_CONSTANT),
                Address::new(1, AddressKind::FLOAT_CONSTANT),
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::FLOAT_REGISTER)),
        ],
        positions: vec![Span(0, 12), Span(12, 12)],
        float_constants: vec![2.40, 40.02],
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
                Destination::stack(0),
                Address::new(0, AddressKind::INTEGER_CONSTANT),
                Address::new(1, AddressKind::INTEGER_CONSTANT),
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::INTEGER_REGISTER)),
        ],
        positions: vec![Span(0, 6), Span(6, 6)],
        integer_constants: vec![40, 2],
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
                Destination::stack(0),
                Address::new(0, AddressKind::STRING_CONSTANT),
                Address::new(1, AddressKind::STRING_CONSTANT),
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::STRING_REGISTER)),
        ],
        positions: vec![Span(0, 20), Span(20, 20)],
        string_constants: vec![DustString::from("Hello, "), DustString::from("World!")],
        ..Chunk::default()
    };
    let return_value = Some(Value::string("Hello, World!"));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

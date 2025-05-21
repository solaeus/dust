use crate::{
    Address, Chunk, DustString, FunctionType, Instruction, Span, Type, Value, compile,
    instruction::{AddressKind, Destination},
    run,
};

#[test]
fn less_equal_booleans() {
    let source = "true <= false";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::load_encoded(
                Destination::register(0),
                true as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::load_encoded(
                Destination::register(1),
                false as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::less_equal(
                false,
                Address::new(0, AddressKind::BOOLEAN_REGISTER),
                Address::new(1, AddressKind::BOOLEAN_REGISTER),
            ),
            Instruction::jump(1, true),
            Instruction::load_encoded(
                Destination::register(2),
                true as u16,
                AddressKind::BOOLEAN_MEMORY,
                true,
            ),
            Instruction::load_encoded(
                Destination::register(2),
                false as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::r#return(true, Address::new(2, AddressKind::BOOLEAN_REGISTER)),
        ],
        positions: vec![
            Span(0, 4),
            Span(8, 13),
            Span(0, 13),
            Span(0, 13),
            Span(0, 13),
            Span(0, 13),
            Span(13, 13),
        ],
        ..Chunk::default()
    };
    let return_value = Some(Value::boolean(false));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn less_equal_bytes() {
    let source = "0x0A <= 0x03";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::load_encoded(
                Destination::register(0),
                0x0A,
                AddressKind::BYTE_MEMORY,
                false,
            ),
            Instruction::load_encoded(
                Destination::register(1),
                0x03,
                AddressKind::BYTE_MEMORY,
                false,
            ),
            Instruction::less_equal(
                false,
                Address::new(0, AddressKind::BYTE_REGISTER),
                Address::new(1, AddressKind::BYTE_REGISTER),
            ),
            Instruction::jump(1, true),
            Instruction::load_encoded(
                Destination::register(0),
                true as u16,
                AddressKind::BOOLEAN_MEMORY,
                true,
            ),
            Instruction::load_encoded(
                Destination::register(0),
                false as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::BOOLEAN_REGISTER)),
        ],
        positions: vec![
            Span(0, 4),
            Span(8, 12),
            Span(0, 12),
            Span(0, 12),
            Span(0, 12),
            Span(0, 12),
            Span(12, 12),
        ],
        ..Chunk::default()
    };
    let return_value = Some(Value::boolean(false));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn less_equal_characters() {
    let source = "'a' <= 'b'";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::less_equal(
                false,
                Address::new(0, AddressKind::CHARACTER_CONSTANT),
                Address::new(1, AddressKind::CHARACTER_CONSTANT),
            ),
            Instruction::jump(1, true),
            Instruction::load_encoded(
                Destination::register(0),
                true as u16,
                AddressKind::BOOLEAN_MEMORY,
                true,
            ),
            Instruction::load_encoded(
                Destination::register(0),
                false as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::BOOLEAN_REGISTER)),
        ],
        positions: vec![
            Span(0, 10),
            Span(0, 10),
            Span(0, 10),
            Span(0, 10),
            Span(10, 10),
        ],
        character_constants: vec!['a', 'b'],
        ..Chunk::default()
    };
    let return_value = Some(Value::boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn less_equal_floats() {
    let source = "10.0 <= 3.0";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::less_equal(
                false,
                Address::new(0, AddressKind::FLOAT_CONSTANT),
                Address::new(1, AddressKind::FLOAT_CONSTANT),
            ),
            Instruction::jump(1, true),
            Instruction::load_encoded(
                Destination::register(0),
                true as u16,
                AddressKind::BOOLEAN_MEMORY,
                true,
            ),
            Instruction::load_encoded(
                Destination::register(0),
                false as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::BOOLEAN_REGISTER)),
        ],
        positions: vec![
            Span(0, 11),
            Span(0, 11),
            Span(0, 11),
            Span(0, 11),
            Span(11, 11),
        ],
        float_constants: vec![10.0, 3.0],
        ..Chunk::default()
    };
    let return_value = Some(Value::boolean(false));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn less_equal_integers() {
    let source = "10 <= 3";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::less_equal(
                false,
                Address::new(0, AddressKind::INTEGER_CONSTANT),
                Address::new(1, AddressKind::INTEGER_CONSTANT),
            ),
            Instruction::jump(1, true),
            Instruction::load_encoded(
                Destination::register(0),
                true as u16,
                AddressKind::BOOLEAN_MEMORY,
                true,
            ),
            Instruction::load_encoded(
                Destination::register(0),
                false as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::BOOLEAN_REGISTER)),
        ],
        positions: vec![Span(0, 7), Span(0, 7), Span(0, 7), Span(0, 7), Span(7, 7)],
        integer_constants: vec![10, 3],
        ..Chunk::default()
    };
    let return_value = Some(Value::boolean(false));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn less_equal_strings() {
    let source = "\"abc\" <= \"def\"";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::less_equal(
                false,
                Address::new(0, AddressKind::STRING_CONSTANT),
                Address::new(1, AddressKind::STRING_CONSTANT),
            ),
            Instruction::jump(1, true),
            Instruction::load_encoded(
                Destination::register(0),
                true as u16,
                AddressKind::BOOLEAN_MEMORY,
                true,
            ),
            Instruction::load_encoded(
                Destination::register(0),
                false as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::BOOLEAN_REGISTER)),
        ],
        positions: vec![
            Span(0, 14),
            Span(0, 14),
            Span(0, 14),
            Span(0, 14),
            Span(14, 14),
        ],
        string_constants: vec![DustString::from("abc"), DustString::from("def")],
        ..Chunk::default()
    };
    let return_value = Some(Value::boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn less_equal_lists() {
    let source = "[1, 2, 3] <= [4, 5, 6]";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::load_constant(
                Destination::memory(0),
                Address::new(0, AddressKind::INTEGER_CONSTANT),
                false,
            ),
            Instruction::load_constant(
                Destination::memory(1),
                Address::new(1, AddressKind::INTEGER_CONSTANT),
                false,
            ),
            Instruction::load_constant(
                Destination::memory(2),
                Address::new(2, AddressKind::INTEGER_CONSTANT),
                false,
            ),
            Instruction::load_list(
                Destination::register(0),
                Address::new(0, AddressKind::INTEGER_MEMORY),
                2,
                false,
            ),
            Instruction::load_constant(
                Destination::memory(3),
                Address::new(3, AddressKind::INTEGER_CONSTANT),
                false,
            ),
            Instruction::load_constant(
                Destination::memory(4),
                Address::new(4, AddressKind::INTEGER_CONSTANT),
                false,
            ),
            Instruction::load_constant(
                Destination::memory(5),
                Address::new(5, AddressKind::INTEGER_CONSTANT),
                false,
            ),
            Instruction::load_list(
                Destination::register(1),
                Address::new(3, AddressKind::INTEGER_MEMORY),
                5,
                false,
            ),
            Instruction::less_equal(
                false,
                Address::new(0, AddressKind::LIST_REGISTER),
                Address::new(1, AddressKind::LIST_REGISTER),
            ),
            Instruction::jump(1, true),
            Instruction::load_encoded(
                Destination::register(0),
                true as u16,
                AddressKind::BOOLEAN_MEMORY,
                true,
            ),
            Instruction::load_encoded(
                Destination::register(0),
                false as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::BOOLEAN_REGISTER)),
        ],
        positions: vec![
            Span(1, 2),
            Span(4, 5),
            Span(7, 8),
            Span(0, 9),
            Span(14, 15),
            Span(17, 18),
            Span(20, 21),
            Span(13, 22),
            Span(0, 22),
            Span(0, 22),
            Span(0, 22),
            Span(0, 22),
            Span(22, 22),
        ],
        integer_constants: vec![1, 2, 3, 4, 5, 6],
        ..Chunk::default()
    };
    let return_value = Some(Value::boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

use crate::{
    Address, Chunk, DustString, FunctionType, Instruction, Span, Type, Value, compile,
    instruction::{AddressKind, Destination},
    run,
};

#[test]
fn less_booleans() {
    let source = "true < false";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::load_encoded(
                Destination::stack(0),
                true as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::load_encoded(
                Destination::stack(1),
                false as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::less(
                true,
                Address::new(0, AddressKind::BOOLEAN_REGISTER),
                Address::new(1, AddressKind::BOOLEAN_REGISTER),
            ),
            Instruction::jump(1, true),
            Instruction::load_encoded(
                Destination::stack(2),
                true as u16,
                AddressKind::BOOLEAN_MEMORY,
                true,
            ),
            Instruction::load_encoded(
                Destination::stack(2),
                false as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::r#return(true, Address::new(2, AddressKind::BOOLEAN_REGISTER)),
        ],
        positions: vec![
            Span(0, 4),
            Span(7, 12),
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
fn less_bytes() {
    let source = "0x0A < 0x03";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::load_encoded(
                Destination::stack(0),
                0x0A,
                AddressKind::BYTE_MEMORY,
                false,
            ),
            Instruction::load_encoded(
                Destination::stack(1),
                0x03,
                AddressKind::BYTE_MEMORY,
                false,
            ),
            Instruction::less(
                true,
                Address::new(0, AddressKind::BYTE_REGISTER),
                Address::new(1, AddressKind::BYTE_REGISTER),
            ),
            Instruction::jump(1, true),
            Instruction::load_encoded(
                Destination::stack(0),
                true as u16,
                AddressKind::BOOLEAN_MEMORY,
                true,
            ),
            Instruction::load_encoded(
                Destination::stack(0),
                false as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::BOOLEAN_REGISTER)),
        ],
        positions: vec![
            Span(0, 4),
            Span(7, 11),
            Span(0, 11),
            Span(0, 11),
            Span(0, 11),
            Span(0, 11),
            Span(11, 11),
        ],
        ..Chunk::default()
    };
    let return_value = Some(Value::boolean(false));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn less_characters() {
    let source = "'a' < 'b'";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::less(
                true,
                Address::new(0, AddressKind::CHARACTER_CONSTANT),
                Address::new(1, AddressKind::CHARACTER_CONSTANT),
            ),
            Instruction::jump(1, true),
            Instruction::load_encoded(
                Destination::stack(0),
                true as u16,
                AddressKind::BOOLEAN_MEMORY,
                true,
            ),
            Instruction::load_encoded(
                Destination::stack(0),
                false as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::BOOLEAN_REGISTER)),
        ],
        positions: vec![Span(0, 9), Span(0, 9), Span(0, 9), Span(0, 9), Span(9, 9)],
        character_constants: vec!['a', 'b'],
        ..Chunk::default()
    };
    let return_value = Some(Value::boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn less_floats() {
    let source = "10.0 < 3.0";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::less(
                true,
                Address::new(0, AddressKind::FLOAT_CONSTANT),
                Address::new(1, AddressKind::FLOAT_CONSTANT),
            ),
            Instruction::jump(1, true),
            Instruction::load_encoded(
                Destination::stack(0),
                true as u16,
                AddressKind::BOOLEAN_MEMORY,
                true,
            ),
            Instruction::load_encoded(
                Destination::stack(0),
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
        float_constants: vec![10.0, 3.0],
        ..Chunk::default()
    };
    let return_value = Some(Value::boolean(false));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn less_integers() {
    let source = "10 < 3";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::less(
                true,
                Address::new(0, AddressKind::INTEGER_CONSTANT),
                Address::new(1, AddressKind::INTEGER_CONSTANT),
            ),
            Instruction::jump(1, true),
            Instruction::load_encoded(
                Destination::stack(0),
                true as u16,
                AddressKind::BOOLEAN_MEMORY,
                true,
            ),
            Instruction::load_encoded(
                Destination::stack(0),
                false as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::BOOLEAN_REGISTER)),
        ],
        positions: vec![Span(0, 6), Span(0, 6), Span(0, 6), Span(0, 6), Span(6, 6)],
        integer_constants: vec![10, 3],
        ..Chunk::default()
    };
    let return_value = Some(Value::boolean(false));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn less_strings() {
    let source = "\"abc\" < \"def\"";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::less(
                true,
                Address::new(0, AddressKind::STRING_CONSTANT),
                Address::new(1, AddressKind::STRING_CONSTANT),
            ),
            Instruction::jump(1, true),
            Instruction::load_encoded(
                Destination::stack(0),
                true as u16,
                AddressKind::BOOLEAN_MEMORY,
                true,
            ),
            Instruction::load_encoded(
                Destination::stack(0),
                false as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::BOOLEAN_REGISTER)),
        ],
        positions: vec![
            Span(0, 13),
            Span(0, 13),
            Span(0, 13),
            Span(0, 13),
            Span(13, 13),
        ],
        string_constants: vec![DustString::from("abc"), DustString::from("def")],
        ..Chunk::default()
    };
    let return_value = Some(Value::boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn less_lists() {
    let source = "[1, 2, 3] < [4, 5, 6]";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::load_constant(
                Destination::heap(0),
                Address::new(0, AddressKind::INTEGER_CONSTANT),
                false,
            ),
            Instruction::load_constant(
                Destination::heap(1),
                Address::new(1, AddressKind::INTEGER_CONSTANT),
                false,
            ),
            Instruction::load_constant(
                Destination::heap(2),
                Address::new(2, AddressKind::INTEGER_CONSTANT),
                false,
            ),
            Instruction::load_list(
                Destination::stack(0),
                Address::new(0, AddressKind::INTEGER_MEMORY),
                2,
                false,
            ),
            Instruction::load_constant(
                Destination::heap(3),
                Address::new(3, AddressKind::INTEGER_CONSTANT),
                false,
            ),
            Instruction::load_constant(
                Destination::heap(4),
                Address::new(4, AddressKind::INTEGER_CONSTANT),
                false,
            ),
            Instruction::load_constant(
                Destination::heap(5),
                Address::new(5, AddressKind::INTEGER_CONSTANT),
                false,
            ),
            Instruction::load_list(
                Destination::stack(1),
                Address::new(3, AddressKind::INTEGER_MEMORY),
                5,
                false,
            ),
            Instruction::less(
                true,
                Address::new(0, AddressKind::LIST_REGISTER),
                Address::new(1, AddressKind::LIST_REGISTER),
            ),
            Instruction::jump(1, true),
            Instruction::load_encoded(
                Destination::stack(0),
                true as u16,
                AddressKind::BOOLEAN_MEMORY,
                true,
            ),
            Instruction::load_encoded(
                Destination::stack(0),
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
            Span(13, 14),
            Span(16, 17),
            Span(19, 20),
            Span(12, 21),
            Span(0, 21),
            Span(0, 21),
            Span(0, 21),
            Span(0, 21),
            Span(21, 21),
        ],
        integer_constants: vec![1, 2, 3, 4, 5, 6],
        ..Chunk::default()
    };
    let return_value = Some(Value::boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

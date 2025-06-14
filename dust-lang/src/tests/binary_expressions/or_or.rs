use crate::{
    Address, Chunk, FunctionType, Instruction, Span, Type, Value, compile,
    instruction::{AddressKind, Destination},
    run,
};

#[test]
fn true_or_true_or_true() {
    let source = "true || true || true";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::load_encoded(
                Destination::stack(0),
                true as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::test(Address::new(0, AddressKind::BOOLEAN_REGISTER), false),
            Instruction::jump(2, true),
            Instruction::load_encoded(
                Destination::stack(0),
                true as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::test(Address::new(0, AddressKind::BOOLEAN_REGISTER), false),
            Instruction::jump(1, true),
            Instruction::load_encoded(
                Destination::stack(0),
                true as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::BOOLEAN_REGISTER)),
        ],
        positions: vec![
            Span(0, 4),
            Span(5, 7),
            Span(5, 7),
            Span(8, 12),
            Span(13, 15),
            Span(13, 15),
            Span(16, 20),
            Span(20, 20),
        ],
        ..Chunk::default()
    };
    let return_value = Some(Value::boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn true_or_true_or_false() {
    let source = "true || true || false";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::load_encoded(
                Destination::stack(0),
                true as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::test(Address::new(0, AddressKind::BOOLEAN_REGISTER), false),
            Instruction::jump(2, true),
            Instruction::load_encoded(
                Destination::stack(0),
                true as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::test(Address::new(0, AddressKind::BOOLEAN_REGISTER), false),
            Instruction::jump(1, true),
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
            Span(5, 7),
            Span(5, 7),
            Span(8, 12),
            Span(13, 15),
            Span(13, 15),
            Span(16, 21),
            Span(21, 21),
        ],
        ..Chunk::default()
    };
    let return_value = Some(Value::boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn false_or_true_or_true() {
    let source = "false || true || true";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::load_encoded(
                Destination::stack(0),
                false as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::test(Address::new(0, AddressKind::BOOLEAN_REGISTER), false),
            Instruction::jump(2, true),
            Instruction::load_encoded(
                Destination::stack(0),
                true as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::test(Address::new(0, AddressKind::BOOLEAN_REGISTER), false),
            Instruction::jump(1, true),
            Instruction::load_encoded(
                Destination::stack(0),
                true as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::BOOLEAN_REGISTER)),
        ],
        positions: vec![
            Span(0, 5),
            Span(6, 8),
            Span(6, 8),
            Span(9, 13),
            Span(14, 16),
            Span(14, 16),
            Span(17, 21),
            Span(21, 21),
        ],
        ..Chunk::default()
    };
    let return_value = Some(Value::boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn false_or_false_or_false() {
    let source = "false || false || false";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::load_encoded(
                Destination::stack(0),
                false as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::test(Address::new(0, AddressKind::BOOLEAN_REGISTER), false),
            Instruction::jump(2, true),
            Instruction::load_encoded(
                Destination::stack(0),
                false as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::test(Address::new(0, AddressKind::BOOLEAN_REGISTER), false),
            Instruction::jump(1, true),
            Instruction::load_encoded(
                Destination::stack(0),
                false as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::BOOLEAN_REGISTER)),
        ],
        positions: vec![
            Span(0, 5),
            Span(6, 8),
            Span(6, 8),
            Span(9, 14),
            Span(15, 17),
            Span(15, 17),
            Span(18, 23),
            Span(23, 23),
        ],
        ..Chunk::default()
    };
    let return_value = Some(Value::boolean(false));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

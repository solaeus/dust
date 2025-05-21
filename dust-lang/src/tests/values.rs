use crate::{
    Address, Chunk, ConcreteList, ConcreteValue, DustString, FunctionType, Instruction, Span, Type,
    compile,
    instruction::{AddressKind, Destination},
    run,
};

#[test]
fn load_boolean_true() {
    let source = "true";
    let chunk = Chunk {
        name: Some(DustString::from("anonymous")),
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::load_encoded(
                Destination::register(0),
                true as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::BOOLEAN_REGISTER)),
        ],
        positions: vec![Span(0, 4), Span(4, 4)],
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_boolean_false() {
    let source = "false";
    let chunk = Chunk {
        name: Some(DustString::from("anonymous")),
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::load_encoded(
                Destination::register(0),
                false as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::BOOLEAN_REGISTER)),
        ],
        positions: vec![Span(0, 5), Span(5, 5)],
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(false));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_byte() {
    let source = "0x2a";
    let chunk = Chunk {
        name: Some(DustString::from("anonymous")),
        r#type: FunctionType::new([], [], Type::Byte),
        instructions: vec![
            Instruction::load_encoded(
                Destination::register(0),
                42,
                AddressKind::BYTE_MEMORY,
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::BYTE_REGISTER)),
        ],
        positions: vec![Span(0, 4), Span(4, 4)],
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Byte(42));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_character() {
    let source = "'a'";
    let chunk = Chunk {
        name: Some(DustString::from("anonymous")),
        r#type: FunctionType::new([], [], Type::Character),
        instructions: vec![
            Instruction::load_constant(
                Destination::register(0),
                Address::new(0, AddressKind::CHARACTER_CONSTANT),
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::CHARACTER_REGISTER)),
        ],
        positions: vec![Span(0, 3), Span(3, 3)],
        character_constants: vec!['a'],
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Character('a'));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_float() {
    let source = "42.42";
    let chunk = Chunk {
        name: Some(DustString::from("anonymous")),
        r#type: FunctionType::new([], [], Type::Float),
        instructions: vec![
            Instruction::load_constant(
                Destination::register(0),
                Address::new(0, AddressKind::FLOAT_CONSTANT),
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::FLOAT_REGISTER)),
        ],
        positions: vec![Span(0, 5), Span(5, 5)],
        float_constants: vec![42.42],
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Float(42.42));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_integer() {
    let source = "42";
    let chunk = Chunk {
        name: Some(DustString::from("anonymous")),
        r#type: FunctionType::new([], [], Type::Integer),
        instructions: vec![
            Instruction::load_constant(
                Destination::register(0),
                Address::new(0, AddressKind::INTEGER_CONSTANT),
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::INTEGER_REGISTER)),
        ],
        positions: vec![Span(0, 2), Span(2, 2)],
        integer_constants: vec![42],
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Integer(42));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_string() {
    let source = "\"Hello, World!\"";
    let chunk = Chunk {
        name: Some(DustString::from("anonymous")),
        r#type: FunctionType::new([], [], Type::String),
        instructions: vec![
            Instruction::load_constant(
                Destination::register(0),
                Address::new(0, AddressKind::STRING_CONSTANT),
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::STRING_REGISTER)),
        ],
        positions: vec![Span(0, 15), Span(15, 15)],
        string_constants: vec![DustString::from("Hello, World!")],
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::String(DustString::from("Hello, World!")));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_boolean_list() {
    let source = "[true, false]";
    let chunk = Chunk {
        name: Some(DustString::from("anonymous")),
        r#type: FunctionType::new([], [], Type::List(Box::new(Type::Boolean))),
        instructions: vec![
            Instruction::load_encoded(
                Destination::memory(0),
                true as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::load_encoded(
                Destination::memory(1),
                false as u16,
                AddressKind::BOOLEAN_MEMORY,
                false,
            ),
            Instruction::load_list(
                Destination::register(0),
                Address::new(0, AddressKind::BOOLEAN_MEMORY),
                1,
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::LIST_REGISTER)),
        ],
        positions: vec![Span(1, 5), Span(7, 12), Span(0, 13), Span(13, 13)],
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::List(ConcreteList::Boolean(vec![
        true, false,
    ])));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_byte_list() {
    let source = "[0x2a, 0x42]";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_character_list() {
    let source = "['a', 'b']";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_float_list() {
    let source = "[42.42, 24.24]";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_integer_list() {
    let source = "[1, 2, 3]";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_string_list() {
    let source = "[\"Hello\", \"World\"]";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_nested_list() {
    let source = "[[1, 2], [3, 4]]";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_deeply_nested_list() {
    let source = "[[[1, 2], [3, 4]], [[5, 6], [7, 8]]]";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_function() {
    let source = "fn () {}";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_boolean_in_function() {
    let source = "fn () { true }";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_integer_in_function() {
    let source = "fn () { 42 }";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_string_in_function() {
    let source = "fn () { \"Hello\" }";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_list_in_function() {
    let source = "fn () { [1, 2, 3] }";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_byte_in_function() {
    let source = "fn () { 0x2a }";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_character_in_function() {
    let source = "fn () { 'a' }";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_float_in_function() {
    let source = "fn () { 42.42 }";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_nested_list_in_function() {
    let source = "fn () { [[1, 2], [3, 4]] }";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_deeply_nested_list_in_function() {
    let source = "fn () { [[[1, 2], [3, 4]], [[5, 6], [7, 8]]] }";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_function_in_function() {
    let source = "fn outer() { fn inner() -> int { 42 } }";
    let chunk = Chunk {
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

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
        name: Some(DustString::from("anonymous")),
        r#type: FunctionType::new([], [], Type::List(Box::new(Type::Byte))),
        instructions: vec![
            Instruction::load_encoded(Destination::memory(0), 42, AddressKind::BYTE_MEMORY, false),
            Instruction::load_encoded(Destination::memory(1), 66, AddressKind::BYTE_MEMORY, false),
            Instruction::load_list(
                Destination::register(0),
                Address::new(0, AddressKind::BYTE_MEMORY),
                1,
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::LIST_REGISTER)),
        ],
        positions: vec![Span(1, 5), Span(7, 11), Span(0, 12), Span(12, 12)],
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::List(ConcreteList::Byte(vec![42, 66])));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_character_list() {
    let source = "['a', 'b']";
    let chunk = Chunk {
        name: Some(DustString::from("anonymous")),
        r#type: FunctionType::new([], [], Type::List(Box::new(Type::Character))),
        instructions: vec![
            Instruction::load_constant(
                Destination::memory(0),
                Address::new(0, AddressKind::CHARACTER_CONSTANT),
                false,
            ),
            Instruction::load_constant(
                Destination::memory(1),
                Address::new(1, AddressKind::CHARACTER_CONSTANT),
                false,
            ),
            Instruction::load_list(
                Destination::register(0),
                Address::new(0, AddressKind::CHARACTER_MEMORY),
                1,
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::LIST_REGISTER)),
        ],
        positions: vec![Span(1, 4), Span(6, 9), Span(0, 10), Span(10, 10)],
        character_constants: vec!['a', 'b'],
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::List(ConcreteList::Character(vec!['a', 'b'])));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_float_list() {
    let source = "[42.42, 24.24]";
    let chunk = Chunk {
        name: Some(DustString::from("anonymous")),
        r#type: FunctionType::new([], [], Type::List(Box::new(Type::Float))),
        instructions: vec![
            Instruction::load_constant(
                Destination::memory(0),
                Address::new(0, AddressKind::FLOAT_CONSTANT),
                false,
            ),
            Instruction::load_constant(
                Destination::memory(1),
                Address::new(1, AddressKind::FLOAT_CONSTANT),
                false,
            ),
            Instruction::load_list(
                Destination::register(0),
                Address::new(0, AddressKind::FLOAT_MEMORY),
                1,
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::LIST_REGISTER)),
        ],
        positions: vec![Span(1, 6), Span(8, 13), Span(0, 14), Span(14, 14)],
        float_constants: vec![42.42, 24.24],
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::List(ConcreteList::Float(vec![42.42, 24.24])));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_integer_list() {
    let source = "[1, 2, 3]";
    let chunk = Chunk {
        name: Some(DustString::from("anonymous")),
        r#type: FunctionType::new([], [], Type::List(Box::new(Type::Integer))),
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
            Instruction::r#return(true, Address::new(0, AddressKind::LIST_REGISTER)),
        ],
        positions: vec![Span(1, 2), Span(4, 5), Span(7, 8), Span(0, 9), Span(9, 9)],
        integer_constants: vec![1, 2, 3],
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::List(ConcreteList::Integer(vec![1, 2, 3])));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_string_list() {
    let source = "[\"Hello\", \"World\"]";
    let chunk = Chunk {
        name: Some(DustString::from("anonymous")),
        r#type: FunctionType::new([], [], Type::List(Box::new(Type::String))),
        instructions: vec![
            Instruction::load_constant(
                Destination::memory(0),
                Address::new(0, AddressKind::STRING_CONSTANT),
                false,
            ),
            Instruction::load_constant(
                Destination::memory(1),
                Address::new(1, AddressKind::STRING_CONSTANT),
                false,
            ),
            Instruction::load_list(
                Destination::register(0),
                Address::new(0, AddressKind::STRING_MEMORY),
                1,
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::LIST_REGISTER)),
        ],
        positions: vec![Span(1, 8), Span(10, 17), Span(0, 18), Span(18, 18)],
        string_constants: vec![DustString::from("Hello"), DustString::from("World")],
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::List(ConcreteList::String(vec![
        DustString::from("Hello"),
        DustString::from("World"),
    ])));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_nested_list() {
    let source = "[[1, 2], [3, 4]]";
    let chunk = Chunk {
        name: Some(DustString::from("anonymous")),
        r#type: FunctionType::new(
            [],
            [],
            Type::List(Box::new(Type::List(Box::new(Type::Integer)))),
        ),
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
            Instruction::load_list(
                Destination::memory(0),
                Address::new(0, AddressKind::INTEGER_MEMORY),
                1,
                false,
            ),
            Instruction::load_constant(
                Destination::memory(2),
                Address::new(2, AddressKind::INTEGER_CONSTANT),
                false,
            ),
            Instruction::load_constant(
                Destination::memory(3),
                Address::new(3, AddressKind::INTEGER_CONSTANT),
                false,
            ),
            Instruction::load_list(
                Destination::memory(1),
                Address::new(2, AddressKind::INTEGER_MEMORY),
                3,
                false,
            ),
            Instruction::load_list(
                Destination::register(0),
                Address::new(0, AddressKind::LIST_MEMORY),
                1,
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::LIST_REGISTER)),
        ],
        positions: vec![
            Span(2, 3),
            Span(5, 6),
            Span(1, 7),
            Span(10, 11),
            Span(13, 14),
            Span(9, 15),
            Span(0, 16),
            Span(16, 16),
        ],
        integer_constants: vec![1, 2, 3, 4],
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::List(ConcreteList::List {
        list_items: vec![
            ConcreteList::Integer(vec![1, 2]),
            ConcreteList::Integer(vec![3, 4]),
        ],
        list_item_type: Type::List(Box::new(Type::Integer)),
    }));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_deeply_nested_list() {
    let source = "[[[1, 2], [3, 4]], [[5, 6], [7, 8]]]";
    let chunk = Chunk {
        name: Some(DustString::from("anonymous")),
        r#type: FunctionType::new(
            [],
            [],
            Type::List(Box::new(Type::List(Box::new(Type::List(Box::new(
                Type::Integer,
            )))))),
        ),
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
            Instruction::load_list(
                Destination::memory(0),
                Address::new(0, AddressKind::INTEGER_MEMORY),
                1,
                false,
            ),
            Instruction::load_constant(
                Destination::memory(2),
                Address::new(2, AddressKind::INTEGER_CONSTANT),
                false,
            ),
            Instruction::load_constant(
                Destination::memory(3),
                Address::new(3, AddressKind::INTEGER_CONSTANT),
                false,
            ),
            Instruction::load_list(
                Destination::memory(1),
                Address::new(2, AddressKind::INTEGER_MEMORY),
                3,
                false,
            ),
            Instruction::load_list(
                Destination::memory(2),
                Address::new(0, AddressKind::LIST_MEMORY),
                1,
                false,
            ),
            Instruction::close(
                Address::new(0, AddressKind::LIST_MEMORY),
                Address::new(1, AddressKind::LIST_MEMORY),
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
                Destination::memory(3),
                Address::new(4, AddressKind::INTEGER_MEMORY),
                5,
                false,
            ),
            Instruction::load_constant(
                Destination::memory(6),
                Address::new(6, AddressKind::INTEGER_CONSTANT),
                false,
            ),
            Instruction::load_constant(
                Destination::memory(7),
                Address::new(7, AddressKind::INTEGER_CONSTANT),
                false,
            ),
            Instruction::load_list(
                Destination::memory(4),
                Address::new(6, AddressKind::INTEGER_MEMORY),
                7,
                false,
            ),
            Instruction::load_list(
                Destination::memory(5),
                Address::new(3, AddressKind::LIST_MEMORY),
                4,
                false,
            ),
            Instruction::close(
                Address::new(3, AddressKind::LIST_MEMORY),
                Address::new(4, AddressKind::LIST_MEMORY),
            ),
            Instruction::load_list(
                Destination::register(0),
                Address::new(2, AddressKind::LIST_MEMORY),
                5,
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::LIST_REGISTER)),
        ],
        positions: vec![
            Span(3, 4),
            Span(6, 7),
            Span(2, 8),
            Span(11, 12),
            Span(14, 15),
            Span(10, 16),
            Span(1, 17),
            Span(19, 20),
            Span(21, 22),
            Span(24, 25),
            Span(20, 26),
            Span(29, 30),
            Span(32, 33),
            Span(28, 34),
            Span(19, 35),
            Span(35, 36),
            Span(0, 36),
            Span(36, 36),
        ],
        integer_constants: vec![1, 2, 3, 4, 5, 6, 7, 8],
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::List(ConcreteList::of_lists(vec![
        ConcreteList::of_lists(vec![
            ConcreteList::Integer(vec![1, 2]),
            ConcreteList::Integer(vec![3, 4]),
        ]),
        ConcreteList::of_lists(vec![
            ConcreteList::Integer(vec![5, 6]),
            ConcreteList::Integer(vec![7, 8]),
        ]),
    ])));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_function() {
    let source = "fn () {}";
    let chunk = Chunk {
        name: Some(DustString::from("anonymous")),
        r#type: FunctionType::new(
            [],
            [],
            Type::Function(Box::new(FunctionType::new(vec![], vec![], Type::None))),
        ),
        instructions: vec![
            Instruction::load_function(
                Destination::register(0),
                Address::new(0, AddressKind::FUNCTION_PROTOTYPE),
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::FUNCTION_REGISTER)),
        ],
        positions: vec![Span(0, 8), Span(8, 8)],
        prototypes: vec![Chunk {
            name: None,
            instructions: vec![Instruction::r#return(
                false,
                Address::new(0, AddressKind::NONE),
            )],
            positions: vec![Span(7, 8)],
            ..Default::default()
        }],
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Function(chunk.prototypes[0].clone()));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_boolean_in_function() {
    let source = "fn () { true }";
    let chunk = Chunk {
        name: Some(DustString::from("anonymous")),
        r#type: FunctionType::new(
            [],
            [],
            Type::Function(Box::new(FunctionType::new(vec![], vec![], Type::Boolean))),
        ),
        instructions: vec![
            Instruction::load_function(
                Destination::register(0),
                Address::new(0, AddressKind::FUNCTION_PROTOTYPE),
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::FUNCTION_REGISTER)),
        ],
        positions: vec![Span(0, 14), Span(14, 14)],
        prototypes: vec![Chunk {
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
            positions: vec![Span(8, 12), Span(13, 14)],
            ..Default::default()
        }],
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Function(chunk.prototypes[0].clone()));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_integer_in_function() {
    let source = "fn () { 42 }";
    let chunk = Chunk {
        name: Some(DustString::from("anonymous")),
        r#type: FunctionType::new(
            [],
            [],
            Type::Function(Box::new(FunctionType::new(vec![], vec![], Type::Integer))),
        ),
        instructions: vec![
            Instruction::load_function(
                Destination::register(0),
                Address::new(0, AddressKind::FUNCTION_PROTOTYPE),
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::FUNCTION_REGISTER)),
        ],
        positions: vec![Span(0, 12), Span(12, 12)],
        prototypes: vec![Chunk {
            r#type: FunctionType::new([], [], Type::Integer),
            instructions: vec![
                Instruction::load_constant(
                    Destination::register(0),
                    Address::new(0, AddressKind::INTEGER_CONSTANT),
                    false,
                ),
                Instruction::r#return(true, Address::new(0, AddressKind::INTEGER_REGISTER)),
            ],
            positions: vec![Span(8, 10), Span(11, 12)],
            integer_constants: vec![42],
            ..Default::default()
        }],
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Function(chunk.prototypes[0].clone()));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_string_in_function() {
    let source = "fn () { \"Hello\" }";
    let chunk = Chunk {
        name: Some(DustString::from("anonymous")),
        r#type: FunctionType::new(
            [],
            [],
            Type::Function(Box::new(FunctionType::new(vec![], vec![], Type::String))),
        ),
        instructions: vec![
            Instruction::load_function(
                Destination::register(0),
                Address::new(0, AddressKind::FUNCTION_PROTOTYPE),
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::FUNCTION_REGISTER)),
        ],
        positions: vec![Span(0, 17), Span(17, 17)],
        prototypes: vec![Chunk {
            r#type: FunctionType::new([], [], Type::String),
            instructions: vec![
                Instruction::load_constant(
                    Destination::register(0),
                    Address::new(0, AddressKind::STRING_CONSTANT),
                    false,
                ),
                Instruction::r#return(true, Address::new(0, AddressKind::STRING_REGISTER)),
            ],
            positions: vec![Span(8, 15), Span(16, 17)],
            string_constants: vec![DustString::from("Hello")],
            ..Default::default()
        }],
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Function(chunk.prototypes[0].clone()));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_list_in_function() {
    let source = "fn () { [1, 2, 3] }";
    let chunk = Chunk {
        name: Some(DustString::from("anonymous")),
        r#type: FunctionType::new(
            [],
            [],
            Type::Function(Box::new(FunctionType::new(
                vec![],
                vec![],
                Type::List(Box::new(Type::Integer)),
            ))),
        ),
        instructions: vec![
            Instruction::load_function(
                Destination::register(0),
                Address::new(0, AddressKind::FUNCTION_PROTOTYPE),
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::FUNCTION_REGISTER)),
        ],
        positions: vec![Span(0, 19), Span(19, 19)],
        prototypes: vec![Chunk {
            r#type: FunctionType::new([], [], Type::List(Box::new(Type::Integer))),
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
                Instruction::r#return(true, Address::new(0, AddressKind::LIST_REGISTER)),
            ],
            positions: vec![
                Span(9, 10),
                Span(12, 13),
                Span(15, 16),
                Span(8, 17),
                Span(18, 19),
            ],
            integer_constants: vec![1, 2, 3],
            ..Default::default()
        }],
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Function(chunk.prototypes[0].clone()));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_byte_in_function() {
    let source = "fn () { 0x2a }";
    let chunk = Chunk {
        name: Some(DustString::from("anonymous")),
        r#type: FunctionType::new(
            [],
            [],
            Type::Function(Box::new(FunctionType::new(vec![], vec![], Type::Byte))),
        ),
        instructions: vec![
            Instruction::load_function(
                Destination::register(0),
                Address::new(0, AddressKind::FUNCTION_PROTOTYPE),
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::FUNCTION_REGISTER)),
        ],
        positions: vec![Span(0, 14), Span(14, 14)], // Placeholder positions
        prototypes: vec![Chunk {
            r#type: FunctionType::new(vec![], vec![], Type::Byte),
            instructions: vec![
                Instruction::load_encoded(
                    Destination::register(0),
                    42,
                    AddressKind::BYTE_MEMORY,
                    false,
                ),
                Instruction::r#return(true, Address::new(0, AddressKind::BYTE_REGISTER)),
            ],
            positions: vec![Span(8, 12), Span(13, 14)],
            ..Default::default()
        }],
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Function(chunk.prototypes[0].clone()));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_character_in_function() {
    let source = "fn () { 'a' }";
    let chunk = Chunk {
        name: Some(DustString::from("anonymous")),
        r#type: FunctionType::new(
            [],
            [],
            Type::Function(Box::new(FunctionType::new(vec![], vec![], Type::Character))),
        ),
        instructions: vec![
            Instruction::load_function(
                Destination::register(0),
                Address::new(0, AddressKind::FUNCTION_PROTOTYPE),
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::FUNCTION_REGISTER)),
        ],
        positions: vec![Span(0, 13), Span(13, 13)],
        prototypes: vec![Chunk {
            r#type: FunctionType::new(vec![], vec![], Type::Character),
            instructions: vec![
                Instruction::load_constant(
                    Destination::register(0),
                    Address::new(0, AddressKind::CHARACTER_CONSTANT),
                    false,
                ),
                Instruction::r#return(true, Address::new(0, AddressKind::CHARACTER_REGISTER)),
            ],
            positions: vec![Span(8, 11), Span(12, 13)],
            character_constants: vec!['a'],
            ..Default::default()
        }],
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Function(chunk.prototypes[0].clone()));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_float_in_function() {
    let source = "fn () { 42.42 }";
    let chunk = Chunk {
        name: Some(DustString::from("anonymous")),
        r#type: FunctionType::new(
            [],
            [],
            Type::Function(Box::new(FunctionType::new(vec![], vec![], Type::Float))),
        ),
        instructions: vec![
            Instruction::load_function(
                Destination::register(0),
                Address::new(0, AddressKind::FUNCTION_PROTOTYPE),
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::FUNCTION_REGISTER)),
        ],
        positions: vec![Span(0, 15), Span(15, 15)],
        prototypes: vec![Chunk {
            r#type: FunctionType::new(vec![], vec![], Type::Float),
            instructions: vec![
                Instruction::load_constant(
                    Destination::register(0),
                    Address::new(0, AddressKind::FLOAT_CONSTANT),
                    false,
                ),
                Instruction::r#return(true, Address::new(0, AddressKind::FLOAT_REGISTER)),
            ],
            positions: vec![Span(8, 13), Span(14, 15)],
            float_constants: vec![42.42],
            ..Default::default()
        }],
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Function(chunk.prototypes[0].clone()));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_nested_list_in_function() {
    let source = "fn () { [[1, 2], [3, 4]] }";
    let chunk = Chunk {
        name: Some(DustString::from("anonymous")),
        r#type: FunctionType::new(
            [],
            [],
            Type::Function(Box::new(FunctionType::new(
                vec![],
                vec![],
                Type::List(Box::new(Type::List(Box::new(Type::Integer)))),
            ))),
        ),
        instructions: vec![
            Instruction::load_function(
                Destination::register(0),
                Address::new(0, AddressKind::FUNCTION_PROTOTYPE),
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::FUNCTION_REGISTER)),
        ],
        positions: vec![Span(0, 26), Span(26, 26)],
        prototypes: vec![Chunk {
            r#type: FunctionType::new(
                vec![],
                vec![],
                Type::List(Box::new(Type::List(Box::new(Type::Integer)))),
            ),
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
                Instruction::load_list(
                    Destination::memory(0),
                    Address::new(0, AddressKind::INTEGER_MEMORY),
                    1,
                    false,
                ),
                Instruction::load_constant(
                    Destination::memory(2),
                    Address::new(2, AddressKind::INTEGER_CONSTANT),
                    false,
                ),
                Instruction::load_constant(
                    Destination::memory(3),
                    Address::new(3, AddressKind::INTEGER_CONSTANT),
                    false,
                ),
                Instruction::load_list(
                    Destination::memory(1),
                    Address::new(2, AddressKind::INTEGER_MEMORY),
                    3,
                    false,
                ),
                Instruction::load_list(
                    Destination::register(0),
                    Address::new(0, AddressKind::LIST_MEMORY),
                    1,
                    false,
                ),
                Instruction::r#return(true, Address::new(0, AddressKind::LIST_REGISTER)),
            ],
            positions: vec![
                Span(10, 11),
                Span(13, 14),
                Span(9, 15),
                Span(18, 19),
                Span(21, 22),
                Span(17, 23),
                Span(8, 24),
                Span(25, 26),
            ],
            integer_constants: vec![1, 2, 3, 4],
            ..Default::default()
        }],
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Function(chunk.prototypes[0].clone()));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_deeply_nested_list_in_function() {
    let source = "fn () { [[[1, 2], [3, 4]], [[5, 6], [7, 8]]] }";
    let chunk = Chunk {
        name: Some(DustString::from("anonymous")),
        r#type: FunctionType::new(
            [],
            [],
            Type::Function(Box::new(FunctionType::new(
                vec![],
                vec![],
                Type::List(Box::new(Type::List(Box::new(Type::List(Box::new(
                    Type::Integer,
                )))))),
            ))),
        ),
        instructions: vec![
            Instruction::load_function(
                Destination::register(0),
                Address::new(0, AddressKind::FUNCTION_PROTOTYPE),
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::FUNCTION_REGISTER)),
        ],
        positions: vec![Span(0, 46), Span(46, 46)],
        prototypes: vec![Chunk {
            r#type: FunctionType::new(
                [],
                [],
                Type::List(Box::new(Type::List(Box::new(Type::List(Box::new(
                    Type::Integer,
                )))))),
            ),
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
                Instruction::load_list(
                    Destination::memory(0),
                    Address::new(0, AddressKind::INTEGER_MEMORY),
                    1,
                    false,
                ),
                Instruction::load_constant(
                    Destination::memory(2),
                    Address::new(2, AddressKind::INTEGER_CONSTANT),
                    false,
                ),
                Instruction::load_constant(
                    Destination::memory(3),
                    Address::new(3, AddressKind::INTEGER_CONSTANT),
                    false,
                ),
                Instruction::load_list(
                    Destination::memory(1),
                    Address::new(2, AddressKind::INTEGER_MEMORY),
                    3,
                    false,
                ),
                Instruction::load_list(
                    Destination::memory(2),
                    Address::new(0, AddressKind::LIST_MEMORY),
                    1,
                    false,
                ),
                Instruction::close(
                    Address::new(0, AddressKind::LIST_MEMORY),
                    Address::new(1, AddressKind::LIST_MEMORY),
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
                    Destination::memory(3),
                    Address::new(4, AddressKind::INTEGER_MEMORY),
                    5,
                    false,
                ),
                Instruction::load_constant(
                    Destination::memory(6),
                    Address::new(6, AddressKind::INTEGER_CONSTANT),
                    false,
                ),
                Instruction::load_constant(
                    Destination::memory(7),
                    Address::new(7, AddressKind::INTEGER_CONSTANT),
                    false,
                ),
                Instruction::load_list(
                    Destination::memory(4),
                    Address::new(6, AddressKind::INTEGER_MEMORY),
                    7,
                    false,
                ),
                Instruction::load_list(
                    Destination::memory(5),
                    Address::new(3, AddressKind::LIST_MEMORY),
                    4,
                    false,
                ),
                Instruction::close(
                    Address::new(3, AddressKind::LIST_MEMORY),
                    Address::new(4, AddressKind::LIST_MEMORY),
                ),
                Instruction::load_list(
                    Destination::register(0),
                    Address::new(2, AddressKind::LIST_MEMORY),
                    5,
                    false,
                ),
                Instruction::r#return(true, Address::new(0, AddressKind::LIST_REGISTER)),
            ],
            positions: vec![
                Span(11, 12),
                Span(14, 15),
                Span(10, 16),
                Span(19, 20),
                Span(22, 23),
                Span(18, 24),
                Span(9, 25),
                Span(27, 28),
                Span(29, 30),
                Span(32, 33),
                Span(28, 34),
                Span(37, 38),
                Span(40, 41),
                Span(36, 42),
                Span(27, 43),
                Span(43, 44),
                Span(8, 44),
                Span(45, 46),
            ],
            integer_constants: vec![1, 2, 3, 4, 5, 6, 7, 8],
            ..Default::default()
        }],
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Function(chunk.prototypes[0].clone()));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_function_in_function() {
    let source = "fn () { fn () -> int { 42 } }";
    let chunk = Chunk {
        name: Some(DustString::from("anonymous")),
        r#type: FunctionType::new(
            [],
            [],
            Type::Function(Box::new(FunctionType::new(
                vec![],
                vec![],
                Type::Function(Box::new(FunctionType::new(vec![], vec![], Type::Integer))),
            ))),
        ),
        instructions: vec![
            Instruction::load_function(
                Destination::register(0),
                Address::new(0, AddressKind::FUNCTION_PROTOTYPE),
                false,
            ),
            Instruction::r#return(true, Address::new(0, AddressKind::FUNCTION_REGISTER)),
        ],
        positions: vec![Span(0, 29), Span(29, 29)],
        prototypes: vec![Chunk {
            r#type: FunctionType::new(
                [],
                [],
                Type::Function(Box::new(FunctionType::new(vec![], vec![], Type::Integer))),
            ),
            instructions: vec![
                Instruction::load_function(
                    Destination::register(0),
                    Address::new(0, AddressKind::FUNCTION_PROTOTYPE),
                    false,
                ),
                Instruction::r#return(true, Address::new(0, AddressKind::FUNCTION_REGISTER)),
            ],
            positions: vec![Span(8, 27), Span(28, 29)],
            prototypes: vec![Chunk {
                r#type: FunctionType::new(vec![], vec![], Type::Integer),
                instructions: vec![
                    Instruction::load_constant(
                        Destination::register(0),
                        Address::new(0, AddressKind::INTEGER_CONSTANT),
                        false,
                    ),
                    Instruction::r#return(true, Address::new(0, AddressKind::INTEGER_REGISTER)),
                ],
                positions: vec![Span(23, 25), Span(26, 27)],
                integer_constants: vec![42],
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    };
    let return_value = Some(ConcreteValue::Function(chunk.prototypes[0].clone()));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

use dust_lang::{
    Chunk, ConcreteValue, DustString, FunctionType, Instruction, Span, Type, Value, compile,
    instruction::TypeCode, run,
};

#[test]
fn load_boolean_true() {
    let source = "true";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::load_encoded(0, true as u8, TypeCode::BOOLEAN, false),
            Instruction::r#return(true, 0, TypeCode::BOOLEAN),
        ],
        positions: vec![Span(0, 4), Span(4, 4)],
        ..Chunk::default()
    };
    let return_value = Some(Value::boolean(true));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_boolean_false() {
    let source = "false";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Boolean),
        instructions: vec![
            Instruction::load_encoded(0, false as u8, TypeCode::BOOLEAN, false),
            Instruction::r#return(true, 0, TypeCode::BOOLEAN),
        ],
        positions: vec![Span(0, 5), Span(5, 5)],
        ..Chunk::default()
    };
    let return_value = Some(Value::boolean(false));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_byte() {
    let source = "0x2a";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Byte),
        instructions: vec![
            Instruction::load_encoded(0, 0x2a, TypeCode::BYTE, false),
            Instruction::r#return(true, 0, TypeCode::BYTE),
        ],
        positions: vec![Span(0, 4), Span(4, 4)],
        ..Chunk::default()
    };
    let return_value = Some(Value::byte(0x2a));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_character() {
    let source = "'a'";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Character),
        instructions: vec![
            Instruction::load_constant(0, 0, TypeCode::CHARACTER, false),
            Instruction::r#return(true, 0, TypeCode::CHARACTER),
        ],
        positions: vec![Span(0, 3), Span(3, 3)],
        constants: vec![ConcreteValue::Character('a')],
        ..Chunk::default()
    };
    let return_value = Some(Value::character('a'));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_float() {
    let source = "42.42";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Float),
        instructions: vec![
            Instruction::load_constant(0, 0, TypeCode::FLOAT, false),
            Instruction::r#return(true, 0, TypeCode::FLOAT),
        ],
        positions: vec![Span(0, 5), Span(5, 5)],
        constants: vec![ConcreteValue::Float(42.42)],
        ..Chunk::default()
    };
    let return_value = Some(Value::float(42.42));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_integer() {
    let source = "42";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::Integer),
        instructions: vec![
            Instruction::load_constant(0, 0, TypeCode::INTEGER, false),
            Instruction::r#return(true, 0, TypeCode::INTEGER),
        ],
        positions: vec![Span(0, 2), Span(2, 2)],
        constants: vec![ConcreteValue::Integer(42)],
        ..Chunk::default()
    };
    let return_value = Some(Value::integer(42));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_string() {
    let source = "\"Hello, World!\"";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::String),
        instructions: vec![
            Instruction::load_constant(0, 0, TypeCode::STRING, false),
            Instruction::r#return(true, 0, TypeCode::STRING),
        ],
        positions: vec![Span(0, 15), Span(15, 15)],
        constants: vec![ConcreteValue::String(DustString::from("Hello, World!"))],
        ..Chunk::default()
    };
    let return_value = Some(Value::string("Hello, World!"));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_boolean_list() {
    let source = "[true, false]";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::List(TypeCode::BOOLEAN)),
        instructions: vec![
            Instruction::load_encoded(0, true as u8, TypeCode::BOOLEAN, false),
            Instruction::load_encoded(1, false as u8, TypeCode::BOOLEAN, false),
            Instruction::load_list(0, TypeCode::BOOLEAN, 0, 1, false),
            Instruction::r#return(true, 0, TypeCode::LIST),
        ],
        positions: vec![Span(1, 5), Span(7, 12), Span(0, 13), Span(13, 13)],
        ..Chunk::default()
    };
    let return_value = Some(Value::Concrete(ConcreteValue::List(vec![
        ConcreteValue::Boolean(true),
        ConcreteValue::Boolean(false),
    ])));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_byte_list() {
    let source = "[0x2a, 0x42]";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::List(TypeCode::BYTE)),
        instructions: vec![
            Instruction::load_encoded(0, 0x2a, TypeCode::BYTE, false),
            Instruction::load_encoded(1, 0x42, TypeCode::BYTE, false),
            Instruction::load_list(0, TypeCode::BYTE, 0, 1, false),
            Instruction::r#return(true, 0, TypeCode::LIST),
        ],
        positions: vec![Span(1, 5), Span(7, 11), Span(0, 12), Span(12, 12)],
        ..Chunk::default()
    };
    let return_value = Some(Value::Concrete(ConcreteValue::List(vec![
        ConcreteValue::Byte(0x2a),
        ConcreteValue::Byte(0x42),
    ])));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_character_list() {
    let source = "['a', 'b']";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::List(TypeCode::CHARACTER)),
        instructions: vec![
            Instruction::load_constant(0, 0, TypeCode::CHARACTER, false),
            Instruction::load_constant(1, 1, TypeCode::CHARACTER, false),
            Instruction::load_list(0, TypeCode::CHARACTER, 0, 1, false),
            Instruction::r#return(true, 0, TypeCode::LIST),
        ],
        positions: vec![Span(1, 4), Span(6, 9), Span(0, 10), Span(10, 10)],
        constants: vec![ConcreteValue::Character('a'), ConcreteValue::Character('b')],
        ..Chunk::default()
    };
    let return_value = Some(Value::Concrete(ConcreteValue::List(vec![
        ConcreteValue::Character('a'),
        ConcreteValue::Character('b'),
    ])));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_float_list() {
    let source = "[42.42, 24.24]";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::List(TypeCode::FLOAT)),
        instructions: vec![
            Instruction::load_constant(0, 0, TypeCode::FLOAT, false),
            Instruction::load_constant(1, 1, TypeCode::FLOAT, false),
            Instruction::load_list(0, TypeCode::FLOAT, 0, 1, false),
            Instruction::r#return(true, 0, TypeCode::LIST),
        ],
        positions: vec![Span(1, 6), Span(8, 13), Span(0, 14), Span(14, 14)],
        constants: vec![ConcreteValue::Float(42.42), ConcreteValue::Float(24.24)],
        ..Chunk::default()
    };
    let return_value = Some(Value::Concrete(ConcreteValue::List(vec![
        ConcreteValue::Float(42.42),
        ConcreteValue::Float(24.24),
    ])));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_integer_list() {
    let source = "[1, 2, 3]";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::List(TypeCode::INTEGER)),
        instructions: vec![
            Instruction::load_constant(0, 0, TypeCode::INTEGER, false),
            Instruction::load_constant(1, 1, TypeCode::INTEGER, false),
            Instruction::load_constant(2, 2, TypeCode::INTEGER, false),
            Instruction::load_list(0, TypeCode::INTEGER, 0, 2, false),
            Instruction::r#return(true, 0, TypeCode::LIST),
        ],
        positions: vec![Span(1, 2), Span(4, 5), Span(7, 8), Span(0, 9), Span(9, 9)],
        constants: vec![
            ConcreteValue::Integer(1),
            ConcreteValue::Integer(2),
            ConcreteValue::Integer(3),
        ],
        ..Chunk::default()
    };
    let return_value = Some(Value::Concrete(ConcreteValue::List(vec![
        ConcreteValue::Integer(1),
        ConcreteValue::Integer(2),
        ConcreteValue::Integer(3),
    ])));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_string_list() {
    let source = "[\"Hello\", \"World\"]";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::List(TypeCode::STRING)),
        instructions: vec![
            Instruction::load_constant(0, 0, TypeCode::STRING, false),
            Instruction::load_constant(1, 1, TypeCode::STRING, false),
            Instruction::load_list(0, TypeCode::STRING, 0, 1, false),
            Instruction::r#return(true, 0, TypeCode::LIST),
        ],
        positions: vec![Span(1, 8), Span(10, 17), Span(0, 18), Span(18, 18)],
        constants: vec![
            ConcreteValue::String(DustString::from("Hello")),
            ConcreteValue::String(DustString::from("World")),
        ],
        ..Chunk::default()
    };
    let return_value = Some(Value::Concrete(ConcreteValue::List(vec![
        ConcreteValue::String(DustString::from("Hello")),
        ConcreteValue::String(DustString::from("World")),
    ])));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_nested_list() {
    let source = "[[1, 2], [3, 4]]";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::List(TypeCode::LIST)),
        instructions: vec![
            Instruction::load_constant(0, 0, TypeCode::INTEGER, false),
            Instruction::load_constant(1, 1, TypeCode::INTEGER, false),
            Instruction::load_list(0, TypeCode::INTEGER, 0, 1, false),
            Instruction::load_constant(2, 2, TypeCode::INTEGER, false),
            Instruction::load_constant(3, 3, TypeCode::INTEGER, false),
            Instruction::load_list(1, TypeCode::INTEGER, 2, 3, false),
            Instruction::load_list(2, TypeCode::LIST, 0, 1, false),
            Instruction::r#return(true, 2, TypeCode::LIST),
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
        constants: vec![
            ConcreteValue::Integer(1),
            ConcreteValue::Integer(2),
            ConcreteValue::Integer(3),
            ConcreteValue::Integer(4),
        ],
        ..Chunk::default()
    };
    let return_value = Some(Value::Concrete(ConcreteValue::List(vec![
        ConcreteValue::List(vec![ConcreteValue::Integer(1), ConcreteValue::Integer(2)]),
        ConcreteValue::List(vec![ConcreteValue::Integer(3), ConcreteValue::Integer(4)]),
    ])));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_deeply_nested_list() {
    let source = "[[[1, 2], [3, 4]], [[5, 6], [7, 8]]]";
    let chunk = Chunk {
        r#type: FunctionType::new([], [], Type::List(TypeCode::LIST)),
        instructions: vec![
            Instruction::load_constant(0, 0, TypeCode::INTEGER, false),
            Instruction::load_constant(1, 1, TypeCode::INTEGER, false),
            Instruction::load_list(0, TypeCode::INTEGER, 0, 1, false),
            Instruction::load_constant(2, 2, TypeCode::INTEGER, false),
            Instruction::load_constant(3, 3, TypeCode::INTEGER, false),
            Instruction::load_list(1, TypeCode::INTEGER, 2, 3, false),
            Instruction::load_list(2, TypeCode::LIST, 0, 1, false),
            Instruction::close(0, 1, TypeCode::LIST),
            Instruction::load_constant(4, 4, TypeCode::INTEGER, false),
            Instruction::load_constant(5, 5, TypeCode::INTEGER, false),
            Instruction::load_list(3, TypeCode::INTEGER, 4, 5, false),
            Instruction::load_constant(6, 6, TypeCode::INTEGER, false),
            Instruction::load_constant(7, 7, TypeCode::INTEGER, false),
            Instruction::load_list(4, TypeCode::INTEGER, 6, 7, false),
            Instruction::load_list(5, TypeCode::LIST, 3, 4, false),
            Instruction::close(3, 4, TypeCode::LIST),
            Instruction::load_list(6, TypeCode::LIST, 2, 5, false),
            Instruction::r#return(true, 6, TypeCode::LIST),
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
        constants: vec![
            ConcreteValue::Integer(1),
            ConcreteValue::Integer(2),
            ConcreteValue::Integer(3),
            ConcreteValue::Integer(4),
            ConcreteValue::Integer(5),
            ConcreteValue::Integer(6),
            ConcreteValue::Integer(7),
            ConcreteValue::Integer(8),
        ],
        ..Chunk::default()
    };
    let return_value = Some(Value::Concrete(ConcreteValue::List(vec![
        ConcreteValue::List(vec![
            ConcreteValue::List(vec![ConcreteValue::Integer(1), ConcreteValue::Integer(2)]),
            ConcreteValue::List(vec![ConcreteValue::Integer(3), ConcreteValue::Integer(4)]),
        ]),
        ConcreteValue::List(vec![
            ConcreteValue::List(vec![ConcreteValue::Integer(5), ConcreteValue::Integer(6)]),
            ConcreteValue::List(vec![ConcreteValue::Integer(7), ConcreteValue::Integer(8)]),
        ]),
    ])));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

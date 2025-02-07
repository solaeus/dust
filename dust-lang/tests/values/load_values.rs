use dust_lang::{
    AbstractList, Chunk, ConcreteValue, DustString, FunctionType, Instruction, Span, Type, Value,
    compile, instruction::TypeCode, run, vm::Pointer,
};

#[test]
fn load_boolean_true() {
    let source = "true";
    let chunk = Chunk {
        r#type: FunctionType {
            return_type: Type::Boolean,
            ..FunctionType::default()
        },
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
        r#type: FunctionType {
            return_type: Type::Boolean,
            ..FunctionType::default()
        },
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
        r#type: FunctionType {
            return_type: Type::Byte,
            ..FunctionType::default()
        },
        instructions: vec![
            Instruction::load_encoded(0, 0x2a, TypeCode::BYTE, false),
            Instruction::r#return(true, 0, TypeCode::BYTE),
        ],
        positions: vec![Span(0, 6), Span(6, 6)],
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
        r#type: FunctionType {
            return_type: Type::Character,
            ..FunctionType::default()
        },
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
        r#type: FunctionType {
            return_type: Type::Float,
            ..FunctionType::default()
        },
        instructions: vec![
            Instruction::load_constant(0, 0, TypeCode::FLOAT, false),
            Instruction::r#return(true, 0, TypeCode::FLOAT),
        ],
        positions: vec![Span(0, 4), Span(4, 4)],
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
        r#type: FunctionType {
            return_type: Type::Integer,
            ..FunctionType::default()
        },
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
        r#type: FunctionType {
            return_type: Type::String,
            ..FunctionType::default()
        },
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
        r#type: FunctionType {
            return_type: Type::List(TypeCode::BOOLEAN),
            ..FunctionType::default()
        },
        instructions: vec![
            Instruction::load_encoded(0, true as u8, TypeCode::BOOLEAN, false),
            Instruction::load_encoded(1, false as u8, TypeCode::BOOLEAN, false),
            Instruction::load_list(0, TypeCode::BOOLEAN, 0, 2, false),
            Instruction::r#return(true, 0, TypeCode::LIST),
        ],
        positions: vec![Span(0, 13), Span(13, 13)],
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
        r#type: FunctionType {
            return_type: Type::List(TypeCode::BYTE),
            ..FunctionType::default()
        },
        instructions: vec![
            Instruction::load_encoded(0, 0x2a, TypeCode::BYTE, false),
            Instruction::load_encoded(1, 0x42, TypeCode::BYTE, false),
            Instruction::load_list(0, TypeCode::BYTE, 0, 2, false),
            Instruction::r#return(true, 0, TypeCode::LIST),
        ],
        positions: vec![Span(0, 15), Span(15, 15)],
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
        r#type: FunctionType {
            return_type: Type::List(TypeCode::CHARACTER),
            ..FunctionType::default()
        },
        instructions: vec![
            Instruction::load_constant(0, 0, TypeCode::CHARACTER, false),
            Instruction::load_constant(1, 1, TypeCode::CHARACTER, false),
            Instruction::load_list(0, TypeCode::CHARACTER, 0, 2, false),
            Instruction::r#return(true, 0, TypeCode::LIST),
        ],
        positions: vec![Span(0, 9), Span(9, 9)],
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
        r#type: FunctionType {
            return_type: Type::List(TypeCode::FLOAT),
            ..FunctionType::default()
        },
        instructions: vec![
            Instruction::load_constant(0, 0, TypeCode::FLOAT, false),
            Instruction::load_constant(1, 1, TypeCode::FLOAT, false),
            Instruction::load_list(0, TypeCode::FLOAT, 0, 2, false),
            Instruction::r#return(true, 0, TypeCode::LIST),
        ],
        positions: vec![Span(0, 15), Span(15, 15)],
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
        r#type: FunctionType {
            return_type: Type::List(TypeCode::INTEGER),
            ..FunctionType::default()
        },
        instructions: vec![
            Instruction::load_constant(0, 0, TypeCode::INTEGER, false),
            Instruction::load_constant(1, 1, TypeCode::INTEGER, false),
            Instruction::load_constant(2, 2, TypeCode::INTEGER, false),
            Instruction::load_list(0, TypeCode::INTEGER, 0, 3, false),
            Instruction::r#return(true, 0, TypeCode::LIST),
        ],
        positions: vec![Span(0, 9), Span(9, 9)],
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
        r#type: FunctionType {
            return_type: Type::List(TypeCode::STRING),
            ..FunctionType::default()
        },
        instructions: vec![
            Instruction::load_constant(0, 0, TypeCode::STRING, false),
            Instruction::load_constant(1, 1, TypeCode::STRING, false),
            Instruction::load_list(0, TypeCode::STRING, 0, 2, false),
            Instruction::r#return(true, 0, TypeCode::LIST),
        ],
        positions: vec![Span(0, 19), Span(19, 19)],
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
        r#type: FunctionType {
            return_type: Type::List(TypeCode::LIST),
            ..FunctionType::default()
        },
        instructions: vec![
            Instruction::load_constant(0, 0, TypeCode::INTEGER, false),
            Instruction::load_constant(1, 1, TypeCode::INTEGER, false),
            Instruction::load_list(0, TypeCode::INTEGER, 0, 2, false),
            Instruction::load_constant(2, 2, TypeCode::INTEGER, false),
            Instruction::load_constant(3, 3, TypeCode::INTEGER, false),
            Instruction::load_list(1, TypeCode::INTEGER, 2, 2, false),
            Instruction::load_list(2, TypeCode::LIST, 0, 2, false),
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

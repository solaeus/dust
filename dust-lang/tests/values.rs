use std::sync::Arc;

use dust_lang::{
    Chunk, ConcreteValue, DustString, Function, FunctionType, Instruction, Local, Scope, Span,
    Type, Value, compile, instruction::TypeCode, run,
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
        character_constants: vec!['a'],
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
        float_constants: vec![42.42],
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
        integer_constants: vec![42],
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
        string_constants: vec![DustString::from("Hello, World!")],
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
        character_constants: vec!['a', 'b'],
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
        float_constants: vec![42.42, 24.24],
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
        integer_constants: vec![1, 2, 3],
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
        string_constants: vec![DustString::from("Hello"), DustString::from("World")],
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
        integer_constants: vec![1, 2, 3, 4],
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
        integer_constants: vec![1, 2, 3, 4, 5, 6, 7, 8],
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

#[test]
fn load_function() {
    let source = "fn () {}";
    let chunk = Chunk {
        r#type: FunctionType::new(
            [],
            [],
            Type::Function(FunctionType::new([], [], Type::None)),
        ),
        instructions: vec![
            Instruction::load_function(0, 0, false),
            Instruction::r#return(true, 0, TypeCode::FUNCTION),
        ],
        positions: vec![Span(0, 8), Span(8, 8)],
        prototypes: vec![Arc::new(Chunk {
            instructions: vec![Instruction::r#return(false, 0, TypeCode::NONE)],
            positions: vec![Span(8, 8)],
            ..Default::default()
        })],
        ..Default::default()
    };
    let return_value = Some(Value::Function(Function {
        name: None,
        r#type: FunctionType::new([], [], Type::None),
        prototype_index: 0,
    }));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_boolean_in_function() {
    let source = "fn () { true }";
    let chunk = Chunk {
        r#type: FunctionType::new(
            [],
            [],
            Type::Function(FunctionType::new([], [], Type::Boolean)),
        ),
        instructions: vec![
            Instruction::load_function(0, 0, false),
            Instruction::r#return(true, 0, TypeCode::FUNCTION),
        ],
        positions: vec![Span(0, 14), Span(14, 14)],
        prototypes: vec![Arc::new(Chunk {
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::load_encoded(0, true as u8, TypeCode::BOOLEAN, false),
                Instruction::r#return(true, 0, TypeCode::BOOLEAN),
            ],
            positions: vec![Span(8, 12), Span(13, 14)],
            ..Default::default()
        })],
        ..Default::default()
    };
    let return_value = Some(Value::Function(Function {
        name: None,
        r#type: FunctionType::new([], [], Type::Boolean),
        prototype_index: 0,
    }));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_integer_in_function() {
    let source = "fn () { 42 }";
    let chunk = Chunk {
        r#type: FunctionType::new(
            [],
            [],
            Type::Function(FunctionType::new([], [], Type::Integer)),
        ),
        instructions: vec![
            Instruction::load_function(0, 0, false),
            Instruction::r#return(true, 0, TypeCode::FUNCTION),
        ],
        positions: vec![Span(0, 12), Span(12, 12)],
        prototypes: vec![Arc::new(Chunk {
            r#type: FunctionType::new([], [], Type::Integer),
            instructions: vec![
                Instruction::load_constant(0, 0, TypeCode::INTEGER, false),
                Instruction::r#return(true, 0, TypeCode::INTEGER),
            ],
            positions: vec![Span(8, 10), Span(11, 12)],
            integer_constants: vec![42],
            ..Default::default()
        })],
        ..Default::default()
    };
    let return_value = Some(Value::Function(Function {
        name: None,
        r#type: FunctionType::new([], [], Type::Integer),
        prototype_index: 0,
    }));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_string_in_function() {
    let source = "fn () { \"Hello\" }";
    let chunk = Chunk {
        r#type: FunctionType::new(
            [],
            [],
            Type::Function(FunctionType::new([], [], Type::String)),
        ),
        instructions: vec![
            Instruction::load_function(0, 0, false),
            Instruction::r#return(true, 0, TypeCode::FUNCTION),
        ],
        positions: vec![Span(0, 17), Span(17, 17)],
        prototypes: vec![Arc::new(Chunk {
            r#type: FunctionType::new([], [], Type::String),
            instructions: vec![
                Instruction::load_constant(0, 0, TypeCode::STRING, false),
                Instruction::r#return(true, 0, TypeCode::STRING),
            ],
            positions: vec![Span(8, 15), Span(16, 17)],
            string_constants: vec![DustString::from("Hello")],
            ..Default::default()
        })],
        ..Default::default()
    };
    let return_value = Some(Value::Function(Function {
        name: None,
        r#type: FunctionType::new([], [], Type::String),
        prototype_index: 0,
    }));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_list_in_function() {
    let source = "fn () { [1, 2, 3] }";
    let chunk = Chunk {
        r#type: FunctionType::new(
            [],
            [],
            Type::Function(FunctionType::new([], [], Type::List(TypeCode::INTEGER))),
        ),
        instructions: vec![
            Instruction::load_function(0, 0, false),
            Instruction::r#return(true, 0, TypeCode::FUNCTION),
        ],
        positions: vec![Span(0, 19), Span(19, 19)],
        prototypes: vec![Arc::new(Chunk {
            r#type: FunctionType::new([], [], Type::List(TypeCode::INTEGER)),
            instructions: vec![
                Instruction::load_constant(0, 0, TypeCode::INTEGER, false),
                Instruction::load_constant(1, 1, TypeCode::INTEGER, false),
                Instruction::load_constant(2, 2, TypeCode::INTEGER, false),
                Instruction::load_list(0, TypeCode::INTEGER, 0, 2, false),
                Instruction::r#return(true, 0, TypeCode::LIST),
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
        })],
        ..Default::default()
    };
    let return_value = Some(Value::Function(Function {
        name: None,
        r#type: FunctionType::new([], [], Type::List(TypeCode::INTEGER)),
        prototype_index: 0,
    }));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_byte_in_function() {
    let source = "fn () { 0x2a }";
    let chunk = Chunk {
        r#type: FunctionType::new(
            [],
            [],
            Type::Function(FunctionType::new([], [], Type::Byte)),
        ),
        instructions: vec![
            Instruction::load_function(0, 0, false),
            Instruction::r#return(true, 0, TypeCode::FUNCTION),
        ],
        positions: vec![Span(0, 14), Span(14, 14)],
        prototypes: vec![Arc::new(Chunk {
            r#type: FunctionType::new([], [], Type::Byte),
            instructions: vec![
                Instruction::load_encoded(0, 0x2a, TypeCode::BYTE, false),
                Instruction::r#return(true, 0, TypeCode::BYTE),
            ],
            positions: vec![Span(8, 12), Span(13, 14)],
            ..Default::default()
        })],
        ..Default::default()
    };
    let return_value = Some(Value::Function(Function {
        name: None,
        r#type: FunctionType::new([], [], Type::Byte),
        prototype_index: 0,
    }));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_character_in_function() {
    let source = "fn () { 'a' }";
    let chunk = Chunk {
        r#type: FunctionType::new(
            [],
            [],
            Type::Function(FunctionType::new([], [], Type::Character)),
        ),
        instructions: vec![
            Instruction::load_function(0, 0, false),
            Instruction::r#return(true, 0, TypeCode::FUNCTION),
        ],
        positions: vec![Span(0, 13), Span(13, 13)],
        prototypes: vec![Arc::new(Chunk {
            r#type: FunctionType::new([], [], Type::Character),
            instructions: vec![
                Instruction::load_constant(0, 0, TypeCode::CHARACTER, false),
                Instruction::r#return(true, 0, TypeCode::CHARACTER),
            ],
            positions: vec![Span(8, 11), Span(12, 13)],
            character_constants: vec!['a'],
            ..Default::default()
        })],
        ..Default::default()
    };
    let return_value = Some(Value::Function(Function {
        name: None,
        r#type: FunctionType::new([], [], Type::Character),
        prototype_index: 0,
    }));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_float_in_function() {
    let source = "fn () { 42.42 }";
    let chunk = Chunk {
        r#type: FunctionType::new(
            [],
            [],
            Type::Function(FunctionType::new([], [], Type::Float)),
        ),
        instructions: vec![
            Instruction::load_function(0, 0, false),
            Instruction::r#return(true, 0, TypeCode::FUNCTION),
        ],
        positions: vec![Span(0, 15), Span(15, 15)],
        prototypes: vec![Arc::new(Chunk {
            r#type: FunctionType::new([], [], Type::Float),
            instructions: vec![
                Instruction::load_constant(0, 0, TypeCode::FLOAT, false),
                Instruction::r#return(true, 0, TypeCode::FLOAT),
            ],
            positions: vec![Span(8, 13), Span(14, 15)],
            float_constants: vec![42.42],
            ..Default::default()
        })],
        ..Default::default()
    };
    let return_value = Some(Value::Function(Function {
        name: None,
        r#type: FunctionType::new([], [], Type::Float),
        prototype_index: 0,
    }));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_nested_list_in_function() {
    let source = "fn () { [[1, 2], [3, 4]] }";
    let chunk = Chunk {
        r#type: FunctionType::new(
            [],
            [],
            Type::Function(FunctionType::new([], [], Type::List(TypeCode::LIST))),
        ),
        instructions: vec![
            Instruction::load_function(0, 0, false),
            Instruction::r#return(true, 0, TypeCode::FUNCTION),
        ],
        positions: vec![Span(0, 26), Span(26, 26)],
        prototypes: vec![Arc::new(Chunk {
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
        })],
        ..Default::default()
    };
    let return_value = Some(Value::Function(Function {
        name: None,
        r#type: FunctionType::new([], [], Type::List(TypeCode::LIST)),
        prototype_index: 0,
    }));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_deeply_nested_list_in_function() {
    let source = "fn () { [[[1, 2], [3, 4]], [[5, 6], [7, 8]]] }";
    let chunk = Chunk {
        r#type: FunctionType::new(
            [],
            [],
            Type::Function(FunctionType::new([], [], Type::List(TypeCode::LIST))),
        ),
        instructions: vec![
            Instruction::load_function(0, 0, false),
            Instruction::r#return(true, 0, TypeCode::FUNCTION),
        ],
        positions: vec![Span(0, 46), Span(46, 46)],
        prototypes: vec![Arc::new(Chunk {
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
            ..Chunk::default()
        })],
        ..Default::default()
    };
    let return_value = Some(Value::Function(Function {
        name: None,
        r#type: FunctionType::new([], [], Type::List(TypeCode::LIST)),
        prototype_index: 0,
    }));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

#[test]
fn load_function_in_function() {
    let source = "fn outer() { fn inner() -> int { 42 } }";
    let chunk = Chunk {
        r#type: FunctionType::new(
            [],
            [],
            Type::function([], [], Type::function([], [], Type::Integer)),
        ),
        instructions: vec![
            Instruction::load_function(0, 0, false),
            Instruction::r#return(true, 0, TypeCode::FUNCTION),
        ],
        positions: vec![Span(0, 39), Span(39, 39)],
        locals: vec![Local::new(
            0,
            0,
            Type::function([], [], Type::function([], [], Type::Integer)),
            false,
            Scope::new(0, 0),
        )],
        string_constants: vec![DustString::from("outer")],
        prototypes: vec![Arc::new(Chunk {
            name: Some(DustString::from("outer")),
            r#type: FunctionType::new([], [], Type::function([], [], Type::Integer)),
            instructions: vec![
                Instruction::load_function(0, 0, false),
                Instruction::r#return(true, 0, TypeCode::FUNCTION),
            ],
            positions: vec![Span(13, 37), Span(38, 39)],
            locals: vec![Local::new(
                0,
                0,
                Type::Function(FunctionType::new([], [], Type::Integer)),
                false,
                Scope::new(0, 0),
            )],
            string_constants: vec![DustString::from("inner")],
            prototypes: vec![Arc::new(Chunk {
                name: Some(DustString::from("inner")),
                r#type: FunctionType::new([], [], Type::Integer),
                instructions: vec![
                    Instruction::load_constant(0, 0, TypeCode::INTEGER, false),
                    Instruction::r#return(true, 0, TypeCode::INTEGER),
                ],
                positions: vec![Span(33, 35), Span(36, 37)],
                integer_constants: vec![42],
                ..Default::default()
            })],
            ..Default::default()
        })],
        ..Default::default()
    };
    let return_value = Some(Value::Function(Function {
        name: Some(DustString::from("outer")),
        r#type: FunctionType::new([], [], Type::function([], [], Type::Integer)),
        prototype_index: 0,
    }));

    assert_eq!(chunk, compile(source).unwrap());
    assert_eq!(return_value, run(source).unwrap());
}

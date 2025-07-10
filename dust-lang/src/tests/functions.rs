use crate::{
    Address, BlockScope, DebugChunk, FunctionType, Instruction, Local, OperandType, Path,
    StrippedChunk, Type, Value, compile, run,
};

use indexmap::indexmap;

#[test]
fn function_returns_boolean() {
    let source = "fn foo() { true } foo()";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::call(
                    Address::register(0),
                    Address::constant(0),
                    0,
                    OperandType::BOOLEAN
                ),
                Instruction::r#return(true, Address::register(0), OperandType::BOOLEAN)
            ],
            locals: indexmap! {
                Path::new("foo").unwrap() => Local::new(
                    Address::constant(0),
                    Type::Function(Box::new(FunctionType::new([], [], Type::Boolean))),
                    false,
                    BlockScope::new(0,0)
                ),
            },
            constants: vec![Value::function(DebugChunk {
                name: Some(Path::new("foo").unwrap()),
                r#type: FunctionType::new([], [], Type::Boolean),
                instructions: vec![Instruction::r#return(
                    true,
                    Address::encoded(1),
                    OperandType::BOOLEAN
                )],
                constants: vec![],
                ..Default::default()
            })],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Boolean(true)));
}

#[test]
fn function_returns_byte() {
    let source = "fn foo() { 0x64 } foo()";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Byte),
            instructions: vec![
                Instruction::call(
                    Address::register(0),
                    Address::constant(0),
                    0,
                    OperandType::BYTE
                ),
                Instruction::r#return(true, Address::register(0), OperandType::BYTE)
            ],
            locals: indexmap! {
                Path::new("foo").unwrap() => Local::new(
                    Address::constant(0),
                    Type::Function(Box::new(FunctionType::new([], [], Type::Byte))),
                    false,
                    BlockScope::new(0,0)
                ),
            },
            constants: vec![Value::function(DebugChunk {
                name: Some(Path::new("foo").unwrap()),
                r#type: FunctionType::new([], [], Type::Byte),
                instructions: vec![Instruction::r#return(
                    true,
                    Address::encoded(0x64),
                    OperandType::BYTE
                )],
                constants: vec![],
                ..Default::default()
            })],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Byte(0x64)));
}

#[test]
fn function_returns_character() {
    let source = "fn foo() { 'a' } foo()";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Character),
            instructions: vec![
                Instruction::call(
                    Address::register(0),
                    Address::constant(0),
                    0,
                    OperandType::CHARACTER
                ),
                Instruction::r#return(true, Address::register(0), OperandType::CHARACTER)
            ],
            locals: indexmap! {
                Path::new("foo").unwrap() => Local::new(
                    Address::constant(0),
                    Type::Function(Box::new(FunctionType::new([], [], Type::Character))),
                    false,
                    BlockScope::new(0,0)
                ),
            },
            constants: vec![Value::function(DebugChunk {
                name: Some(Path::new("foo").unwrap()),
                r#type: FunctionType::new([], [], Type::Character),
                instructions: vec![Instruction::r#return(
                    true,
                    Address::constant(0),
                    OperandType::CHARACTER
                )],
                constants: vec![Value::character('a')],
                ..Default::default()
            })],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Character('a')));
}

#[test]
fn function_returns_float() {
    let source = "fn foo() { 42.0 } foo()";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Float),
            instructions: vec![
                Instruction::call(
                    Address::register(0),
                    Address::constant(0),
                    0,
                    OperandType::FLOAT
                ),
                Instruction::r#return(true, Address::register(0), OperandType::FLOAT)
            ],
            locals: indexmap! {
                Path::new("foo").unwrap() => Local::new(
                    Address::constant(0),
                    Type::Function(Box::new(FunctionType::new([], [], Type::Float))),
                    false,
                    BlockScope::new(0,0)
                ),
            },
            constants: vec![Value::function(DebugChunk {
                name: Some(Path::new("foo").unwrap()),
                r#type: FunctionType::new([], [], Type::Float),
                instructions: vec![Instruction::r#return(
                    true,
                    Address::constant(0),
                    OperandType::FLOAT
                )],
                constants: vec![Value::Float(42.0)],
                ..Default::default()
            })],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Float(42.0)));
}

#[test]
fn function_returns_integer() {
    let source = "fn foo() { 42 } foo()";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::Integer),
            instructions: vec![
                Instruction::call(
                    Address::register(0),
                    Address::constant(0),
                    0,
                    OperandType::INTEGER
                ),
                Instruction::r#return(true, Address::register(0), OperandType::INTEGER)
            ],
            locals: indexmap! {
                Path::new("foo").unwrap() => Local::new(
                    Address::constant(0),
                    Type::Function(Box::new(FunctionType::new([], [], Type::Integer))),
                    false,
                    BlockScope::new(0,0)
                ),
            },
            constants: vec![Value::function(DebugChunk {
                name: Some(Path::new("foo").unwrap()),
                r#type: FunctionType::new([], [], Type::Integer),
                instructions: vec![Instruction::r#return(
                    true,
                    Address::constant(0),
                    OperandType::INTEGER
                )],
                constants: vec![Value::Integer(42)],
                ..Default::default()
            })],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::Integer(42)));
}

#[test]
fn function_returns_string() {
    let source = "fn foo() { \"Hello, World!\" } foo()";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::String),
            instructions: vec![
                Instruction::call(
                    Address::register(0),
                    Address::constant(0),
                    0,
                    OperandType::STRING
                ),
                Instruction::r#return(true, Address::register(0), OperandType::STRING)
            ],
            locals: indexmap! {
                Path::new("foo").unwrap() => Local::new(
                    Address::constant(0),
                    Type::Function(Box::new(FunctionType::new([], [], Type::String))),
                    false,
                    BlockScope::new(0,0)
                ),
            },
            constants: vec![Value::function(DebugChunk {
                name: Some(Path::new("foo").unwrap()),
                r#type: FunctionType::new([], [], Type::String),
                instructions: vec![Instruction::r#return(
                    true,
                    Address::constant(0),
                    OperandType::STRING
                )],
                constants: vec![Value::string("Hello, World!")],
                ..Default::default()
            })],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::string("Hello, World!")));
}

#[test]
fn function_returns_list_of_integers() {
    let source = "fn foo() { [42, 100] } foo()";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new([], [], Type::List(Box::new(Type::Integer))),
            instructions: vec![
                Instruction::call(
                    Address::register(0),
                    Address::constant(0),
                    0,
                    OperandType::LIST_INTEGER
                ),
                Instruction::r#return(true, Address::register(0), OperandType::LIST_INTEGER)
            ],
            locals: indexmap! {
                Path::new("foo").unwrap() => Local::new(
                    Address::constant(0),
                    Type::Function(Box::new(FunctionType::new([], [], Type::List(Box::new(Type::Integer))))),
                    false,
                    BlockScope::new(0,0)
                ),
            },
            constants: vec![Value::function(DebugChunk {
                name: Some(Path::new("foo").unwrap()),
                r#type: FunctionType::new([], [], Type::List(Box::new(Type::Integer))),
                instructions: vec![
                    Instruction::load(
                        Address::register(0),
                        Address::constant(0),
                        OperandType::INTEGER,
                        false
                    ),
                    Instruction::load(
                        Address::register(1),
                        Address::constant(1),
                        OperandType::INTEGER,
                        false
                    ),
                    Instruction::list(
                        Address::register(2),
                        Address::register(0),
                        Address::register(1),
                        OperandType::LIST_INTEGER,
                    ),
                    Instruction::r#return(true, Address::register(2), OperandType::LIST_INTEGER)
                ],
                constants: vec![Value::Integer(42), Value::Integer(100)],
                ..Default::default()
            })],
            ..Default::default()
        }
    );
    assert_eq!(return_value, Some(Value::integer_list([42, 100])));
}

#[test]
fn function_returns_function() {
    let source = "fn foo() { fn() { 42 } } foo()";
    let chunk = compile::<DebugChunk>(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        chunk,
        DebugChunk {
            name: Some(Path::new("main").unwrap()),
            r#type: FunctionType::new(
                [],
                [],
                Type::Function(Box::new(FunctionType::new([], [], Type::Integer)))
            ),
            instructions: vec![
                Instruction::call(
                    Address::register(0),
                    Address::constant(0),
                    0,
                    OperandType::FUNCTION
                ),
                Instruction::r#return(true, Address::register(0), OperandType::FUNCTION)
            ],
            locals: indexmap! {
                Path::new("foo").unwrap() => Local::new(
                    Address::constant(0),
                    Type::Function(Box::new(FunctionType::new([], [], Type::Function(Box::new(FunctionType::new([], [], Type::Integer)))))),
                    false,
                    BlockScope::new(0,0)
                ),
            },
            constants: vec![Value::function(DebugChunk {
                name: Some(Path::new("foo").unwrap()),
                r#type: FunctionType::new(
                    [],
                    [],
                    Type::Function(Box::new(FunctionType::new([], [], Type::Integer)))
                ),
                instructions: vec![Instruction::r#return(
                    true,
                    Address::constant(0),
                    OperandType::FUNCTION
                )],
                constants: vec![Value::function(DebugChunk {
                    name: None,
                    r#type: FunctionType::new([], [], Type::Integer),
                    instructions: vec![Instruction::r#return(
                        true,
                        Address::constant(0),
                        OperandType::INTEGER
                    )],
                    constants: vec![Value::Integer(42)],
                    ..Default::default()
                })],
                ..Default::default()
            })],
            ..Default::default()
        }
    );
    assert_eq!(
        return_value,
        Some(Value::function(StrippedChunk {
            r#type: FunctionType::new([], [], Type::Integer),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::INTEGER
            )],
            constants: vec![Value::Integer(42)],
            ..Default::default()
        }))
    );
}

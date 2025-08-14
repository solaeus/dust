use crate::{
    Address, Chunk, FunctionType, Instruction, OperandType, Path, Program, Type, Value, compile,
    run,
};

#[test]
fn boolean_true() {
    let source = "true";
    let program = compile(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        program,
        Program {
            main_chunk: Chunk {
                name: Some(Path::new("main").unwrap()),
                r#type: FunctionType::new([], [], Type::Boolean),
                instructions: vec![Instruction::r#return(
                    true,
                    Address::encoded(true as u16),
                    OperandType::BOOLEAN
                )],
                prototype_index: u16::MAX,
                ..Default::default()
            },
            cell_count: 0,
            prototypes: Vec::new(),
        }
    );
    assert_eq!(return_value, Some(Value::Boolean(true)));
}

#[test]
fn boolean_false() {
    let source = "false";
    let program = compile(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        program,
        Program {
            main_chunk: Chunk {
                name: Some(Path::new("main").unwrap()),
                r#type: FunctionType::new([], [], Type::Boolean),
                instructions: vec![Instruction::r#return(
                    true,
                    Address::encoded(false as u16),
                    OperandType::BOOLEAN
                )],
                prototype_index: u16::MAX,
                ..Default::default()
            },
            cell_count: 0,
            prototypes: Vec::new(),
        }
    );
    assert_eq!(return_value, Some(Value::Boolean(false)));
}

#[test]
fn byte() {
    let source = "0xFF";
    let program = compile(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        program,
        Program {
            main_chunk: Chunk {
                name: Some(Path::new("main").unwrap()),
                r#type: FunctionType::new([], [], Type::Byte),
                instructions: vec![Instruction::r#return(
                    true,
                    Address::encoded(255),
                    OperandType::BYTE
                )],
                prototype_index: u16::MAX,
                ..Default::default()
            },
            cell_count: 0,
            prototypes: Vec::new(),
        }
    );
    assert_eq!(return_value, Some(Value::Byte(255)));
}

#[test]
fn character() {
    let source = "'a'";
    let program = compile(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        program,
        Program {
            main_chunk: Chunk {
                name: Some(Path::new("main").unwrap()),
                r#type: FunctionType::new([], [], Type::Character),
                instructions: vec![Instruction::r#return(
                    true,
                    Address::constant(0),
                    OperandType::CHARACTER
                )],
                constants: vec![Value::Character('a')],
                prototype_index: u16::MAX,
                ..Default::default()
            },
            cell_count: 0,
            prototypes: Vec::new(),
        }
    );
    assert_eq!(return_value, Some(Value::Character('a')));
}

#[test]
fn float() {
    let source = "42.0";
    let program = compile(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        program,
        Program {
            main_chunk: Chunk {
                name: Some(Path::new("main").unwrap()),
                r#type: FunctionType::new([], [], Type::Float),
                instructions: vec![Instruction::r#return(
                    true,
                    Address::constant(0),
                    OperandType::FLOAT
                )],
                constants: vec![Value::Float(42.0)],
                prototype_index: u16::MAX,
                ..Default::default()
            },
            cell_count: 0,
            prototypes: Vec::new(),
        }
    );
    assert_eq!(return_value, Some(Value::Float(42.0)));
}

#[test]
fn integer() {
    let source = "42";
    let program = compile(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        program,
        Program {
            main_chunk: Chunk {
                name: Some(Path::new("main").unwrap()),
                r#type: FunctionType::new([], [], Type::Integer),
                instructions: vec![Instruction::r#return(
                    true,
                    Address::constant(0),
                    OperandType::INTEGER
                )],
                constants: vec![Value::Integer(42)],
                prototype_index: u16::MAX,
                ..Default::default()
            },
            cell_count: 0,
            prototypes: Vec::new(),
        }
    );
    assert_eq!(return_value, Some(Value::Integer(42)));
}

#[test]
fn string() {
    let source = "\"foobar\"";
    let program = compile(source).unwrap();
    let return_value = run(source).unwrap();

    assert_eq!(
        program,
        Program {
            main_chunk: Chunk {
                name: Some(Path::new("main").unwrap()),
                r#type: FunctionType::new([], [], Type::String),
                instructions: vec![Instruction::r#return(
                    true,
                    Address::constant(0),
                    OperandType::STRING
                )],
                constants: vec![Value::String("foobar".to_string())],
                prototype_index: u16::MAX,
                ..Default::default()
            },
            cell_count: 0,
            prototypes: Vec::new(),
        }
    );
    assert_eq!(return_value, Some(Value::String("foobar".to_string())));
}

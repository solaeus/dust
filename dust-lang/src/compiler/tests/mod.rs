use crate::{
    Address, Chunk, ConstantTable, FunctionType, Instruction, OperandType, Type, compile,
    tests::cases,
};

#[test]
fn boolean() {
    let source = cases::BOOLEAN;
    let chunk = compile(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![Instruction::r#return(
                true,
                Address::encoded(true as u16),
                OperandType::BOOLEAN
            )],
            ..Default::default()
        }
    );
}

#[test]
fn byte() {
    let source = cases::BYTE;
    let chunk = compile(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::Byte),
            instructions: vec![Instruction::r#return(
                true,
                Address::encoded(42),
                OperandType::BYTE
            )],
            ..Default::default()
        }
    );
}

#[test]
fn character() {
    let source = cases::CHARACTER;
    let chunk = compile(source).unwrap();
    let mut constants = ConstantTable::new();

    constants.add_character('q');

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::Character),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::CHARACTER
            )],
            constants,
            ..Default::default()
        }
    );
}

#[test]
fn float() {
    let source = cases::FLOAT;
    let chunk = compile(source).unwrap();
    let mut constants = ConstantTable::new();

    constants.add_float(42.0);

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::Float),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::FLOAT
            )],
            constants,
            ..Default::default()
        }
    );
}

#[test]
fn integer() {
    let source = cases::INTEGER;
    let chunk = compile(source).unwrap();
    let mut constants = ConstantTable::new();

    constants.add_integer(42);

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::Integer),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::INTEGER
            )],
            constants,
            ..Default::default()
        }
    );
}

#[test]
fn string() {
    let source = cases::STRING;
    let chunk = compile(source).unwrap();
    let mut constants = ConstantTable::new();

    constants.add_string("foobar");
    constants.trim_string_pool();

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::String),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::STRING
            )],
            constants,
            ..Default::default()
        }
    );
}

#[test]
fn constant_byte_addition() {
    let source = cases::CONSTANT_BYTE_ADDITION;
    let chunk = compile(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::Byte),
            instructions: vec![Instruction::r#return(
                true,
                Address::encoded(42),
                OperandType::BYTE
            )],
            ..Default::default()
        }
    );
}

#[test]
fn constant_float_addition() {
    let source = cases::CONSTANT_FLOAT_ADDITION;
    let chunk = compile(source).unwrap();
    let mut constants = ConstantTable::new();

    constants.add_float(42.0);

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::Float),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::FLOAT
            )],
            constants,
            ..Default::default()
        }
    );
}

#[test]
fn constant_integer_addition() {
    let source = cases::CONSTANT_INTEGER_ADDITION;
    let chunk = compile(source).unwrap();
    let mut constants = ConstantTable::new();

    constants.add_integer(42);

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::Integer),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::INTEGER
            )],
            constants,
            ..Default::default()
        }
    );
}

#[test]
fn constant_byte_subtraction() {
    let source = cases::CONSTANT_BYTE_SUBTRACTION;
    let chunk = compile(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::Byte),
            instructions: vec![Instruction::r#return(
                true,
                Address::encoded(42),
                OperandType::BYTE
            )],
            ..Default::default()
        }
    );
}

#[test]
fn constant_float_subtraction() {
    let source = cases::CONSTANT_FLOAT_SUBTRACTION;
    let chunk = compile(source).unwrap();
    let mut constants = ConstantTable::new();

    constants.add_float(42.0);

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::Float),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::FLOAT
            )],
            constants,
            ..Default::default()
        }
    );
}

#[test]
fn constant_integer_subtraction() {
    let source = cases::CONSTANT_INTEGER_SUBTRACTION;
    let chunk = compile(source).unwrap();
    let mut constants = ConstantTable::new();

    constants.add_integer(42);

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::Integer),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::INTEGER
            )],
            constants,
            ..Default::default()
        }
    );
}

#[test]
fn constant_byte_multiplication() {
    let source = cases::CONSTANT_BYTE_MULTIPLICATION;
    let chunk = compile(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::Byte),
            instructions: vec![Instruction::r#return(
                true,
                Address::encoded(42),
                OperandType::BYTE
            )],
            ..Default::default()
        }
    );
}

#[test]
fn constant_float_multiplication() {
    let source = cases::CONSTANT_FLOAT_MULTIPLICATION;
    let chunk = compile(source).unwrap();
    let mut constants = ConstantTable::new();

    constants.add_float(42.0);

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::Float),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::FLOAT
            )],
            constants,
            ..Default::default()
        }
    );
}

#[test]
fn constant_integer_multiplication() {
    let source = cases::CONSTANT_INTEGER_MULTIPLICATION;
    let chunk = compile(source).unwrap();
    let mut constants = ConstantTable::new();

    constants.add_integer(42);

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::Integer),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::INTEGER
            )],
            constants,
            ..Default::default()
        }
    );
}

#[test]
fn constant_byte_division() {
    let source = cases::CONSTANT_BYTE_DIVISION;
    let chunk = compile(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::Byte),
            instructions: vec![Instruction::r#return(
                true,
                Address::encoded(42),
                OperandType::BYTE
            )],
            ..Default::default()
        }
    );
}

#[test]
fn constant_float_division() {
    let source = cases::CONSTANT_FLOAT_DIVISION;
    let chunk = compile(source).unwrap();
    let mut constants = ConstantTable::new();

    constants.add_float(42.0);

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::Float),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::FLOAT
            )],
            constants,
            ..Default::default()
        }
    );
}

#[test]
fn constant_integer_division() {
    let source = cases::CONSTANT_INTEGER_DIVISION;
    let chunk = compile(source).unwrap();
    let mut constants = ConstantTable::new();

    constants.add_integer(42);

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::Integer),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::INTEGER
            )],
            constants,
            ..Default::default()
        }
    );
}

#[test]
fn constant_string_concatenation() {
    let source = cases::CONSTANT_STRING_CONCATENATION;
    let chunk = compile(source).unwrap();
    let mut constants = ConstantTable::new();

    constants.add_string("foobar");
    constants.trim_string_pool();

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::String),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::STRING
            )],
            constants,
            ..Default::default()
        }
    );
}

#[test]
fn constant_character_concatentation() {
    let source = cases::CONSTANT_CHARACTER_CONCATENATION;
    let chunk = compile(source).unwrap();
    let mut constants = ConstantTable::new();

    constants.add_string("qq");
    constants.trim_string_pool();

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::String),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::STRING
            )],
            constants,
            ..Default::default()
        }
    );
}

#[test]
fn constant_string_character_concatenation() {
    let source = cases::CONSTANT_STRING_CHARACTER_CONCATENATION;
    let chunk = compile(source).unwrap();
    let mut constants = ConstantTable::new();

    constants.add_string("fooq");
    constants.trim_string_pool();

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::String),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::STRING
            )],
            constants,
            ..Default::default()
        }
    );
}

#[test]
fn constant_character_string_concatenation() {
    let source = cases::CONSTANT_CHARACTER_STRING_CONCATENATION;
    let chunk = compile(source).unwrap();
    let mut constants = ConstantTable::new();

    constants.push_str_to_string_pool("foo");
    constants.add_string("qfoo");
    constants.trim_string_pool();

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::String),
            instructions: vec![Instruction::r#return(
                true,
                Address::constant(0),
                OperandType::STRING
            )],
            constants,
            ..Default::default()
        }
    );
}

#[test]
fn local_declaration() {
    let source = cases::LOCAL_DECLARATION;
    let chunk = compile(source).unwrap();
    let mut constants = ConstantTable::new();

    constants.add_integer(42);

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::None),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::constant(0),
                    OperandType::INTEGER,
                    false
                ),
                Instruction::r#return(false, Address::default(), OperandType::NONE),
            ],
            register_count: 1,
            constants,
            ..Default::default()
        }
    );
}

#[test]
fn local_mut_declaration() {
    let source = cases::LOCAL_MUT_DECLARATION;
    let chunk = compile(source).unwrap();
    let mut constants = ConstantTable::new();

    constants.add_integer(42);

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::None),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::constant(0),
                    OperandType::INTEGER,
                    false
                ),
                Instruction::r#return(false, Address::default(), OperandType::NONE),
            ],
            register_count: 1,
            constants,
            ..Default::default()
        }
    );
}

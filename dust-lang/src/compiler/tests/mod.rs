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
fn constant_boolean_and() {
    let source = cases::CONSTANT_BOOLEAN_AND;
    let chunk = compile(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![Instruction::r#return(
                true,
                Address::encoded(false as u16),
                OperandType::BOOLEAN
            )],
            ..Default::default()
        }
    );
}

#[test]
fn constant_boolean_or() {
    let source = cases::CONSTANT_BOOLEAN_OR;
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

// Comparison and equality tests (in order after CONSTANT_BYTE_GREATER_THAN_OR_EQUAL)

#[test]
fn constant_byte_less_than_or_equal() {
    let source = cases::CONSTANT_BYTE_LESS_THAN_OR_EQUAL;
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
fn constant_byte_equal() {
    let source = cases::CONSTANT_BYTE_EQUAL;
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
fn constant_byte_not_equal() {
    let source = cases::CONSTANT_BYTE_NOT_EQUAL;
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
fn constant_character_greater_than() {
    let source = cases::CONSTANT_CHARACTER_GREATER_THAN;
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
fn constant_character_less_than() {
    let source = cases::CONSTANT_CHARACTER_LESS_THAN;
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
fn constant_character_greater_than_or_equal() {
    let source = cases::CONSTANT_CHARACTER_GREATER_THAN_OR_EQUAL;
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
fn constant_character_less_than_or_equal() {
    let source = cases::CONSTANT_CHARACTER_LESS_THAN_OR_EQUAL;
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
fn constant_character_equal() {
    let source = cases::CONSTANT_CHARACTER_EQUAL;
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
fn constant_character_not_equal() {
    let source = cases::CONSTANT_CHARACTER_NOT_EQUAL;
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
fn constant_float_greater_than() {
    let source = cases::CONSTANT_FLOAT_GREATER_THAN;
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
fn constant_float_less_than() {
    let source = cases::CONSTANT_FLOAT_LESS_THAN;
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
fn constant_float_greater_than_or_equal() {
    let source = cases::CONSTANT_FLOAT_GREATER_THAN_OR_EQUAL;
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
fn constant_float_less_than_or_equal() {
    let source = cases::CONSTANT_FLOAT_LESS_THAN_OR_EQUAL;
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
fn constant_float_equal() {
    let source = cases::CONSTANT_FLOAT_EQUAL;
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
fn constant_float_not_equal() {
    let source = cases::CONSTANT_FLOAT_NOT_EQUAL;
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
fn constant_integer_greater_than() {
    let source = cases::CONSTANT_INTEGER_GREATER_THAN;
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
fn constant_integer_less_than() {
    let source = cases::CONSTANT_INTEGER_LESS_THAN;
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
fn constant_integer_greater_than_or_equal() {
    let source = cases::CONSTANT_INTEGER_GREATER_THAN_OR_EQUAL;
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
fn constant_integer_less_than_or_equal() {
    let source = cases::CONSTANT_INTEGER_LESS_THAN_OR_EQUAL;
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
fn constant_integer_equal() {
    let source = cases::CONSTANT_INTEGER_EQUAL;
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
fn constant_integer_not_equal() {
    let source = cases::CONSTANT_INTEGER_NOT_EQUAL;
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
fn constant_string_greater_than() {
    let source = cases::CONSTANT_STRING_GREATER_THAN;
    let chunk = compile(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![Instruction::r#return(
                true,
                Address::encoded(false as u16),
                OperandType::BOOLEAN
            )],
            ..Default::default()
        }
    );
}

#[test]
fn constant_string_less_than() {
    let source = cases::CONSTANT_STRING_LESS_THAN;
    let chunk = compile(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            name: Some("main".to_string()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![Instruction::r#return(
                true,
                Address::encoded(false as u16),
                OperandType::BOOLEAN
            )],
            ..Default::default()
        }
    );
}

#[test]
fn constant_string_greater_than_or_equal() {
    let source = cases::CONSTANT_STRING_GREATER_THAN_OR_EQUAL;
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
fn constant_string_less_than_or_equal() {
    let source = cases::CONSTANT_STRING_LESS_THAN_OR_EQUAL;
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
fn constant_string_equal() {
    let source = cases::CONSTANT_STRING_EQUAL;
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
fn constant_string_not_equal() {
    let source = cases::CONSTANT_STRING_NOT_EQUAL;
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

#[test]
fn local_evalutation() {
    let source = cases::LOCAL_EVALUATION;
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
            ),],
            constants,
            ..Default::default()
        }
    );
}

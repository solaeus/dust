use crate::{
    Address, Chunk, FunctionType, Instruction, OperandType, Type, compile_main, tests::local_cases,
};
use std::sync::Arc;

#[test]
fn local_boolean_greater_than_or_equal() {
    let source = local_cases::LOCAL_BOOLEAN_GREATER_THAN_OR_EQUAL;
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            name: Arc::new("main".to_string()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::encoded(true as u16),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::load(
                    Address::register(1),
                    Address::encoded(true as u16),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::less(
                    false,
                    Address::register(0),
                    Address::register(1),
                    OperandType::BOOLEAN
                ),
                Instruction::jump(1, true),
                Instruction::load(
                    Address::register(2),
                    Address::encoded(true as u16),
                    OperandType::BOOLEAN,
                    true
                ),
                Instruction::load(
                    Address::register(2),
                    Address::encoded(false as u16),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::r#return(true, Address::register(2), OperandType::BOOLEAN)
            ],
            register_count: 3,
            ..Default::default()
        }
    );
}

#[test]
fn local_byte_greater_than_or_equal() {
    let source = local_cases::LOCAL_BYTE_GREATER_THAN_OR_EQUAL;
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            name: Arc::new("main".to_string()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::encoded(0x2A),
                    OperandType::BYTE,
                    false
                ),
                Instruction::load(
                    Address::register(1),
                    Address::encoded(0x2A),
                    OperandType::BYTE,
                    false
                ),
                Instruction::less(
                    false,
                    Address::register(0),
                    Address::register(1),
                    OperandType::BYTE
                ),
                Instruction::jump(1, true),
                Instruction::load(
                    Address::register(2),
                    Address::encoded(true as u16),
                    OperandType::BOOLEAN,
                    true
                ),
                Instruction::load(
                    Address::register(2),
                    Address::encoded(false as u16),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::r#return(true, Address::register(2), OperandType::BOOLEAN)
            ],
            register_count: 3,
            ..Default::default()
        }
    );
}

#[test]
fn local_character_greater_than_or_equal() {
    let source = local_cases::LOCAL_CHARACTER_GREATER_THAN_OR_EQUAL;
    let chunk = compile_main(source).unwrap();
    let mut constants = chunk.constants.clone();

    constants.add_character('z');
    constants.add_character('z');

    assert_eq!(
        chunk,
        Chunk {
            name: Arc::new("main".to_string()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::constant(0),
                    OperandType::CHARACTER,
                    false
                ),
                Instruction::load(
                    Address::register(1),
                    Address::constant(0),
                    OperandType::CHARACTER,
                    false
                ),
                Instruction::less(
                    false,
                    Address::register(0),
                    Address::register(1),
                    OperandType::CHARACTER
                ),
                Instruction::jump(1, true),
                Instruction::load(
                    Address::register(2),
                    Address::encoded(true as u16),
                    OperandType::BOOLEAN,
                    true
                ),
                Instruction::load(
                    Address::register(2),
                    Address::encoded(false as u16),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::r#return(true, Address::register(2), OperandType::BOOLEAN)
            ],
            constants,
            register_count: 3,
            ..Default::default()
        }
    );
}

#[test]
fn local_float_greater_than_or_equal() {
    let source = local_cases::LOCAL_FLOAT_GREATER_THAN_OR_EQUAL;
    let chunk = compile_main(source).unwrap();
    let mut constants = chunk.constants.clone();

    constants.add_float(42.0);
    constants.add_float(42.0);

    assert_eq!(
        chunk,
        Chunk {
            name: Arc::new("main".to_string()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::constant(0),
                    OperandType::FLOAT,
                    false
                ),
                Instruction::load(
                    Address::register(1),
                    Address::constant(0),
                    OperandType::FLOAT,
                    false
                ),
                Instruction::less(
                    false,
                    Address::register(0),
                    Address::register(1),
                    OperandType::FLOAT
                ),
                Instruction::jump(1, true),
                Instruction::load(
                    Address::register(2),
                    Address::encoded(true as u16),
                    OperandType::BOOLEAN,
                    true
                ),
                Instruction::load(
                    Address::register(2),
                    Address::encoded(false as u16),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::r#return(true, Address::register(2), OperandType::BOOLEAN)
            ],
            constants,
            register_count: 3,
            ..Default::default()
        }
    );
}

#[test]
fn local_integer_greater_than_or_equal() {
    let source = local_cases::LOCAL_INTEGER_GREATER_THAN_OR_EQUAL;
    let chunk = compile_main(source).unwrap();
    let mut constants = chunk.constants.clone();

    constants.add_integer(42);
    constants.add_integer(42);

    assert_eq!(
        chunk,
        Chunk {
            name: Arc::new("main".to_string()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::constant(0),
                    OperandType::INTEGER,
                    false
                ),
                Instruction::load(
                    Address::register(1),
                    Address::constant(0),
                    OperandType::INTEGER,
                    false
                ),
                Instruction::less(
                    false,
                    Address::register(0),
                    Address::register(1),
                    OperandType::INTEGER
                ),
                Instruction::jump(1, true),
                Instruction::load(
                    Address::register(2),
                    Address::encoded(true as u16),
                    OperandType::BOOLEAN,
                    true
                ),
                Instruction::load(
                    Address::register(2),
                    Address::encoded(false as u16),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::r#return(true, Address::register(2), OperandType::BOOLEAN)
            ],
            constants,
            register_count: 3,
            ..Default::default()
        }
    );
}

#[test]
fn local_string_greater_than_or_equal() {
    let source = local_cases::LOCAL_STRING_GREATER_THAN_OR_EQUAL;
    let chunk = compile_main(source).unwrap();
    let mut constants = chunk.constants.clone();

    constants.add_string("foo");
    constants.add_string("foo");

    assert_eq!(
        chunk,
        Chunk {
            name: Arc::new("main".to_string()),
            r#type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::constant(0),
                    OperandType::STRING,
                    false
                ),
                Instruction::load(
                    Address::register(1),
                    Address::constant(0),
                    OperandType::STRING,
                    false
                ),
                Instruction::less(
                    false,
                    Address::register(0),
                    Address::register(1),
                    OperandType::STRING
                ),
                Instruction::jump(1, true),
                Instruction::load(
                    Address::register(2),
                    Address::encoded(true as u16),
                    OperandType::BOOLEAN,
                    true
                ),
                Instruction::load(
                    Address::register(2),
                    Address::encoded(false as u16),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::r#return(true, Address::register(2), OperandType::BOOLEAN)
            ],
            constants,
            register_count: 3,
            ..Default::default()
        }
    );
}

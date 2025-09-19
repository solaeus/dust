use crate::{
    Address, Chunk, Instruction, OperandType, compile_main,
    resolver::{DeclarationId, TypeId},
    tests::local_cases,
};

#[test]
fn local_boolean_greater_than() {
    let source = local_cases::LOCAL_BOOLEAN_GREATER_THAN.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            declaration_id: DeclarationId::MAIN,
            type_id: TypeId(2),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::encoded(true as u16),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::load(
                    Address::register(1),
                    Address::encoded(false as u16),
                    OperandType::BOOLEAN,
                    false
                ),
                Instruction::less_equal(
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
fn local_byte_greater_than() {
    let source = local_cases::LOCAL_BYTE_GREATER_THAN.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            declaration_id: DeclarationId::MAIN,
            type_id: TypeId(2),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::encoded(0x2B),
                    OperandType::BYTE,
                    false
                ),
                Instruction::load(
                    Address::register(1),
                    Address::encoded(0x2A),
                    OperandType::BYTE,
                    false
                ),
                Instruction::less_equal(
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
fn local_character_greater_than() {
    let source = local_cases::LOCAL_CHARACTER_GREATER_THAN.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            declaration_id: DeclarationId::MAIN,
            type_id: TypeId(2),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::constant(0),
                    OperandType::CHARACTER,
                    false
                ),
                Instruction::load(
                    Address::register(1),
                    Address::constant(1),
                    OperandType::CHARACTER,
                    false
                ),
                Instruction::less_equal(
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
            register_count: 3,
            ..Default::default()
        }
    );
}

#[test]
fn local_float_greater_than() {
    let source = local_cases::LOCAL_FLOAT_GREATER_THAN.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            declaration_id: DeclarationId::MAIN,
            type_id: TypeId(2),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::constant(0),
                    OperandType::FLOAT,
                    false
                ),
                Instruction::load(
                    Address::register(1),
                    Address::constant(1),
                    OperandType::FLOAT,
                    false
                ),
                Instruction::less_equal(
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
            register_count: 3,
            ..Default::default()
        }
    );
}

#[test]
fn local_integer_greater_than() {
    let source = local_cases::LOCAL_INTEGER_GREATER_THAN.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            declaration_id: DeclarationId::MAIN,
            type_id: TypeId(2),
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
                Instruction::less_equal(
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
            register_count: 3,
            ..Default::default()
        }
    );
}

#[test]
fn local_string_greater_than() {
    let source = local_cases::LOCAL_STRING_GREATER_THAN.to_string();
    let chunk = compile_main(source).unwrap();

    assert_eq!(
        chunk,
        Chunk {
            declaration_id: DeclarationId::MAIN,
            type_id: TypeId(2),
            instructions: vec![
                Instruction::load(
                    Address::register(0),
                    Address::constant(0),
                    OperandType::STRING,
                    false
                ),
                Instruction::load(
                    Address::register(1),
                    Address::constant(1),
                    OperandType::STRING,
                    false
                ),
                Instruction::less_equal(
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
            register_count: 3,
            ..Default::default()
        }
    );
}

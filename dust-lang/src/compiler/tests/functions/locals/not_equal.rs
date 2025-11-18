use crate::{
    prototype::Prototype,
    compiler::compile,
    instruction::{Address, Instruction, OperandType},
    tests::{create_function_case, local_cases},
    source::{Position, SourceFileId, Span},
    r#type::{FunctionType, Type},
};

#[test]
fn local_boolean_not_equal() {
    let source = create_function_case(local_cases::LOCAL_BOOLEAN_NOT_EQUAL);
    let prototypes = compile(source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            name_position: Some(Position {
                file_id: SourceFileId(0),
                span: Span(16, 22)
            }),
            function_type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::r#move(0, Address::encoded(true as u16), OperandType::BOOLEAN),
                Instruction::r#move(1, Address::encoded(false as u16), OperandType::BOOLEAN),
                Instruction::equal(
                    false,
                    Address::register(0),
                    Address::register(1),
                    OperandType::BOOLEAN
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    OperandType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), OperandType::BOOLEAN),
                Instruction::r#return(true, Address::register(2), OperandType::BOOLEAN),
            ],
            register_count: 3,
            ..Default::default()
        }
    );
}

#[test]
fn local_byte_not_equal() {
    let source = create_function_case(local_cases::LOCAL_BYTE_NOT_EQUAL);
    let prototypes = compile(source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            name_position: Some(Position {
                file_id: SourceFileId(0),
                span: Span(16, 22)
            }),
            function_type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::r#move(0, Address::encoded(0x2A), OperandType::BYTE),
                Instruction::r#move(1, Address::encoded(0x2B), OperandType::BYTE),
                Instruction::equal(
                    false,
                    Address::register(0),
                    Address::register(1),
                    OperandType::BYTE
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    OperandType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), OperandType::BOOLEAN),
                Instruction::r#return(true, Address::register(2), OperandType::BOOLEAN),
            ],
            register_count: 3,
            ..Default::default()
        }
    );
}

#[test]
fn local_character_not_equal() {
    let source = create_function_case(local_cases::LOCAL_CHARACTER_NOT_EQUAL);
    let prototypes = compile(source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            name_position: Some(Position {
                file_id: SourceFileId(0),
                span: Span(16, 22)
            }),
            function_type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::CHARACTER),
                Instruction::r#move(1, Address::constant(1), OperandType::CHARACTER),
                Instruction::equal(
                    false,
                    Address::register(0),
                    Address::register(1),
                    OperandType::CHARACTER
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    OperandType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), OperandType::BOOLEAN),
                Instruction::r#return(true, Address::register(2), OperandType::BOOLEAN),
            ],
            register_count: 3,
            ..Default::default()
        }
    );
}

#[test]
fn local_float_not_equal() {
    let source = create_function_case(local_cases::LOCAL_FLOAT_NOT_EQUAL);
    let prototypes = compile(source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            name_position: Some(Position {
                file_id: SourceFileId(0),
                span: Span(16, 22)
            }),
            function_type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::FLOAT),
                Instruction::r#move(1, Address::constant(1), OperandType::FLOAT),
                Instruction::equal(
                    false,
                    Address::register(0),
                    Address::register(1),
                    OperandType::FLOAT
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    OperandType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), OperandType::BOOLEAN),
                Instruction::r#return(true, Address::register(2), OperandType::BOOLEAN),
            ],
            register_count: 3,
            ..Default::default()
        }
    );
}

#[test]
fn local_integer_not_equal() {
    let source = create_function_case(local_cases::LOCAL_INTEGER_NOT_EQUAL);
    let prototypes = compile(source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            name_position: Some(Position {
                file_id: SourceFileId(0),
                span: Span(16, 22)
            }),
            function_type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER),
                Instruction::r#move(1, Address::constant(1), OperandType::INTEGER),
                Instruction::equal(
                    false,
                    Address::register(0),
                    Address::register(1),
                    OperandType::INTEGER
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    OperandType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), OperandType::BOOLEAN),
                Instruction::r#return(true, Address::register(2), OperandType::BOOLEAN),
            ],
            register_count: 3,
            ..Default::default()
        }
    );
}

#[test]
fn local_string_not_equal() {
    let source = create_function_case(local_cases::LOCAL_STRING_NOT_EQUAL);
    let prototypes = compile(source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            name_position: Some(Position {
                file_id: SourceFileId(0),
                span: Span(16, 22)
            }),
            function_type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::STRING),
                Instruction::r#move(1, Address::constant(1), OperandType::STRING),
                Instruction::equal(
                    false,
                    Address::register(0),
                    Address::register(1),
                    OperandType::STRING
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    OperandType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), OperandType::BOOLEAN),
                Instruction::r#return(true, Address::register(2), OperandType::BOOLEAN),
            ],
            register_count: 3,
            ..Default::default()
        }
    );
}

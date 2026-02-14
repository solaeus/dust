use crate::{
    compiler::compile,
    dust_type::DustType,
    instruction::{Address, Instruction, ByteType},
    prototype::Prototype,
    tests::{create_function_case, local_cases},
};

#[test]
fn local_boolean_greater_than() {
    let source = create_function_case(
        local_cases::LOCAL_BOOLEAN_GREATER_THAN,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::encoded(true as u16), ByteType::BOOLEAN),
                Instruction::r#move(1, Address::encoded(false as u16), ByteType::BOOLEAN),
                Instruction::less_equal(
                    false,
                    Address::register(0),
                    Address::register(1),
                    ByteType::BOOLEAN
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    ByteType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), ByteType::BOOLEAN),
                Instruction::r#return(Address::register(2), ByteType::BOOLEAN)
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_byte_greater_than() {
    let source = create_function_case(local_cases::LOCAL_BYTE_GREATER_THAN, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::encoded(0x2B), ByteType::BYTE),
                Instruction::r#move(1, Address::encoded(0x2A), ByteType::BYTE),
                Instruction::less_equal(
                    false,
                    Address::register(0),
                    Address::register(1),
                    ByteType::BYTE
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    ByteType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), ByteType::BOOLEAN),
                Instruction::r#return(Address::register(2), ByteType::BOOLEAN)
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_character_greater_than() {
    let source = create_function_case(
        local_cases::LOCAL_CHARACTER_GREATER_THAN,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), ByteType::CHARACTER),
                Instruction::r#move(1, Address::constant(1), ByteType::CHARACTER),
                Instruction::less_equal(
                    false,
                    Address::register(0),
                    Address::register(1),
                    ByteType::CHARACTER
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    ByteType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), ByteType::BOOLEAN),
                Instruction::r#return(Address::register(2), ByteType::BOOLEAN)
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_float_greater_than() {
    let source = create_function_case(local_cases::LOCAL_FLOAT_GREATER_THAN, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), ByteType::FLOAT),
                Instruction::r#move(1, Address::constant(1), ByteType::FLOAT),
                Instruction::less_equal(
                    false,
                    Address::register(0),
                    Address::register(1),
                    ByteType::FLOAT
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    ByteType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), ByteType::BOOLEAN),
                Instruction::r#return(Address::register(2), ByteType::BOOLEAN)
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_integer_greater_than() {
    let source = create_function_case(
        local_cases::LOCAL_INTEGER_GREATER_THAN,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), ByteType::INTEGER),
                Instruction::r#move(1, Address::constant(1), ByteType::INTEGER),
                Instruction::less_equal(
                    false,
                    Address::register(0),
                    Address::register(1),
                    ByteType::INTEGER
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    ByteType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), ByteType::BOOLEAN),
                Instruction::r#return(Address::register(2), ByteType::BOOLEAN)
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_string_greater_than() {
    let source = create_function_case(local_cases::LOCAL_STRING_GREATER_THAN, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), ByteType::STRING),
                Instruction::r#move(1, Address::constant(1), ByteType::STRING),
                Instruction::less_equal(
                    false,
                    Address::register(0),
                    Address::register(1),
                    ByteType::STRING
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    ByteType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), ByteType::BOOLEAN),
                Instruction::r#return(Address::register(2), ByteType::BOOLEAN)
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

use crate::{
    compiler::compile,
    dust_type::DustType,
    instruction::{Address, Instruction, ByteType},
    prototype::Prototype,
    tests::{create_function_case, local_cases},
};

#[test]
fn local_boolean_not_equal() {
    let source = create_function_case(local_cases::LOCAL_BOOLEAN_NOT_EQUAL, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::encoded_boolean(true)),
                Instruction::r#move(1, Address::encoded_boolean(false)),
                Instruction::equal(false, Address::register(0), Address::register(1)),
                Instruction::move_with_jump(2, Address::encoded_boolean(false), 1, true),
                Instruction::r#move(2, Address::encoded_boolean(true)),
                Instruction::r#return(Address::register(2)),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_byte_not_equal() {
    let source = create_function_case(local_cases::LOCAL_BYTE_NOT_EQUAL, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::encoded_byte(0x2A)),
                Instruction::r#move(1, Address::encoded_byte(0x2B)),
                Instruction::equal(false, Address::register(0), Address::register(1)),
                Instruction::move_with_jump(2, Address::encoded_boolean(false), 1, true),
                Instruction::r#move(2, Address::encoded_boolean(true)),
                Instruction::r#return(Address::register(2)),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_character_not_equal() {
    let source = create_function_case(local_cases::LOCAL_CHARACTER_NOT_EQUAL, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#move(1, Address::constant(1)),
                Instruction::equal(false, Address::register(0), Address::register(1)),
                Instruction::move_with_jump(2, Address::encoded_boolean(false), 1, true),
                Instruction::r#move(2, Address::encoded_boolean(true)),
                Instruction::r#return(Address::register(2)),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_float_not_equal() {
    let source = create_function_case(local_cases::LOCAL_FLOAT_NOT_EQUAL, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#move(1, Address::constant(1)),
                Instruction::equal(false, Address::register(0), Address::register(1)),
                Instruction::move_with_jump(2, Address::encoded_boolean(false), 1, true),
                Instruction::r#move(2, Address::encoded_boolean(true)),
                Instruction::r#return(Address::register(2)),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_integer_not_equal() {
    let source = create_function_case(local_cases::LOCAL_INTEGER_NOT_EQUAL, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#move(1, Address::constant(1)),
                Instruction::equal(false, Address::register(0), Address::register(1)),
                Instruction::move_with_jump(2, Address::encoded_boolean(false), 1, true),
                Instruction::r#move(2, Address::encoded_boolean(true)),
                Instruction::r#return(Address::register(2)),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_string_not_equal() {
    let source = create_function_case(local_cases::LOCAL_STRING_NOT_EQUAL, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#move(1, Address::constant(1)),
                Instruction::equal(false, Address::register(0), Address::register(1)),
                Instruction::move_with_jump(2, Address::encoded_boolean(false), 1, true),
                Instruction::r#move(2, Address::encoded_boolean(true)),
                Instruction::r#return(Address::register(2)),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

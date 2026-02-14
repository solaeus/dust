use crate::{
    compiler::compile,
    dust_type::DustType,
    instruction::{Address, Instruction, ByteType},
    prototype::Prototype,
    tests::{create_function_case, local_cases},
};

#[test]
fn local_boolean_equal() {
    let source = create_function_case(local_cases::LOCAL_BOOLEAN_EQUAL, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::encoded(true as u16)),
                Instruction::r#move(1, Address::encoded(true as u16)),
                Instruction::equal(true, Address::register(0), Address::register(1)),
                Instruction::move_with_jump(2, Address::encoded(false as u16), 1, true),
                Instruction::r#move(2, Address::encoded(true as u16)),
                Instruction::r#return(Address::register(2)),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_byte_equal() {
    let source = create_function_case(local_cases::LOCAL_BYTE_EQUAL, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::encoded(0x2A)),
                Instruction::r#move(1, Address::encoded(0x2A)),
                Instruction::equal(true, Address::register(0), Address::register(1)),
                Instruction::move_with_jump(2, Address::encoded(false as u16), 1, true),
                Instruction::r#move(2, Address::encoded(true as u16)),
                Instruction::r#return(Address::register(2)),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_character_equal() {
    let source = create_function_case(local_cases::LOCAL_CHARACTER_EQUAL, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#move(1, Address::constant(0)),
                Instruction::equal(true, Address::register(0), Address::register(1)),
                Instruction::move_with_jump(2, Address::encoded(false as u16), 1, true),
                Instruction::r#move(2, Address::encoded(true as u16)),
                Instruction::r#return(Address::register(2)),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_float_equal() {
    let source = create_function_case(local_cases::LOCAL_FLOAT_EQUAL, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#move(1, Address::constant(0)),
                Instruction::equal(true, Address::register(0), Address::register(1)),
                Instruction::move_with_jump(2, Address::encoded(false as u16), 1, true),
                Instruction::r#move(2, Address::encoded(true as u16)),
                Instruction::r#return(Address::register(2)),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_integer_equal() {
    let source = create_function_case(local_cases::LOCAL_INTEGER_EQUAL, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#move(1, Address::constant(0)),
                Instruction::equal(true, Address::register(0), Address::register(1)),
                Instruction::move_with_jump(2, Address::encoded(false as u16), 1, true),
                Instruction::r#move(2, Address::encoded(true as u16)),
                Instruction::r#return(Address::register(2)),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_string_equal() {
    let source = create_function_case(local_cases::LOCAL_STRING_EQUAL, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#move(1, Address::constant(0)),
                Instruction::equal(true, Address::register(0), Address::register(1)),
                Instruction::move_with_jump(2, Address::encoded(false as u16), 1, true),
                Instruction::r#move(2, Address::encoded(true as u16)),
                Instruction::r#return(Address::register(2)),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

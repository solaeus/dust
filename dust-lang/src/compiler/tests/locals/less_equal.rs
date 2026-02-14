use crate::{
    compiler::compile_main,
    dust_type::DustType,
    instruction::{Address, Instruction, ByteType},
    prototype::Prototype,
    tests::local_cases,
};

#[test]
fn local_boolean_less_than_or_equal() {
    let source = local_cases::LOCAL_BOOLEAN_LESS_THAN_OR_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::encoded(true as u16), ByteType::BOOLEAN),
                Instruction::r#move(1, Address::encoded(true as u16), ByteType::BOOLEAN),
                Instruction::less_equal(
                    true,
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
fn local_byte_less_than_or_equal() {
    let source = local_cases::LOCAL_BYTE_LESS_THAN_OR_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::encoded(0x2A), ByteType::BYTE),
                Instruction::r#move(1, Address::encoded(0x2A), ByteType::BYTE),
                Instruction::less_equal(
                    true,
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
fn local_character_less_than_or_equal() {
    let source = local_cases::LOCAL_CHARACTER_LESS_THAN_OR_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), ByteType::CHARACTER),
                Instruction::r#move(1, Address::constant(0), ByteType::CHARACTER),
                Instruction::less_equal(
                    true,
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
fn local_float_less_than_or_equal() {
    let source = local_cases::LOCAL_FLOAT_LESS_THAN_OR_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), ByteType::FLOAT),
                Instruction::r#move(1, Address::constant(0), ByteType::FLOAT),
                Instruction::less_equal(
                    true,
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
fn local_integer_less_than_or_equal() {
    let source = local_cases::LOCAL_INTEGER_LESS_THAN_OR_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), ByteType::INTEGER),
                Instruction::r#move(1, Address::constant(0), ByteType::INTEGER),
                Instruction::less_equal(
                    true,
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
fn local_string_less_than_or_equal() {
    let source = local_cases::LOCAL_STRING_LESS_THAN_OR_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), ByteType::STRING),
                Instruction::r#move(1, Address::constant(0), ByteType::STRING),
                Instruction::less_equal(
                    true,
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

use crate::{
    compiler::compile_main,
    dust_type::DustType,
    instruction::{Address, Instruction, ByteType},
    prototype::Prototype,
    tests::local_cases,
};

#[test]
fn local_byte_addition() {
    let source = local_cases::LOCAL_BYTE_ADDITION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Byte,
            instructions: vec![
                Instruction::r#move(0, Address::encoded(40), ByteType::BYTE),
                Instruction::r#move(1, Address::encoded(2), ByteType::BYTE),
                Instruction::add(
                    2,
                    Address::register(0),
                    Address::register(1),
                    ByteType::BYTE
                ),
                Instruction::r#return(Address::register(2), ByteType::BYTE)
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_float_addition() {
    let source = local_cases::LOCAL_FLOAT_ADDITION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Float,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), ByteType::FLOAT),
                Instruction::r#move(1, Address::constant(1), ByteType::FLOAT),
                Instruction::add(
                    2,
                    Address::register(0),
                    Address::register(1),
                    ByteType::FLOAT
                ),
                Instruction::r#return(Address::register(2), ByteType::FLOAT)
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_integer_addition() {
    let source = local_cases::LOCAL_INTEGER_ADDITION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), ByteType::INTEGER),
                Instruction::r#move(1, Address::constant(1), ByteType::INTEGER),
                Instruction::add(
                    2,
                    Address::register(0),
                    Address::register(1),
                    ByteType::INTEGER
                ),
                Instruction::r#return(Address::register(2), ByteType::INTEGER)
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_string_concatenation() {
    let source = local_cases::LOCAL_STRING_CONCATENATION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::String,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), ByteType::STRING),
                Instruction::r#move(1, Address::constant(1), ByteType::STRING),
                Instruction::add(
                    2,
                    Address::register(0),
                    Address::register(1),
                    ByteType::STRING
                ),
                Instruction::r#return(Address::register(2), ByteType::STRING)
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_character_concatenation() {
    let source = local_cases::LOCAL_CHARACTER_CONCATENATION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::String,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), ByteType::CHARACTER),
                Instruction::r#move(1, Address::constant(0), ByteType::CHARACTER),
                Instruction::add(
                    2,
                    Address::register(0),
                    Address::register(1),
                    ByteType::CHARACTER
                ),
                Instruction::r#return(Address::register(2), ByteType::STRING)
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_string_character_concatenation() {
    let source = local_cases::LOCAL_STRING_CHARACTER_CONCATENATION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::String,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), ByteType::STRING),
                Instruction::r#move(1, Address::constant(1), ByteType::CHARACTER),
                Instruction::add(
                    2,
                    Address::register(0),
                    Address::register(1),
                    ByteType::STRING_CHARACTER
                ),
                Instruction::r#return(Address::register(2), ByteType::STRING)
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_character_string_concatenation() {
    let source = local_cases::LOCAL_CHARACTER_STRING_CONCATENATION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::String,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), ByteType::CHARACTER),
                Instruction::r#move(1, Address::constant(1), ByteType::STRING),
                Instruction::add(
                    2,
                    Address::register(0),
                    Address::register(1),
                    ByteType::CHARACTER_STRING
                ),
                Instruction::r#return(Address::register(2), ByteType::STRING)
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_mut_byte_addition() {
    let source = local_cases::LOCAL_MUT_BYTE_ADDITION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Byte,
            instructions: vec![
                Instruction::r#move(0, Address::encoded(40), ByteType::BYTE),
                Instruction::add(
                    0,
                    Address::register(0),
                    Address::encoded(2),
                    ByteType::BYTE
                ),
                Instruction::r#return(Address::register(0), ByteType::BYTE)
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_mut_float_addition() {
    let source = local_cases::LOCAL_MUT_FLOAT_ADDITION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Float,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), ByteType::FLOAT),
                Instruction::add(
                    0,
                    Address::register(0),
                    Address::constant(1),
                    ByteType::FLOAT
                ),
                Instruction::r#return(Address::register(0), ByteType::FLOAT)
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_mut_integer_addition() {
    let source = local_cases::LOCAL_MUT_INTEGER_ADDITION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), ByteType::INTEGER),
                Instruction::add(
                    0,
                    Address::register(0),
                    Address::constant(1),
                    ByteType::INTEGER
                ),
                Instruction::r#return(Address::register(0), ByteType::INTEGER)
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_mut_string_concatenation() {
    let source = local_cases::LOCAL_MUT_STRING_CONCATENATION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::String,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), ByteType::STRING),
                Instruction::add(
                    0,
                    Address::register(0),
                    Address::constant(1),
                    ByteType::STRING
                ),
                Instruction::r#return(Address::register(0), ByteType::STRING)
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_mut_string_character_concatenation() {
    let source = local_cases::LOCAL_MUT_STRING_CHARACTER_CONCATENATION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::String,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), ByteType::STRING),
                Instruction::add(
                    0,
                    Address::register(0),
                    Address::constant(1),
                    ByteType::STRING_CHARACTER
                ),
                Instruction::r#return(Address::register(0), ByteType::STRING)
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

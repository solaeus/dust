use crate::{
    compiler::compile_main,
    dust_type::DustType,
    instruction::{Address, Instruction},
    prototype::Prototype,
    tests::local_cases,
};

#[test]
fn local_boolean_equal() {
    let source = local_cases::LOCAL_BOOLEAN_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::encoded_boolean(true)),
                Instruction::r#move(1, Address::encoded_boolean(true)),
                Instruction::equal(true, Address::register(0), Address::register(1)),
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
fn local_byte_equal() {
    let source = local_cases::LOCAL_BYTE_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::encoded_byte(0x2A)),
                Instruction::r#move(1, Address::encoded_byte(0x2A)),
                Instruction::equal(true, Address::register(0), Address::register(1)),
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
fn local_character_equal() {
    let source = local_cases::LOCAL_CHARACTER_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#move(1, Address::constant(0)),
                Instruction::equal(true, Address::register(0), Address::register(1)),
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
fn local_float_equal() {
    let source = local_cases::LOCAL_FLOAT_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#move(1, Address::constant(0)),
                Instruction::equal(true, Address::register(0), Address::register(1)),
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
fn local_integer_equal() {
    let source = local_cases::LOCAL_INTEGER_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#move(1, Address::constant(0)),
                Instruction::equal(true, Address::register(0), Address::register(1)),
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
fn local_string_equal() {
    let source = local_cases::LOCAL_STRING_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#move(1, Address::constant(0)),
                Instruction::equal(true, Address::register(0), Address::register(1)),
                Instruction::move_with_jump(2, Address::encoded_boolean(false), 1, true),
                Instruction::r#move(2, Address::encoded_boolean(true)),
                Instruction::r#return(Address::register(2)),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

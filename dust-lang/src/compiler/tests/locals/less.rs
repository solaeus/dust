use crate::{
    compiler::compile_main,
    dust_type::DustType,
    instruction::{Address, Instruction},
    prototype::Prototype,
    tests::local_cases,
};

#[test]
fn local_boolean_less_than() {
    let source = local_cases::LOCAL_BOOLEAN_LESS_THAN;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::encoded(false as u16)),
                Instruction::r#move(1, Address::encoded(true as u16)),
                Instruction::less(true, Address::register(0), Address::register(1)),
                Instruction::move_with_jump(2, Address::encoded(false as u16), 1, true),
                Instruction::r#move(2, Address::encoded(true as u16)),
                Instruction::r#return(Address::register(2))
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_byte_less_than() {
    let source = local_cases::LOCAL_BYTE_LESS_THAN;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::encoded(0x29)),
                Instruction::r#move(1, Address::encoded(0x2A)),
                Instruction::less(true, Address::register(0), Address::register(1)),
                Instruction::move_with_jump(2, Address::encoded(false as u16), 1, true),
                Instruction::r#move(2, Address::encoded(true as u16)),
                Instruction::r#return(Address::register(2))
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_character_less_than() {
    let source = local_cases::LOCAL_CHARACTER_LESS_THAN;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#move(1, Address::constant(1)),
                Instruction::less(true, Address::register(0), Address::register(1)),
                Instruction::move_with_jump(2, Address::encoded(false as u16), 1, true),
                Instruction::r#move(2, Address::encoded(true as u16)),
                Instruction::r#return(Address::register(2))
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_float_less_than() {
    let source = local_cases::LOCAL_FLOAT_LESS_THAN;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#move(1, Address::constant(1)),
                Instruction::less(true, Address::register(0), Address::register(1)),
                Instruction::move_with_jump(2, Address::encoded(false as u16), 1, true),
                Instruction::r#move(2, Address::encoded(true as u16)),
                Instruction::r#return(Address::register(2))
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_integer_less_than() {
    let source = local_cases::LOCAL_INTEGER_LESS_THAN;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#move(1, Address::constant(1)),
                Instruction::less(true, Address::register(0), Address::register(1)),
                Instruction::move_with_jump(2, Address::encoded(false as u16), 1, true),
                Instruction::r#move(2, Address::encoded(true as u16)),
                Instruction::r#return(Address::register(2))
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_string_less_than() {
    let source = local_cases::LOCAL_STRING_LESS_THAN;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#move(1, Address::constant(1)),
                Instruction::less(true, Address::register(0), Address::register(1)),
                Instruction::move_with_jump(2, Address::encoded(false as u16), 1, true),
                Instruction::r#move(2, Address::encoded(true as u16)),
                Instruction::r#return(Address::register(2))
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

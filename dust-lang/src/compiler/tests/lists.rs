use crate::{
    compiler::compile_main,
    dust_type::DustType,
    instruction::{Address, Instruction},
    prototype::Prototype,
    tests::list_cases,
};

#[test]
fn list_boolean() {
    let source = list_cases::LIST_BOOLEAN;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::list(DustType::Boolean),
            instructions: vec![
                Instruction::new_list(0, Address::constant(0)),
                Instruction::set_list(0, Address::encoded(true as u16), Address::constant(1)),
                Instruction::set_list(0, Address::encoded(false as u16), Address::constant(2)),
                Instruction::set_list(0, Address::encoded(true as u16), Address::constant(3)),
                Instruction::r#return(Address::register(0)),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn list_byte() {
    let source = list_cases::LIST_BYTE;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::list(DustType::Byte),
            instructions: vec![
                Instruction::new_list(0, Address::constant(0)),
                Instruction::set_list(0, Address::encoded(42), Address::constant(1)),
                Instruction::set_list(0, Address::encoded(43), Address::constant(2)),
                Instruction::set_list(0, Address::encoded(44), Address::constant(3)),
                Instruction::r#return(Address::register(0)),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn list_character() {
    let source = list_cases::LIST_CHARACTER;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::list(DustType::Character),
            instructions: vec![
                Instruction::new_list(0, Address::constant(0)),
                Instruction::set_list(0, Address::constant(1), Address::constant(2)),
                Instruction::set_list(0, Address::constant(3), Address::constant(4)),
                Instruction::set_list(0, Address::constant(5), Address::constant(6)),
                Instruction::r#return(Address::register(0)),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn list_float() {
    let source = list_cases::LIST_FLOAT;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::list(DustType::Float),
            instructions: vec![
                Instruction::new_list(0, Address::constant(0)),
                Instruction::set_list(0, Address::constant(1), Address::constant(2)),
                Instruction::set_list(0, Address::constant(3), Address::constant(4)),
                Instruction::set_list(0, Address::constant(5), Address::constant(6)),
                Instruction::r#return(Address::register(0)),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn list_integer() {
    let source = list_cases::LIST_INTEGER;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::list(DustType::Integer),
            instructions: vec![
                Instruction::new_list(0, Address::constant(0)),
                Instruction::set_list(0, Address::constant(1), Address::constant(2)),
                Instruction::set_list(0, Address::constant(3), Address::constant(1)),
                Instruction::set_list(0, Address::constant(0), Address::constant(3)),
                Instruction::r#return(Address::register(0)),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn list_equal() {
    let source = list_cases::LIST_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0)),
                Instruction::set_list(0, Address::encoded(true as u16), Address::constant(1)),
                Instruction::set_list(0, Address::encoded(false as u16), Address::constant(2)),
                Instruction::new_list(1, Address::constant(0)),
                Instruction::set_list(1, Address::encoded(true as u16), Address::constant(1)),
                Instruction::set_list(1, Address::encoded(false as u16), Address::constant(2)),
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
fn list_not_equal() {
    let source = list_cases::LIST_NOT_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0)),
                Instruction::set_list(0, Address::encoded(0x2A), Address::constant(1)),
                Instruction::set_list(0, Address::encoded(0x2B), Address::constant(2)),
                Instruction::new_list(1, Address::constant(0)),
                Instruction::set_list(1, Address::encoded(0x2B), Address::constant(1)),
                Instruction::set_list(1, Address::encoded(0x2A), Address::constant(2)),
                Instruction::equal(false, Address::register(0), Address::register(1)),
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
fn list_greater_than() {
    let source = list_cases::LIST_GREATER_THAN;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0)),
                Instruction::set_list(0, Address::constant(1), Address::constant(2)),
                Instruction::set_list(0, Address::constant(3), Address::constant(4)),
                Instruction::new_list(1, Address::constant(0)),
                Instruction::set_list(1, Address::constant(3), Address::constant(2)),
                Instruction::set_list(1, Address::constant(1), Address::constant(4)),
                Instruction::less_equal(false, Address::register(0), Address::register(1)),
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
fn list_less_than() {
    let source = list_cases::LIST_LESS_THAN;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0)),
                Instruction::set_list(0, Address::constant(1), Address::constant(2)),
                Instruction::set_list(0, Address::constant(3), Address::constant(4)),
                Instruction::new_list(1, Address::constant(0)),
                Instruction::set_list(1, Address::constant(3), Address::constant(2)),
                Instruction::set_list(1, Address::constant(1), Address::constant(4)),
                Instruction::less(true, Address::register(0), Address::register(1)),
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
fn list_greater_than_or_equal() {
    let source = list_cases::LIST_GREATER_THAN_OR_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0)),
                Instruction::set_list(0, Address::constant(1), Address::constant(2)),
                Instruction::set_list(0, Address::constant(0), Address::constant(1)),
                Instruction::new_list(1, Address::constant(0)),
                Instruction::set_list(1, Address::constant(1), Address::constant(2)),
                Instruction::set_list(1, Address::constant(0), Address::constant(1)),
                Instruction::less(false, Address::register(0), Address::register(1)),
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
fn list_less_than_or_equal() {
    let source = list_cases::LIST_LESS_THAN_OR_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0)),
                Instruction::set_list(0, Address::constant(1), Address::constant(2)),
                Instruction::set_list(0, Address::constant(3), Address::constant(4)),
                Instruction::new_list(1, Address::constant(0)),
                Instruction::set_list(1, Address::constant(1), Address::constant(2)),
                Instruction::set_list(1, Address::constant(3), Address::constant(4)),
                Instruction::less_equal(true, Address::register(0), Address::register(1)),
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
fn list_index_boolean() {
    let source = list_cases::LIST_INDEX_BOOLEAN;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0)),
                Instruction::set_list(0, Address::encoded(true as u16), Address::constant(1)),
                Instruction::set_list(0, Address::encoded(false as u16), Address::constant(2)),
                Instruction::set_list(0, Address::encoded(true as u16), Address::constant(3)),
                Instruction::get_list(1, Address::register(0), Address::constant(1)),
                Instruction::r#return(Address::register(1)),
            ],
            register_count: 2,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn list_index_byte() {
    let source = list_cases::LIST_INDEX_BYTE;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Byte,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0)),
                Instruction::set_list(0, Address::encoded(0x2A), Address::constant(1)),
                Instruction::set_list(0, Address::encoded(0x2B), Address::constant(2)),
                Instruction::set_list(0, Address::encoded(0x2C), Address::constant(3)),
                Instruction::get_list(1, Address::register(0), Address::constant(2)),
                Instruction::r#return(Address::register(1)),
            ],
            register_count: 2,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn list_index_character() {
    let source = list_cases::LIST_INDEX_CHARACTER;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Character,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0)),
                Instruction::set_list(0, Address::constant(1), Address::constant(2)),
                Instruction::set_list(0, Address::constant(3), Address::constant(4)),
                Instruction::set_list(0, Address::constant(5), Address::constant(6)),
                Instruction::get_list(1, Address::register(0), Address::constant(6)),
                Instruction::r#return(Address::register(1)),
            ],
            register_count: 2,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn list_index_float() {
    let source = list_cases::LIST_INDEX_FLOAT;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Float,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0)),
                Instruction::set_list(0, Address::constant(1), Address::constant(2)),
                Instruction::set_list(0, Address::constant(3), Address::constant(4)),
                Instruction::set_list(0, Address::constant(5), Address::constant(6)),
                Instruction::get_list(1, Address::register(0), Address::constant(4)),
                Instruction::r#return(Address::register(1)),
            ],
            register_count: 2,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn list_index_integer() {
    let source = list_cases::LIST_INDEX_INTEGER;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0)),
                Instruction::set_list(0, Address::constant(1), Address::constant(2)),
                Instruction::set_list(0, Address::constant(3), Address::constant(1)),
                Instruction::set_list(0, Address::constant(0), Address::constant(3)),
                Instruction::get_list(1, Address::register(0), Address::constant(2)),
                Instruction::r#return(Address::register(1)),
            ],
            register_count: 2,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn list_index_string() {
    let source = list_cases::LIST_INDEX_STRING;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::String,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0)),
                Instruction::set_list(0, Address::constant(1), Address::constant(2)),
                Instruction::set_list(0, Address::constant(3), Address::constant(4)),
                Instruction::set_list(0, Address::constant(5), Address::constant(6)),
                Instruction::get_list(1, Address::register(0), Address::constant(6)),
                Instruction::r#return(Address::register(1)),
            ],
            register_count: 2,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_list_boolean() {
    let source = list_cases::LOCAL_LIST_BOOLEAN;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::list(DustType::Boolean),
            instructions: vec![
                Instruction::new_list(0, Address::constant(0)),
                Instruction::set_list(0, Address::encoded(true as u16), Address::constant(1)),
                Instruction::set_list(0, Address::encoded(false as u16), Address::constant(2)),
                Instruction::set_list(0, Address::encoded(true as u16), Address::constant(3)),
                Instruction::r#return(Address::register(0)),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_list_equal() {
    let source = list_cases::LOCAL_LIST_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0)),
                Instruction::set_list(0, Address::encoded(true as u16), Address::constant(1)),
                Instruction::set_list(0, Address::encoded(false as u16), Address::constant(2)),
                Instruction::new_list(1, Address::constant(0)),
                Instruction::set_list(1, Address::encoded(true as u16), Address::constant(1)),
                Instruction::set_list(1, Address::encoded(false as u16), Address::constant(2)),
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
fn local_list_not_equal() {
    let source = list_cases::LOCAL_LIST_NOT_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0)),
                Instruction::set_list(0, Address::encoded(0x2A), Address::constant(1)),
                Instruction::set_list(0, Address::encoded(0x2B), Address::constant(2)),
                Instruction::new_list(1, Address::constant(0)),
                Instruction::set_list(1, Address::encoded(0x2B), Address::constant(1)),
                Instruction::set_list(1, Address::encoded(0x2A), Address::constant(2)),
                Instruction::equal(false, Address::register(0), Address::register(1)),
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
fn local_list_greater_than() {
    let source = list_cases::LOCAL_LIST_GREATER_THAN;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0)),
                Instruction::set_list(0, Address::constant(1), Address::constant(2)),
                Instruction::set_list(0, Address::constant(3), Address::constant(4)),
                Instruction::new_list(1, Address::constant(0)),
                Instruction::set_list(1, Address::constant(3), Address::constant(2)),
                Instruction::set_list(1, Address::constant(1), Address::constant(4)),
                Instruction::less_equal(false, Address::register(0), Address::register(1)),
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
fn local_list_less_than() {
    let source = list_cases::LOCAL_LIST_LESS_THAN;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0)),
                Instruction::set_list(0, Address::constant(1), Address::constant(2)),
                Instruction::set_list(0, Address::constant(3), Address::constant(4)),
                Instruction::new_list(1, Address::constant(0)),
                Instruction::set_list(1, Address::constant(3), Address::constant(2)),
                Instruction::set_list(1, Address::constant(1), Address::constant(4)),
                Instruction::less(true, Address::register(1), Address::register(0)),
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
fn local_list_greater_than_or_equal() {
    let source = list_cases::LOCAL_LIST_GREATER_THAN_OR_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0)),
                Instruction::set_list(0, Address::constant(1), Address::constant(2)),
                Instruction::set_list(0, Address::constant(0), Address::constant(1)),
                Instruction::new_list(1, Address::constant(0)),
                Instruction::set_list(1, Address::constant(1), Address::constant(2)),
                Instruction::set_list(1, Address::constant(0), Address::constant(1)),
                Instruction::less(false, Address::register(0), Address::register(1)),
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
fn local_list_less_than_or_equal() {
    let source = list_cases::LOCAL_LIST_LESS_THAN_OR_EQUAL;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0)),
                Instruction::set_list(0, Address::constant(1), Address::constant(2)),
                Instruction::set_list(0, Address::constant(3), Address::constant(4)),
                Instruction::new_list(1, Address::constant(0)),
                Instruction::set_list(1, Address::constant(1), Address::constant(2)),
                Instruction::set_list(1, Address::constant(3), Address::constant(4)),
                Instruction::less_equal(true, Address::register(0), Address::register(1)),
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
fn list_string() {
    let source = list_cases::LIST_STRING;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::list(DustType::String),
            instructions: vec![
                Instruction::new_list(0, Address::constant(0)),
                Instruction::set_list(0, Address::constant(1), Address::constant(2)),
                Instruction::set_list(0, Address::constant(3), Address::constant(4)),
                Instruction::set_list(0, Address::constant(5), Address::constant(6)),
                Instruction::r#return(Address::register(0)),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

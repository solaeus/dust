use crate::{
    compiler::compile,
    dust_type::DustType,
    instruction::{Address, Instruction, ByteType},
    prototype::Prototype,
    tests::{create_function_case, list_cases},
};

#[test]
fn list_boolean() {
    let source = create_function_case(list_cases::LIST_BOOLEAN, ByteType::LIST_BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::list(DustType::Boolean),
            instructions: vec![
                Instruction::new_list(0, Address::constant(0), ByteType::LIST_BOOLEAN),
                Instruction::set_list(
                    0,
                    Address::encoded(true as u16),
                    Address::constant(1),
                    ByteType::BOOLEAN
                ),
                Instruction::set_list(
                    0,
                    Address::encoded(false as u16),
                    Address::constant(2),
                    ByteType::BOOLEAN
                ),
                Instruction::set_list(
                    0,
                    Address::encoded(true as u16),
                    Address::constant(3),
                    ByteType::BOOLEAN
                ),
                Instruction::r#return(Address::register(0), ByteType::LIST_BOOLEAN),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn list_byte() {
    let source = create_function_case(list_cases::LIST_BYTE, ByteType::LIST_BYTE);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::list(DustType::Byte),
            instructions: vec![
                Instruction::new_list(0, Address::constant(0), ByteType::LIST_BYTE),
                Instruction::set_list(
                    0,
                    Address::encoded(42),
                    Address::constant(1),
                    ByteType::BYTE
                ),
                Instruction::set_list(
                    0,
                    Address::encoded(43),
                    Address::constant(2),
                    ByteType::BYTE
                ),
                Instruction::set_list(
                    0,
                    Address::encoded(44),
                    Address::constant(3),
                    ByteType::BYTE
                ),
                Instruction::r#return(Address::register(0), ByteType::LIST_BYTE),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn list_character() {
    let source = create_function_case(list_cases::LIST_CHARACTER, ByteType::LIST_CHARACTER);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::list(DustType::Character),
            instructions: vec![
                Instruction::new_list(0, Address::constant(0), ByteType::LIST_CHARACTER),
                Instruction::set_list(
                    0,
                    Address::constant(1),
                    Address::constant(2),
                    ByteType::CHARACTER
                ),
                Instruction::set_list(
                    0,
                    Address::constant(3),
                    Address::constant(4),
                    ByteType::CHARACTER
                ),
                Instruction::set_list(
                    0,
                    Address::constant(5),
                    Address::constant(6),
                    ByteType::CHARACTER
                ),
                Instruction::r#return(Address::register(0), ByteType::LIST_CHARACTER),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn list_float() {
    let source = create_function_case(list_cases::LIST_FLOAT, ByteType::LIST_FLOAT);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::list(DustType::Float),
            instructions: vec![
                Instruction::new_list(0, Address::constant(0), ByteType::LIST_FLOAT),
                Instruction::set_list(
                    0,
                    Address::constant(1),
                    Address::constant(2),
                    ByteType::FLOAT
                ),
                Instruction::set_list(
                    0,
                    Address::constant(3),
                    Address::constant(4),
                    ByteType::FLOAT
                ),
                Instruction::set_list(
                    0,
                    Address::constant(5),
                    Address::constant(6),
                    ByteType::FLOAT
                ),
                Instruction::r#return(Address::register(0), ByteType::LIST_FLOAT),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn list_integer() {
    let source = create_function_case(list_cases::LIST_INTEGER, ByteType::LIST_INTEGER);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::list(DustType::Integer),
            instructions: vec![
                Instruction::new_list(0, Address::constant(0), ByteType::LIST_INTEGER),
                Instruction::set_list(
                    0,
                    Address::constant(1),
                    Address::constant(2),
                    ByteType::INTEGER
                ),
                Instruction::set_list(
                    0,
                    Address::constant(3),
                    Address::constant(1),
                    ByteType::INTEGER
                ),
                Instruction::set_list(
                    0,
                    Address::constant(0),
                    Address::constant(3),
                    ByteType::INTEGER
                ),
                Instruction::r#return(Address::register(0), ByteType::LIST_INTEGER),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn list_string() {
    let source = create_function_case(list_cases::LIST_STRING, ByteType::LIST_STRING);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::list(DustType::String),
            instructions: vec![
                Instruction::new_list(0, Address::constant(0), ByteType::LIST_STRING),
                Instruction::set_list(
                    0,
                    Address::constant(1),
                    Address::constant(2),
                    ByteType::STRING
                ),
                Instruction::set_list(
                    0,
                    Address::constant(3),
                    Address::constant(4),
                    ByteType::STRING
                ),
                Instruction::set_list(
                    0,
                    Address::constant(5),
                    Address::constant(6),
                    ByteType::STRING
                ),
                Instruction::r#return(Address::register(0), ByteType::LIST_STRING),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn list_equal() {
    let source = create_function_case(list_cases::LIST_EQUAL, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0), ByteType::LIST_BOOLEAN),
                Instruction::set_list(
                    0,
                    Address::encoded(true as u16),
                    Address::constant(1),
                    ByteType::BOOLEAN
                ),
                Instruction::set_list(
                    0,
                    Address::encoded(false as u16),
                    Address::constant(2),
                    ByteType::BOOLEAN
                ),
                Instruction::new_list(1, Address::constant(0), ByteType::LIST_BOOLEAN),
                Instruction::set_list(
                    1,
                    Address::encoded(true as u16),
                    Address::constant(1),
                    ByteType::BOOLEAN
                ),
                Instruction::set_list(
                    1,
                    Address::encoded(false as u16),
                    Address::constant(2),
                    ByteType::BOOLEAN
                ),
                Instruction::equal(
                    true,
                    Address::register(0),
                    Address::register(1),
                    ByteType::LIST_BOOLEAN
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    ByteType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), ByteType::BOOLEAN),
                Instruction::r#return(Address::register(2), ByteType::BOOLEAN),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn list_not_equal() {
    let source = create_function_case(list_cases::LIST_NOT_EQUAL, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0), ByteType::LIST_BYTE),
                Instruction::set_list(
                    0,
                    Address::encoded(0x2A),
                    Address::constant(1),
                    ByteType::BYTE
                ),
                Instruction::set_list(
                    0,
                    Address::encoded(0x2B),
                    Address::constant(2),
                    ByteType::BYTE
                ),
                Instruction::new_list(1, Address::constant(0), ByteType::LIST_BYTE),
                Instruction::set_list(
                    1,
                    Address::encoded(0x2B),
                    Address::constant(1),
                    ByteType::BYTE
                ),
                Instruction::set_list(
                    1,
                    Address::encoded(0x2A),
                    Address::constant(2),
                    ByteType::BYTE
                ),
                Instruction::equal(
                    false,
                    Address::register(0),
                    Address::register(1),
                    ByteType::LIST_BYTE
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    ByteType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), ByteType::BOOLEAN),
                Instruction::r#return(Address::register(2), ByteType::BOOLEAN),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn list_greater_than() {
    let source = create_function_case(list_cases::LIST_GREATER_THAN, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0), ByteType::LIST_CHARACTER),
                Instruction::set_list(
                    0,
                    Address::constant(1),
                    Address::constant(2),
                    ByteType::CHARACTER
                ),
                Instruction::set_list(
                    0,
                    Address::constant(3),
                    Address::constant(4),
                    ByteType::CHARACTER
                ),
                Instruction::new_list(1, Address::constant(0), ByteType::LIST_CHARACTER),
                Instruction::set_list(
                    1,
                    Address::constant(3),
                    Address::constant(2),
                    ByteType::CHARACTER
                ),
                Instruction::set_list(
                    1,
                    Address::constant(1),
                    Address::constant(4),
                    ByteType::CHARACTER
                ),
                Instruction::less_equal(
                    false,
                    Address::register(0),
                    Address::register(1),
                    ByteType::LIST_CHARACTER
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    ByteType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), ByteType::BOOLEAN),
                Instruction::r#return(Address::register(2), ByteType::BOOLEAN),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn list_less_than() {
    let source = create_function_case(list_cases::LIST_LESS_THAN, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0), ByteType::LIST_FLOAT),
                Instruction::set_list(
                    0,
                    Address::constant(1),
                    Address::constant(2),
                    ByteType::FLOAT
                ),
                Instruction::set_list(
                    0,
                    Address::constant(3),
                    Address::constant(4),
                    ByteType::FLOAT
                ),
                Instruction::new_list(1, Address::constant(0), ByteType::LIST_FLOAT),
                Instruction::set_list(
                    1,
                    Address::constant(3),
                    Address::constant(2),
                    ByteType::FLOAT
                ),
                Instruction::set_list(
                    1,
                    Address::constant(1),
                    Address::constant(4),
                    ByteType::FLOAT
                ),
                Instruction::less(
                    true,
                    Address::register(0),
                    Address::register(1),
                    ByteType::LIST_FLOAT
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    ByteType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), ByteType::BOOLEAN),
                Instruction::r#return(Address::register(2), ByteType::BOOLEAN),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn list_greater_than_or_equal() {
    let source = create_function_case(list_cases::LIST_GREATER_THAN_OR_EQUAL, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0), ByteType::LIST_INTEGER),
                Instruction::set_list(
                    0,
                    Address::constant(1),
                    Address::constant(2),
                    ByteType::INTEGER
                ),
                Instruction::set_list(
                    0,
                    Address::constant(0),
                    Address::constant(1),
                    ByteType::INTEGER
                ),
                Instruction::new_list(1, Address::constant(0), ByteType::LIST_INTEGER),
                Instruction::set_list(
                    1,
                    Address::constant(1),
                    Address::constant(2),
                    ByteType::INTEGER
                ),
                Instruction::set_list(
                    1,
                    Address::constant(0),
                    Address::constant(1),
                    ByteType::INTEGER
                ),
                Instruction::less(
                    false,
                    Address::register(0),
                    Address::register(1),
                    ByteType::LIST_INTEGER
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    ByteType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), ByteType::BOOLEAN),
                Instruction::r#return(Address::register(2), ByteType::BOOLEAN),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn list_less_than_or_equal() {
    let source = create_function_case(list_cases::LIST_LESS_THAN_OR_EQUAL, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0), ByteType::LIST_STRING),
                Instruction::set_list(
                    0,
                    Address::constant(1),
                    Address::constant(2),
                    ByteType::STRING
                ),
                Instruction::set_list(
                    0,
                    Address::constant(3),
                    Address::constant(4),
                    ByteType::STRING
                ),
                Instruction::new_list(1, Address::constant(0), ByteType::LIST_STRING),
                Instruction::set_list(
                    1,
                    Address::constant(1),
                    Address::constant(2),
                    ByteType::STRING
                ),
                Instruction::set_list(
                    1,
                    Address::constant(3),
                    Address::constant(4),
                    ByteType::STRING
                ),
                Instruction::less_equal(
                    true,
                    Address::register(0),
                    Address::register(1),
                    ByteType::LIST_STRING
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    ByteType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), ByteType::BOOLEAN),
                Instruction::r#return(Address::register(2), ByteType::BOOLEAN),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn list_index_boolean() {
    let source = create_function_case(list_cases::LIST_INDEX_BOOLEAN, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0), ByteType::LIST_BOOLEAN),
                Instruction::set_list(
                    0,
                    Address::encoded(true as u16),
                    Address::constant(1),
                    ByteType::BOOLEAN
                ),
                Instruction::set_list(
                    0,
                    Address::encoded(false as u16),
                    Address::constant(2),
                    ByteType::BOOLEAN
                ),
                Instruction::set_list(
                    0,
                    Address::encoded(true as u16),
                    Address::constant(3),
                    ByteType::BOOLEAN
                ),
                Instruction::get_list(
                    1,
                    Address::register(0),
                    Address::constant(1),
                    ByteType::BOOLEAN
                ),
                Instruction::r#return(Address::register(1), ByteType::BOOLEAN),
            ],
            register_count: 2,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn list_index_byte() {
    let source = create_function_case(list_cases::LIST_INDEX_BYTE, ByteType::BYTE);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Byte,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0), ByteType::LIST_BYTE),
                Instruction::set_list(
                    0,
                    Address::encoded(0x2A),
                    Address::constant(1),
                    ByteType::BYTE
                ),
                Instruction::set_list(
                    0,
                    Address::encoded(0x2B),
                    Address::constant(2),
                    ByteType::BYTE
                ),
                Instruction::set_list(
                    0,
                    Address::encoded(0x2C),
                    Address::constant(3),
                    ByteType::BYTE
                ),
                Instruction::get_list(
                    1,
                    Address::register(0),
                    Address::constant(2),
                    ByteType::BYTE
                ),
                Instruction::r#return(Address::register(1), ByteType::BYTE),
            ],
            register_count: 2,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn list_index_character() {
    let source = create_function_case(list_cases::LIST_INDEX_CHARACTER, ByteType::CHARACTER);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Character,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0), ByteType::LIST_CHARACTER),
                Instruction::set_list(
                    0,
                    Address::constant(1),
                    Address::constant(2),
                    ByteType::CHARACTER
                ),
                Instruction::set_list(
                    0,
                    Address::constant(3),
                    Address::constant(4),
                    ByteType::CHARACTER
                ),
                Instruction::set_list(
                    0,
                    Address::constant(5),
                    Address::constant(6),
                    ByteType::CHARACTER
                ),
                Instruction::get_list(
                    1,
                    Address::register(0),
                    Address::constant(6),
                    ByteType::CHARACTER
                ),
                Instruction::r#return(Address::register(1), ByteType::CHARACTER),
            ],
            register_count: 2,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn list_index_float() {
    let source = create_function_case(list_cases::LIST_INDEX_FLOAT, ByteType::FLOAT);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Float,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0), ByteType::LIST_FLOAT),
                Instruction::set_list(
                    0,
                    Address::constant(1),
                    Address::constant(2),
                    ByteType::FLOAT
                ),
                Instruction::set_list(
                    0,
                    Address::constant(3),
                    Address::constant(4),
                    ByteType::FLOAT
                ),
                Instruction::set_list(
                    0,
                    Address::constant(5),
                    Address::constant(6),
                    ByteType::FLOAT
                ),
                Instruction::get_list(
                    1,
                    Address::register(0),
                    Address::constant(4),
                    ByteType::FLOAT
                ),
                Instruction::r#return(Address::register(1), ByteType::FLOAT),
            ],
            register_count: 2,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn list_index_integer() {
    let source = create_function_case(list_cases::LIST_INDEX_INTEGER, ByteType::INTEGER);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0), ByteType::LIST_INTEGER),
                Instruction::set_list(
                    0,
                    Address::constant(1),
                    Address::constant(2),
                    ByteType::INTEGER
                ),
                Instruction::set_list(
                    0,
                    Address::constant(3),
                    Address::constant(1),
                    ByteType::INTEGER
                ),
                Instruction::set_list(
                    0,
                    Address::constant(0),
                    Address::constant(3),
                    ByteType::INTEGER
                ),
                Instruction::get_list(
                    1,
                    Address::register(0),
                    Address::constant(2),
                    ByteType::INTEGER
                ),
                Instruction::r#return(Address::register(1), ByteType::INTEGER),
            ],
            register_count: 2,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn list_index_string() {
    let source = create_function_case(list_cases::LIST_INDEX_STRING, ByteType::STRING);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::String,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0), ByteType::LIST_STRING),
                Instruction::set_list(
                    0,
                    Address::constant(1),
                    Address::constant(2),
                    ByteType::STRING
                ),
                Instruction::set_list(
                    0,
                    Address::constant(3),
                    Address::constant(4),
                    ByteType::STRING
                ),
                Instruction::set_list(
                    0,
                    Address::constant(5),
                    Address::constant(6),
                    ByteType::STRING
                ),
                Instruction::get_list(
                    1,
                    Address::register(0),
                    Address::constant(6),
                    ByteType::STRING
                ),
                Instruction::r#return(Address::register(1), ByteType::STRING),
            ],
            register_count: 2,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_list_boolean() {
    let source = create_function_case(list_cases::LOCAL_LIST_BOOLEAN, ByteType::LIST_BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::list(DustType::Boolean),
            instructions: vec![
                Instruction::new_list(0, Address::constant(0), ByteType::LIST_BOOLEAN),
                Instruction::set_list(
                    0,
                    Address::encoded(true as u16),
                    Address::constant(1),
                    ByteType::BOOLEAN
                ),
                Instruction::set_list(
                    0,
                    Address::encoded(false as u16),
                    Address::constant(2),
                    ByteType::BOOLEAN
                ),
                Instruction::set_list(
                    0,
                    Address::encoded(true as u16),
                    Address::constant(3),
                    ByteType::BOOLEAN
                ),
                Instruction::r#return(Address::register(0), ByteType::LIST_BOOLEAN),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_list_equal() {
    let source = create_function_case(list_cases::LOCAL_LIST_EQUAL, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0), ByteType::LIST_BOOLEAN),
                Instruction::set_list(
                    0,
                    Address::encoded(true as u16),
                    Address::constant(1),
                    ByteType::BOOLEAN
                ),
                Instruction::set_list(
                    0,
                    Address::encoded(false as u16),
                    Address::constant(2),
                    ByteType::BOOLEAN
                ),
                Instruction::new_list(1, Address::constant(0), ByteType::LIST_BOOLEAN),
                Instruction::set_list(
                    1,
                    Address::encoded(true as u16),
                    Address::constant(1),
                    ByteType::BOOLEAN
                ),
                Instruction::set_list(
                    1,
                    Address::encoded(false as u16),
                    Address::constant(2),
                    ByteType::BOOLEAN
                ),
                Instruction::equal(
                    true,
                    Address::register(0),
                    Address::register(1),
                    ByteType::LIST_BOOLEAN
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    ByteType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), ByteType::BOOLEAN),
                Instruction::r#return(Address::register(2), ByteType::BOOLEAN),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_list_not_equal() {
    let source = create_function_case(list_cases::LOCAL_LIST_NOT_EQUAL, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0), ByteType::LIST_BYTE),
                Instruction::set_list(
                    0,
                    Address::encoded(0x2A),
                    Address::constant(1),
                    ByteType::BYTE
                ),
                Instruction::set_list(
                    0,
                    Address::encoded(0x2B),
                    Address::constant(2),
                    ByteType::BYTE
                ),
                Instruction::new_list(1, Address::constant(0), ByteType::LIST_BYTE),
                Instruction::set_list(
                    1,
                    Address::encoded(0x2B),
                    Address::constant(1),
                    ByteType::BYTE
                ),
                Instruction::set_list(
                    1,
                    Address::encoded(0x2A),
                    Address::constant(2),
                    ByteType::BYTE
                ),
                Instruction::equal(
                    false,
                    Address::register(0),
                    Address::register(1),
                    ByteType::LIST_BYTE
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    ByteType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), ByteType::BOOLEAN),
                Instruction::r#return(Address::register(2), ByteType::BOOLEAN),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_list_greater_than() {
    let source = create_function_case(list_cases::LOCAL_LIST_GREATER_THAN, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0), ByteType::LIST_CHARACTER),
                Instruction::set_list(
                    0,
                    Address::constant(1),
                    Address::constant(2),
                    ByteType::CHARACTER
                ),
                Instruction::set_list(
                    0,
                    Address::constant(3),
                    Address::constant(4),
                    ByteType::CHARACTER
                ),
                Instruction::new_list(1, Address::constant(0), ByteType::LIST_CHARACTER),
                Instruction::set_list(
                    1,
                    Address::constant(3),
                    Address::constant(2),
                    ByteType::CHARACTER
                ),
                Instruction::set_list(
                    1,
                    Address::constant(1),
                    Address::constant(4),
                    ByteType::CHARACTER
                ),
                Instruction::less_equal(
                    false,
                    Address::register(0),
                    Address::register(1),
                    ByteType::LIST_CHARACTER
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    ByteType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), ByteType::BOOLEAN),
                Instruction::r#return(Address::register(2), ByteType::BOOLEAN),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_list_less_than() {
    let source = create_function_case(list_cases::LOCAL_LIST_LESS_THAN, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0), ByteType::LIST_FLOAT),
                Instruction::set_list(
                    0,
                    Address::constant(1),
                    Address::constant(2),
                    ByteType::FLOAT
                ),
                Instruction::set_list(
                    0,
                    Address::constant(3),
                    Address::constant(4),
                    ByteType::FLOAT
                ),
                Instruction::new_list(1, Address::constant(0), ByteType::LIST_FLOAT),
                Instruction::set_list(
                    1,
                    Address::constant(3),
                    Address::constant(2),
                    ByteType::FLOAT
                ),
                Instruction::set_list(
                    1,
                    Address::constant(1),
                    Address::constant(4),
                    ByteType::FLOAT
                ),
                Instruction::less(
                    true,
                    Address::register(1),
                    Address::register(0),
                    ByteType::LIST_FLOAT
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    ByteType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), ByteType::BOOLEAN),
                Instruction::r#return(Address::register(2), ByteType::BOOLEAN),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_list_greater_than_or_equal() {
    let source = create_function_case(
        list_cases::LOCAL_LIST_GREATER_THAN_OR_EQUAL,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0), ByteType::LIST_INTEGER),
                Instruction::set_list(
                    0,
                    Address::constant(1),
                    Address::constant(2),
                    ByteType::INTEGER
                ),
                Instruction::set_list(
                    0,
                    Address::constant(0),
                    Address::constant(1),
                    ByteType::INTEGER
                ),
                Instruction::new_list(1, Address::constant(0), ByteType::LIST_INTEGER),
                Instruction::set_list(
                    1,
                    Address::constant(1),
                    Address::constant(2),
                    ByteType::INTEGER
                ),
                Instruction::set_list(
                    1,
                    Address::constant(0),
                    Address::constant(1),
                    ByteType::INTEGER
                ),
                Instruction::less(
                    false,
                    Address::register(0),
                    Address::register(1),
                    ByteType::LIST_INTEGER
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    ByteType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), ByteType::BOOLEAN),
                Instruction::r#return(Address::register(2), ByteType::BOOLEAN),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_list_less_than_or_equal() {
    let source = create_function_case(
        list_cases::LOCAL_LIST_LESS_THAN_OR_EQUAL,
        ByteType::BOOLEAN,
    );
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::new_list(0, Address::constant(0), ByteType::LIST_STRING),
                Instruction::set_list(
                    0,
                    Address::constant(1),
                    Address::constant(2),
                    ByteType::STRING
                ),
                Instruction::set_list(
                    0,
                    Address::constant(3),
                    Address::constant(4),
                    ByteType::STRING
                ),
                Instruction::new_list(1, Address::constant(0), ByteType::LIST_STRING),
                Instruction::set_list(
                    1,
                    Address::constant(1),
                    Address::constant(2),
                    ByteType::STRING
                ),
                Instruction::set_list(
                    1,
                    Address::constant(3),
                    Address::constant(4),
                    ByteType::STRING
                ),
                Instruction::less_equal(
                    true,
                    Address::register(0),
                    Address::register(1),
                    ByteType::LIST_STRING
                ),
                Instruction::move_with_jump(
                    2,
                    Address::encoded(false as u16),
                    ByteType::BOOLEAN,
                    1,
                    true
                ),
                Instruction::r#move(2, Address::encoded(true as u16), ByteType::BOOLEAN),
                Instruction::r#return(Address::register(2), ByteType::BOOLEAN),
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

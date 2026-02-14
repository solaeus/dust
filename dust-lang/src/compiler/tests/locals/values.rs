use crate::{
    compiler::compile_main,
    dust_type::DustType,
    instruction::{Address, ByteType, Instruction},
    prototype::Prototype,
    tests::local_cases,
};

#[test]
fn local_boolean() {
    let source = local_cases::LOCAL_BOOLEAN;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::encoded(true as u16)),
                Instruction::r#return(Address::register(0)),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_byte() {
    let source = local_cases::LOCAL_BYTE;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Byte,
            instructions: vec![
                Instruction::r#move(0, Address::encoded(42)),
                Instruction::r#return(Address::register(0)),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_character() {
    let source = local_cases::LOCAL_CHARACTER;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Character,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#return(Address::register(0)),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_float() {
    let source = local_cases::LOCAL_FLOAT;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Float,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#return(Address::register(0)),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_integer() {
    let source = local_cases::LOCAL_INTEGER;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#return(Address::register(0)),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_string() {
    let source = local_cases::LOCAL_STRING;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::String,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#return(Address::register(0)),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_function() {
    let source = local_cases::LOCAL_FUNCTION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![
                Instruction::r#move(0, Address::constant(1)),
                Instruction::call(Some(1), Address::register(0), 0, 1),
                Instruction::r#return(Address::register(1)),
            ],
            call_arguments: vec![(Address::constant(1), ByteType::INTEGER)],
            register_count: 2,
            ..Prototype::dummy()
        }
    );
}

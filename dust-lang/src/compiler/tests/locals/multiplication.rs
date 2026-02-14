use crate::{
    compiler::compile_main,
    dust_type::DustType,
    instruction::{Address, Instruction},
    prototype::Prototype,
    tests::local_cases,
};

#[test]
fn local_mut_byte_multiplication() {
    let source = local_cases::LOCAL_MUT_BYTE_MULTIPLICATION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Byte,
            instructions: vec![
                Instruction::r#move(0, Address::encoded_byte(14)),
                Instruction::multiply(0, Address::register(0), Address::encoded_byte(3)),
                Instruction::r#return(Address::register(0))
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_mut_float_multiplication() {
    let source = local_cases::LOCAL_MUT_FLOAT_MULTIPLICATION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Float,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::multiply(0, Address::register(0), Address::constant(1)),
                Instruction::r#return(Address::register(0))
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_mut_integer_multiplication() {
    let source = local_cases::LOCAL_MUT_INTEGER_MULTIPLICATION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::multiply(0, Address::register(0), Address::constant(1)),
                Instruction::r#return(Address::register(0))
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_byte_multiplication() {
    let source = local_cases::LOCAL_BYTE_MULTIPLICATION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Byte,
            instructions: vec![
                Instruction::r#move(0, Address::encoded_byte(14)),
                Instruction::r#move(1, Address::encoded_byte(3)),
                Instruction::multiply(2, Address::register(0), Address::register(1)),
                Instruction::r#return(Address::register(2))
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_float_multiplication() {
    let source = local_cases::LOCAL_FLOAT_MULTIPLICATION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Float,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#move(1, Address::constant(1)),
                Instruction::multiply(2, Address::register(0), Address::register(1)),
                Instruction::r#return(Address::register(2))
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_integer_multiplication() {
    let source = local_cases::LOCAL_INTEGER_MULTIPLICATION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::r#move(1, Address::constant(1)),
                Instruction::multiply(2, Address::register(0), Address::register(1)),
                Instruction::r#return(Address::register(2))
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

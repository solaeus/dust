use crate::compiler::Symbol;
use crate::{
    compiler::compile_main,
    dust_type::{DustFunctionType, Type},
    instruction::{Address, Instruction, OperandType},
    prototype::Prototype,
    tests::local_cases,
};

#[test]
fn local_byte_exponent() {
    let source = local_cases::LOCAL_BYTE_EXPONENT;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Byte),
            instructions: vec![
                Instruction::r#move(0, Address::encoded(2), OperandType::BYTE),
                Instruction::r#move(1, Address::encoded(3), OperandType::BYTE),
                Instruction::power(
                    2,
                    Address::register(0),
                    Address::register(1),
                    OperandType::BYTE
                ),
                Instruction::r#return(Address::register(2), OperandType::BYTE)
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_float_exponent() {
    let source = local_cases::LOCAL_FLOAT_EXPONENT;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Float),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::FLOAT),
                Instruction::r#move(1, Address::constant(1), OperandType::FLOAT),
                Instruction::power(
                    2,
                    Address::register(0),
                    Address::register(1),
                    OperandType::FLOAT
                ),
                Instruction::r#return(Address::register(2), OperandType::FLOAT)
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_integer_exponent() {
    let source = local_cases::LOCAL_INTEGER_EXPONENT;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER),
                Instruction::r#move(1, Address::constant(1), OperandType::INTEGER),
                Instruction::power(
                    2,
                    Address::register(0),
                    Address::register(1),
                    OperandType::INTEGER
                ),
                Instruction::r#return(Address::register(2), OperandType::INTEGER)
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_mut_byte_exponent() {
    let source = local_cases::LOCAL_MUT_BYTE_EXPONENT;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Byte),
            instructions: vec![
                Instruction::r#move(0, Address::encoded(2), OperandType::BYTE),
                Instruction::power(
                    0,
                    Address::register(0),
                    Address::encoded(3),
                    OperandType::BYTE
                ),
                Instruction::r#return(Address::register(0), OperandType::BYTE)
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_mut_float_exponent() {
    let source = local_cases::LOCAL_MUT_FLOAT_EXPONENT;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Float),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::FLOAT),
                Instruction::power(
                    0,
                    Address::register(0),
                    Address::constant(1),
                    OperandType::FLOAT
                ),
                Instruction::r#return(Address::register(0), OperandType::FLOAT)
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

#[test]
fn local_mut_integer_exponent() {
    let source = local_cases::LOCAL_MUT_INTEGER_EXPONENT;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            symbol: Symbol::MAIN,
            function_type: DustFunctionType::new([], [], DustType::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER),
                Instruction::power(
                    0,
                    Address::register(0),
                    Address::constant(1),
                    OperandType::INTEGER
                ),
                Instruction::r#return(Address::register(0), OperandType::INTEGER)
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

use crate::{
    compiler::compile_main,
    dust_type::{DustFunctionType, DustType},
    instruction::{Address, Instruction, OperandType},
    prototype::Prototype,
    symbol_table::SymbolId,
    tests::local_cases,
};

#[test]
fn local_byte_division() {
    let source = local_cases::LOCAL_BYTE_DIVISION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: DustFunctionType::new([], [], DustType::Byte),
            instructions: vec![
                Instruction::r#move(0, Address::encoded(84), OperandType::BYTE),
                Instruction::r#move(1, Address::encoded(2), OperandType::BYTE),
                Instruction::divide(
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
fn local_float_division() {
    let source = local_cases::LOCAL_FLOAT_DIVISION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: DustFunctionType::new([], [], DustType::Float),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::FLOAT),
                Instruction::r#move(1, Address::constant(1), OperandType::FLOAT),
                Instruction::divide(
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
fn local_integer_division() {
    let source = local_cases::LOCAL_INTEGER_DIVISION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: DustFunctionType::new([], [], DustType::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER),
                Instruction::r#move(1, Address::constant(1), OperandType::INTEGER),
                Instruction::divide(
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
fn local_mut_byte_division() {
    let source = local_cases::LOCAL_MUT_BYTE_DIVISION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: DustFunctionType::new([], [], DustType::Byte),
            instructions: vec![
                Instruction::r#move(0, Address::encoded(84), OperandType::BYTE),
                Instruction::divide(
                    0,
                    Address::register(0),
                    Address::encoded(2),
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
fn local_mut_float_division() {
    let source = local_cases::LOCAL_MUT_FLOAT_DIVISION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: DustFunctionType::new([], [], DustType::Float),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::FLOAT),
                Instruction::divide(
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
fn local_mut_integer_division() {
    let source = local_cases::LOCAL_MUT_INTEGER_DIVISION;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: DustFunctionType::new([], [], DustType::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER),
                Instruction::divide(
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

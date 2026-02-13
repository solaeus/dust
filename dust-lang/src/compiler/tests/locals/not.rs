use crate::{
    compiler::compile_main,
    dust_type::{DustFunctionType, DustType},
    instruction::{Address, Instruction, OperandType},
    prototype::Prototype,
    symbol_table::SymbolId,
    tests::local_cases,
};

#[test]
fn local_boolean_not() {
    let source = local_cases::LOCAL_BOOLEAN_NOT;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![
                Instruction::r#move(0, Address::encoded(true as u16), OperandType::BOOLEAN),
                Instruction::negate(1, Address::register(0), OperandType::BOOLEAN),
                Instruction::r#return(Address::register(1), OperandType::BOOLEAN)
            ],
            register_count: 2,
            ..Prototype::dummy()
        }
    );
}

use crate::{
    compiler::compile_main,
    dust_type::{DustFunctionType, DustType},
    instruction::{Address, Instruction, OperandType},
    prototype::Prototype,
    symbol_table::SymbolId,
    tests::local_cases,
};

#[test]
fn local_boolean_and() {
    let source = local_cases::LOCAL_BOOLEAN_AND;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![
                Instruction::r#move(0, Address::encoded(true as u16), OperandType::BOOLEAN),
                Instruction::r#move(1, Address::encoded(false as u16), OperandType::BOOLEAN),
                Instruction::test(Address::register(0), false, 1),
                Instruction::move_with_jump(2, Address::register(1), OperandType::BOOLEAN, 1, true),
                Instruction::r#move(2, Address::register(0), OperandType::BOOLEAN),
                Instruction::r#return(Address::register(2), OperandType::BOOLEAN)
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

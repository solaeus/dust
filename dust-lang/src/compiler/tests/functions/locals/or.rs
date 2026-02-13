use crate::{
    compiler::compile,
    dust_type::{DustFunctionType, DustType},
    instruction::{Address, Instruction, OperandType},
    prototype::{Prototype, PrototypeId},
    symbol_table::SymbolId,
    tests::{create_function_case, local_cases},
};

#[test]
fn local_boolean_or() {
    let source = create_function_case(local_cases::LOCAL_BOOLEAN_OR, OperandType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            prototype_id: PrototypeId(1),

            function_type: DustFunctionType::new([], [], DustType::Boolean),
            instructions: vec![
                Instruction::r#move(0, Address::encoded(true as u16), OperandType::BOOLEAN),
                Instruction::r#move(1, Address::encoded(false as u16), OperandType::BOOLEAN),
                Instruction::test(Address::register(0), true, 1),
                Instruction::move_with_jump(2, Address::register(1), OperandType::BOOLEAN, 1, true),
                Instruction::r#move(2, Address::register(0), OperandType::BOOLEAN),
                Instruction::r#return(Address::register(2), OperandType::BOOLEAN)
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

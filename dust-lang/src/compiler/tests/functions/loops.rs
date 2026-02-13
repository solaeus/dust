use crate::{
    compiler::compile,
    dust_type::{DustFunctionType, DustType},
    instruction::{Address, Instruction, OperandType},
    prototype::{Prototype, PrototypeId},
    symbol_table::SymbolId,
    tests::{create_function_case, loop_cases},
};

#[test]
fn while_loop() {
    let source = create_function_case(loop_cases::WHILE_LOOP, OperandType::INTEGER);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            prototype_id: PrototypeId(1),

            function_type: DustFunctionType::new([], [], DustType::Integer),
            instructions: vec![
                Instruction::r#move(0, Address::constant(0), OperandType::INTEGER),
                Instruction::less(
                    true,
                    Address::register(0),
                    Address::constant(1),
                    OperandType::INTEGER
                ),
                Instruction::jump(2, true),
                Instruction::add(
                    0,
                    Address::register(0),
                    Address::constant(2),
                    OperandType::INTEGER
                ),
                Instruction::jump(2, false),
                Instruction::r#return(Address::register(0), OperandType::INTEGER),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

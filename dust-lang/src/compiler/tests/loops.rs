use crate::compiler::Symbol;
use crate::{
    compiler::compile_main,
    dust_type::{DustFunctionType, Type},
    instruction::{Address, Instruction, OperandType},
    prototype::Prototype,
    tests::loop_cases,
};

#[test]
fn while_loop() {
    let source = loop_cases::WHILE_LOOP;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            symbol: Symbol::MAIN,
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

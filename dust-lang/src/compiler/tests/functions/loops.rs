use crate::{
    compiler::compile,
    dust_type::DustType,
    instruction::{Address, Instruction, ByteType},
    prototype::Prototype,
    tests::{create_function_case, loop_cases},
};

#[test]
fn while_loop() {
    let source = create_function_case(loop_cases::WHILE_LOOP, ByteType::INTEGER);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Integer,
            instructions: vec![
                Instruction::r#move(0, Address::constant(0)),
                Instruction::less(true, Address::register(0), Address::constant(1)),
                Instruction::jump(2, true),
                Instruction::add(0, Address::register(0), Address::constant(2)),
                Instruction::jump(2, false),
                Instruction::r#return(Address::register(0)),
            ],
            register_count: 1,
            ..Prototype::dummy()
        }
    );
}

use crate::{
    compiler::compile_main,
    dust_type::DustType,
    instruction::{Address, Instruction},
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

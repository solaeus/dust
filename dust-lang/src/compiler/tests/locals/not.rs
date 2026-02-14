use crate::{
    compiler::compile_main,
    dust_type::DustType,
    instruction::{Address, Instruction, ByteType},
    prototype::Prototype,
    tests::local_cases,
};

#[test]
fn local_boolean_not() {
    let source = local_cases::LOCAL_BOOLEAN_NOT;
    let prototype = compile_main(source).unwrap();

    assert_eq!(
        prototype,
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::encoded(true as u16), ByteType::BOOLEAN),
                Instruction::negate(1, Address::register(0), ByteType::BOOLEAN),
                Instruction::r#return(Address::register(1), ByteType::BOOLEAN)
            ],
            register_count: 2,
            ..Prototype::dummy()
        }
    );
}

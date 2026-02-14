use crate::{
    compiler::compile,
    dust_type::DustType,
    instruction::{Address, Instruction, ByteType},
    prototype::Prototype,
    tests::{create_function_case, local_cases},
};

#[test]
fn local_boolean_and() {
    let source = create_function_case(local_cases::LOCAL_BOOLEAN_AND, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::encoded_boolean(true)),
                Instruction::r#move(1, Address::encoded_boolean(false)),
                Instruction::test(Address::register(0), false, 1),
                Instruction::move_with_jump(2, Address::register(1), 1, true),
                Instruction::r#move(2, Address::register(0)),
                Instruction::r#return(Address::register(2))
            ],
            register_count: 3,
            ..Prototype::dummy()
        }
    );
}

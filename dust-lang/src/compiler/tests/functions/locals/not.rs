use crate::{
    compiler::compile,
    dust_type::DustType,
    instruction::{Address, Instruction, ByteType},
    prototype::Prototype,
    tests::{create_function_case, local_cases},
};

#[test]
fn local_boolean_not() {
    let source = create_function_case(local_cases::LOCAL_BOOLEAN_NOT, ByteType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            return_type: DustType::Boolean,
            instructions: vec![
                Instruction::r#move(0, Address::encoded(true as u16)),
                Instruction::negate(1, Address::register(0)),
                Instruction::r#return(Address::register(1))
            ],
            register_count: 2,
            ..Prototype::dummy()
        }
    );
}

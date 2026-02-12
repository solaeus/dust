use crate::compiler::Symbol;

use crate::{
    compiler::compile,
    instruction::{Address, Instruction, OperandType},
    prototype::Prototype,
    tests::{create_function_case, local_cases},
    r#type::{FunctionType, Type},
};

#[test]
fn local_boolean_and() {
    let source = create_function_case(local_cases::LOCAL_BOOLEAN_AND, OperandType::BOOLEAN);
    let prototypes = compile(&source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            id: 1,
            symbol: Symbol::MAIN,
            function_type: FunctionType::new([], [], Type::Boolean),
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

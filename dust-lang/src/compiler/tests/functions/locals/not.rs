use crate::{
    compiler::compile_prototypes,
    instruction::{Address, Instruction, OperandType},
    prototype::Prototype,
    source::{Position, SourceFileId, Span},
    tests::{create_function_case, local_cases},
    r#type::{FunctionType, Type},
};
use crate::constant_table::ConstantId;
use crate::compiler::Symbol;

#[test]
fn local_boolean_not() {
    let source = create_function_case(local_cases::LOCAL_BOOLEAN_NOT, OperandType::BOOLEAN);
    let prototypes = compile_prototypes(source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            index: 1,
            name: Symbol::Source {
                constant_id: ConstantId(0),
                position: Position::new(SourceFileId::MAIN, Span(16, 22))
            },
            function_type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::r#move(0, Address::encoded(true as u16), OperandType::BOOLEAN),
                Instruction::negate(1, Address::register(0), OperandType::BOOLEAN),
                Instruction::r#return(Address::register(1), OperandType::BOOLEAN)
            ],
            register_count: 2,
            ..Default::default()
        }
    );
}

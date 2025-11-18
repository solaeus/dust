use crate::{
    prototype::Prototype,
    compiler::compile,
    instruction::{Address, Instruction, OperandType},
    tests::{create_function_case, local_cases},
    source::{Position, SourceFileId, Span},
    r#type::{FunctionType, Type},
};

#[test]
fn local_boolean_or() {
    let source = create_function_case(local_cases::LOCAL_BOOLEAN_OR);
    let prototypes = compile(source).unwrap();

    assert_eq!(prototypes.len(), 2);
    assert_eq!(
        prototypes[1],
        Prototype {
            name_position: Some(Position {
                file_id: SourceFileId(0),
                span: Span(16, 22)
            }),
            function_type: FunctionType::new([], [], Type::Boolean),
            instructions: vec![
                Instruction::r#move(0, Address::encoded(true as u16), OperandType::BOOLEAN),
                Instruction::r#move(1, Address::encoded(false as u16), OperandType::BOOLEAN),
                Instruction::test(Address::register(0), true, 1),
                Instruction::move_with_jump(2, Address::register(1), OperandType::BOOLEAN, 1, true),
                Instruction::r#move(2, Address::register(0), OperandType::BOOLEAN),
                Instruction::r#return(true, Address::register(2), OperandType::BOOLEAN)
            ],
            register_count: 3,
            ..Default::default()
        }
    );
}

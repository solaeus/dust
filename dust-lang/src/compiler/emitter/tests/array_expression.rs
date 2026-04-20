use smallvec::smallvec;

use crate::{
    compiler::emitter::tests::emit_function,
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

#[test]
fn three_elements() {
    let prototype = emit_function("fn foo() -> [i32; 3] { [1, 2, 3] }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 1),
                Instruction::r#move(1, OperandType::I_32, MemoryKind::ENCODED, 2),
                Instruction::r#move(2, OperandType::I_32, MemoryKind::ENCODED, 3),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32, OperandType::I_32, OperandType::I_32],
            register_count: 3,
            argument_count: 0,
        }
    );
}

#[test]
fn one_element() {
    let prototype = emit_function("fn foo() -> [i32; 1] { [42] }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 42),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
        }
    );
}

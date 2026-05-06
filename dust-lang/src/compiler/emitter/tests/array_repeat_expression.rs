use smallvec::smallvec;

use crate::{
    compiler::emitter::tests::emit_function,
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

#[test]
fn fill() {
    let prototype = emit_function("fn foo() -> [i32; 3] { [0; 3] }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 0),
                Instruction::r#move(1, OperandType::I_32, MemoryKind::REGISTER, 0),
                Instruction::r#move(2, OperandType::I_32, MemoryKind::REGISTER, 0),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32, OperandType::I_32, OperandType::I_32],
            register_count: 3,
            argument_count: 0,
        }
    );
}

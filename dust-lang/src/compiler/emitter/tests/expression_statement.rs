use smallvec::smallvec;

use crate::{
    compiler::emitter::tests::emit_function,
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

#[test]
fn reassignment() {
    let prototype = emit_function("fn foo() { let mut x: i32 = 0; x = 1; }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 0),
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 1),
                Instruction::r#return(),
            ],
            return_types: smallvec![],
            register_count: 1,
            argument_count: 0,
        }
    );
}

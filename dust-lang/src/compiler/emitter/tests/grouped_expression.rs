use smallvec::smallvec;

use crate::{
    compiler::emitter::tests::emit_function,
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
};

#[test]
fn addition() {
    let prototype = emit_function("fn foo() -> i32 { (1 + 2) }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::ENCODED, 3),
                Instruction::r#return(),
            ],
            return_types: smallvec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
        }
    );
}

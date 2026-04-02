use crate::{
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
    resolver::symbols::SymbolId,
    source::{Position, SourceFileId, Span},
};

use super::emit_function;

#[test]
fn if_else_returning_value() {
    let prototype = emit_function("fn foo() -> i32 { if true { 1 } else { 2 } }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::test(true, MemoryKind::CONSTANT, 0, 2),
                Instruction::r#move(0, OperandType::I_32, MemoryKind::CONSTANT, 1),
                Instruction::jump(1, true),
                Instruction::r#move(0, OperandType::I_32, MemoryKind::CONSTANT, 2),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
            debug_symbol_id: Some(SymbolId(14)),
            debug_position: Position::new(SourceFileId::MAIN, Span::new(0, 45)),
        }
    );
}

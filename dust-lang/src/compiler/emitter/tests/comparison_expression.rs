use crate::{
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
    resolver::symbols::SymbolId,
    source::{Position, SourceFileId, Span},
};

use super::emit_function;

#[test]
fn equal() {
    let prototype = emit_function("fn foo() -> bool { let a: i32 = 1; let b: i32 = 2; a == b }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::equal(
                    true,
                    OperandType::I_32,
                    MemoryKind::CONSTANT,
                    0,
                    MemoryKind::CONSTANT,
                    1
                ),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::BOOLEAN],
            register_count: 1,
            argument_count: 0,
            debug_symbol_id: Some(SymbolId(14)),
            debug_position: Position::new(SourceFileId::MAIN, Span::new(0, 59)),
        }
    );
}

#[test]
fn less_than() {
    let prototype = emit_function("fn foo() -> bool { let a: i32 = 1; let b: i32 = 2; a < b }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::less(
                    true,
                    OperandType::I_32,
                    MemoryKind::CONSTANT,
                    0,
                    MemoryKind::CONSTANT,
                    1
                ),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::BOOLEAN],
            register_count: 1,
            argument_count: 0,
            debug_symbol_id: Some(SymbolId(14)),
            debug_position: Position::new(SourceFileId::MAIN, Span::new(0, 58)),
        }
    );
}

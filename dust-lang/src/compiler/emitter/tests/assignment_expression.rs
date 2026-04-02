use crate::{
    instruction::{Instruction, MemoryKind, OperandType},
    prototype::Prototype,
    resolver::symbols::SymbolId,
    source::{Position, SourceFileId, Span},
};

use super::emit_function;

#[test]
fn simple_reassignment() {
    let prototype = emit_function("fn foo() -> i32 { let mut x: i32 = 1; x = 2; x }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::CONSTANT, 1),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
            debug_symbol_id: Some(SymbolId(14)),
            debug_position: Position::new(SourceFileId::MAIN, Span::new(0, 49)),
        }
    );
}

#[test]
fn compound_addition() {
    let prototype = emit_function("fn foo() -> i32 { let mut x: i32 = 1; x += 2; x }");

    assert_eq!(
        prototype,
        Prototype {
            instructions: vec![
                Instruction::r#move(0, OperandType::I_32, MemoryKind::CONSTANT, 0),
                Instruction::add(
                    0,
                    OperandType::I_32,
                    MemoryKind::REGISTER,
                    0,
                    MemoryKind::CONSTANT,
                    1
                ),
                Instruction::r#return(),
            ],
            return_types: vec![OperandType::I_32],
            register_count: 1,
            argument_count: 0,
            debug_symbol_id: Some(SymbolId(14)),
            debug_position: Position::new(SourceFileId::MAIN, Span::new(0, 50)),
        }
    );
}

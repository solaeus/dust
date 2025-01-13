//! Functions used by the compiler to optimize a chunk's bytecode during compilation.

use tracing::debug;

use crate::{Compiler, Instruction, Operation};

/// Optimizes a control flow pattern to use fewer registers and avoid using a `POINT` instruction.
/// Use this after parsing an if/else statement.
///
/// This makes the following examples compile to the same bytecode:
///
/// ```dust
/// 4 == 4
/// ```
///
/// ```dust
/// if 4 == 4 { true } else { false }
/// ```
///
/// When they occur in the sequence shown below, instructions can be optimized by taking advantage
/// of the loaders' ability to skip an instruction after loading a value. If these instructions are
/// the result of a binary expression, this will not change anything because they were already
/// emitted optimally. Control flow patterns, however, can be optimized because the load
/// instructions are from seperate expressions that each uses its own register. Since only one of
/// the two branches will be executed, this is wasteful. It would also require the compiler to emit
/// a `POINT` instruction to prevent the VM from encountering an empty register.
///
/// The instructions must be in the following order:
///     - `TEST` or any of the `EQUAL`, `LESS` or `LESS_EQUAL` instructions
///     - `JUMP`
///     - `LOAD_BOOLEAN` or `LOAD_CONSTANT`
///     - `LOAD_BOOLEAN` or `LOAD_CONSTANT`
///
/// This optimization was taken from `A No-Frills Introduction to Lua 5.1 VM Instructions` by
/// Kein-Hong Man.
pub fn control_flow_register_consolidation(compiler: &mut Compiler) {
    if !matches!(
        compiler.get_last_operations(),
        Some([
            Operation::TEST | Operation::EQUAL | Operation::LESS | Operation::LESS_EQUAL,
            Operation::JUMP,
            Operation::LOAD_BOOLEAN | Operation::LOAD_CONSTANT,
            Operation::LOAD_BOOLEAN | Operation::LOAD_CONSTANT,
        ])
    ) {
        return;
    }

    debug!("Consolidating registers for control flow optimization");

    let first_loader_index = compiler.instructions.len() - 2;
    let (first_loader, _, _) = &mut compiler.instructions.get_mut(first_loader_index).unwrap();
    let first_loader_destination = first_loader.a_field();
    *first_loader =
        Instruction::load_boolean(first_loader.a_field(), first_loader.b_field() != 0, true);

    let second_loader_index = compiler.instructions.len() - 1;
    let (second_loader, _, _) = &mut compiler.instructions.get_mut(second_loader_index).unwrap();
    *second_loader = Instruction::load_boolean(
        first_loader_destination,
        second_loader.b_field() != 0,
        false,
    );
}

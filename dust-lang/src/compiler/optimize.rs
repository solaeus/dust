//! Functions used by the compiler to optimize a chunk's bytecode during compilation.

use crate::{Compiler, Operation};

/// Optimizes a control flow pattern by removing redundant instructions.
///
/// If a comparison instruction is followed by a test instruction, the test instruction may be
/// redundant because the comparison instruction already sets the correct value. If the test's
/// arguments (i.e. the boolean loaders) are `true` and `false` (in that order) then the boolean
/// loaders, jump and test instructions are removed, leaving a single comparison instruction.
///
/// This makes the following two code snippets compile to the same bytecode:
///
/// ```dust
/// 4 == 4
/// ```
///
/// ```dust
/// if 4 == 4 { true } else { false }
/// ```
///
/// The instructions must be in the following order:
///     - `EQUAL`, `LESS` or `LESS_EQUAL`
///     - `TEST`
///     - `JUMP`
///     - `LOAD_BOOLEAN`
///     - `LOAD_BOOLEAN`
pub fn optimize_test_with_explicit_booleans(compiler: &mut Compiler) {
    if matches!(
        compiler.get_last_operations(),
        Some([
            Operation::EQUAL | Operation::LESS | Operation::LESS_EQUAL,
            Operation::TEST,
            Operation::JUMP,
            Operation::LOAD_BOOLEAN,
            Operation::LOAD_BOOLEAN,
        ])
    ) {
        log::debug!("Removing redundant test, jump and boolean loaders after comparison");

        let first_loader = compiler.instructions.iter().nth_back(1).unwrap();
        let second_loader = compiler.instructions.last().unwrap();
        let first_boolean = first_loader.0.b_field() != 0;
        let second_boolean = second_loader.0.b_field() != 0;

        if first_boolean && !second_boolean {
            compiler.instructions.pop();
            compiler.instructions.pop();
            compiler.instructions.pop();
            compiler.instructions.pop();
        }
    }
}

/// Optimizes a control flow pattern.
///
/// TEST instructions (which are always followed by a JUMP) can be optimized when the next
/// instructions are two constant or boolean loaders. The first loader is set to skip an instruction
/// if it is run while the second loader is modified to use the first's register. Foregoing the use
/// a jump instruction is an optimization but consolidating the registers is a necessity. This is
/// because test instructions are essentially control flow and a subsequent SET_LOCAL instruction
/// would not know at compile time which branch would be executed at runtime.
///
/// The instructions must be in the following order:
///     - `TEST`
///     - `JUMP`
///     - `LOAD_BOOLEAN` or `LOAD_CONSTANT`
///     - `LOAD_BOOLEAN` or `LOAD_CONSTANT`
pub fn optimize_test_with_loader_arguments(compiler: &mut Compiler) {
    if !matches!(
        compiler.get_last_operations(),
        Some([
            Operation::TEST,
            Operation::JUMP,
            Operation::LOAD_BOOLEAN | Operation::LOAD_CONSTANT,
            Operation::LOAD_BOOLEAN | Operation::LOAD_CONSTANT,
        ])
    ) {
        return;
    }

    log::debug!("Consolidating registers for control flow optimization");

    let first_loader = &mut compiler.instructions.iter_mut().nth_back(1).unwrap().0;

    first_loader.set_c_field(true as u8);

    let first_loader_destination = first_loader.a_field();
    let second_loader = &mut compiler.instructions.last_mut().unwrap().0;

    second_loader.set_a_field(first_loader_destination);
}

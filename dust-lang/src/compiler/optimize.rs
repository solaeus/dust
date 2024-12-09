//! Tools used by the compiler to optimize a chunk's bytecode.

use crate::{instruction::SetLocal, CompileError, Compiler, Operation};

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
///     - `Equal`, `Less` or `LessEqual`
///     - `Test`
///     - `Jump`
///     - `LoadBoolean`
///     - `LoadBoolean`
pub fn optimize_test_with_explicit_booleans(compiler: &mut Compiler) {
    if matches!(
        compiler.get_last_operations(),
        Some([
            Operation::Equal | Operation::Less | Operation::LessEqual,
            Operation::Test,
            Operation::Jump,
            Operation::LoadBoolean,
            Operation::LoadBoolean,
        ])
    ) {
        log::debug!("Removing redundant test, jump and boolean loaders after comparison");

        let first_loader = compiler.instructions.iter().nth_back(1).unwrap();
        let second_loader = compiler.instructions.last().unwrap();
        let first_boolean = first_loader.0.b != 0;
        let second_boolean = second_loader.0.b != 0;

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
/// Test instructions (which are always followed by a jump) can be optimized when the next
/// instructions are two constant or boolean loaders. The first loader is set to skip an instruction
/// if it is run while the second loader is modified to use the first's register. This foregoes the
/// use of a jump instruction and uses one fewer register.
///
/// The instructions must be in the following order:
///     - `Test`
///     - `Jump`
///     - `LoadBoolean` or `LoadConstant`
///     - `LoadBoolean` or `LoadConstant`
pub fn optimize_test_with_loader_arguments(compiler: &mut Compiler) {
    if !matches!(
        compiler.get_last_operations(),
        Some([
            Operation::Test,
            Operation::Jump,
            Operation::LoadBoolean | Operation::LoadConstant,
            Operation::LoadBoolean | Operation::LoadConstant,
        ])
    ) {
        return;
    }

    log::debug!("Consolidating registers for control flow optimization");

    let first_loader = &mut compiler.instructions.iter_mut().nth_back(1).unwrap().0;

    first_loader.c = true as u8;

    let first_loader_destination = first_loader.a;
    let second_loader = &mut compiler.instructions.last_mut().unwrap().0;

    second_loader.a = first_loader_destination;
}

/// Optimizes a math assignment pattern.
///
/// The SetLocal instruction is removed and the math instruction is modified to use the local as
/// its destination. This makes the following two code snippets compile to the same bytecode:
///
/// ```dust
/// let a = 0;
/// a = a + 1;
/// ```
///
/// ```dust
/// let a = 0;
/// a += 1;
/// ```
///
/// The instructions must be in the following order:
///     - `Add`, `Subtract`, `Multiply`, `Divide` or `Modulo`
///     - `SetLocal`
pub fn condense_set_local_to_math(compiler: &mut Compiler) -> Result<(), CompileError> {
    if !matches!(
        compiler.get_last_operations(),
        Some([
            Operation::Add
                | Operation::Subtract
                | Operation::Multiply
                | Operation::Divide
                | Operation::Modulo,
            Operation::SetLocal,
        ])
    ) {
        return Ok(());
    }

    log::debug!("Condensing math and SetLocal to math instruction");

    let set_local = SetLocal::from(&compiler.instructions.pop().unwrap().0);
    let (local, _) = compiler.get_local(set_local.local_index)?;
    let local_register_index = local.register_index;
    let math_instruction = &mut compiler.instructions.last_mut().unwrap().0;

    math_instruction.a = local_register_index;

    Ok(())
}

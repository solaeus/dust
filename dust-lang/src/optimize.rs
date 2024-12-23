//! Tools used by the compiler to optimize a chunk's bytecode.

use crate::{instruction::SetLocal, Instruction, Operation, Span, Type};

fn get_last_operations<const COUNT: usize>(
    instructions: &[(Instruction, Type, Span)],
) -> Option<[Operation; COUNT]> {
    let mut n_operations = [Operation::Return; COUNT];

    for (nth, operation) in n_operations.iter_mut().rev().zip(
        instructions
            .iter()
            .rev()
            .map(|(instruction, _, _)| instruction.operation()),
    ) {
        *nth = operation;
    }

    Some(n_operations)
}

/// Optimizes a short control flow pattern.
///
/// Comparison and test instructions (which are always followed by a JUMP) can be optimized when
/// the next instructions are two constant or boolean loaders. The first loader is set to skip
/// an instruction if it is run while the second loader is modified to use the first's register.
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
///     - `Equal`, `Less`, `LessEqual` or `Test`
///     - `Jump`
///     - `LoadBoolean` or `LoadConstant`
///     - `LoadBoolean` or `LoadConstant`
pub fn optimize_control_flow(instructions: &mut [(Instruction, Type, Span)]) {
    if !matches!(
        get_last_operations(instructions),
        Some([
            Operation::Equal | Operation::Less | Operation::LessEqual | Operation::Test,
            Operation::Jump,
            Operation::LoadBoolean | Operation::LoadConstant,
            Operation::LoadBoolean | Operation::LoadConstant,
        ])
    ) {
        return;
    }

    log::debug!("Consolidating registers for control flow optimization");

    let first_loader = &mut instructions.iter_mut().nth_back(1).unwrap().0;

    first_loader.set_c_to_boolean(true);

    let first_loader_register = first_loader.a();
    let second_loader = &mut instructions.last_mut().unwrap().0;
    let second_loader_new = *second_loader.clone().set_a(first_loader_register);

    *second_loader = second_loader_new;
}

/// Optimizes a math instruction followed by a SetLocal instruction.
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
pub fn optimize_set_local(instructions: &mut Vec<(Instruction, Type, Span)>) {
    if !matches!(
        get_last_operations(instructions),
        Some([
            Operation::Add
                | Operation::Subtract
                | Operation::Multiply
                | Operation::Divide
                | Operation::Modulo,
            Operation::SetLocal,
        ])
    ) {
        return;
    }

    log::debug!("Condensing math and SetLocal to math instruction");

    let set_local = SetLocal::from(&instructions.pop().unwrap().0);
    let math_instruction = instructions.last_mut().unwrap().0;
    let math_instruction_new = *math_instruction
        .clone()
        .set_a(set_local.local_index)
        .set_a_is_local(true);

    instructions.last_mut().unwrap().0 = math_instruction_new;
}

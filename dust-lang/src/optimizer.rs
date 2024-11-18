//! Tool used by the compiler to optimize a chunk's bytecode.

use crate::{Chunk, Instruction, Operation, Span};

/// An instruction optimizer that mutably borrows instructions from a chunk.
#[derive(Debug)]
pub struct Optimizer<'a> {
    chunk: &'a mut Chunk,
}

impl<'a> Optimizer<'a> {
    /// Creates a new optimizer with a mutable reference to some of a chunk's instructions.
    pub fn new(instructions: &'a mut Chunk) -> Self {
        Self {
            chunk: instructions,
        }
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
    ///     - `Operation::Equal` | `Operation::Less` | `Operation::LessEqual` | `Operation::Test`
    ///     - `Operation::Jump`
    ///     - `Operation::LoadBoolean` | `Operation::LoadConstant`
    ///     - `Operation::LoadBoolean` | `Operation::LoadConstant`
    pub fn optimize_control_flow(&mut self) -> bool {
        if !matches!(
            self.get_operations(),
            Some([
                Operation::Equal | Operation::Less | Operation::LessEqual | Operation::Test,
                Operation::Jump,
                Operation::LoadBoolean | Operation::LoadConstant,
                Operation::LoadBoolean | Operation::LoadConstant,
            ])
        ) {
            return false;
        }

        log::debug!("Consolidating registers for control flow optimization");

        let instructions = self.instructions_mut();
        let first_loader = &mut instructions.iter_mut().nth_back(1).unwrap().0;

        first_loader.set_c_to_boolean(true);

        let first_loader_register = first_loader.a();
        let second_loader = &mut instructions.last_mut().unwrap().0;
        let mut second_loader_new = Instruction::with_operation(second_loader.operation());

        second_loader_new.set_a(first_loader_register);
        second_loader_new.set_b(second_loader.b());
        second_loader_new.set_c(second_loader.c());

        *second_loader = second_loader_new;

        true
    }

    pub fn optimize_set_local(&mut self) -> bool {
        if !matches!(
            self.get_operations(),
            Some([
                Operation::Add
                    | Operation::Subtract
                    | Operation::Multiply
                    | Operation::Divide
                    | Operation::Modulo,
                Operation::SetLocal,
            ])
        ) {
            return false;
        }

        log::debug!("Condensing math and SetLocal to math instruction");

        let instructions = self.instructions_mut();
        let set_local = instructions.pop().unwrap().0;
        let set_local_register = set_local.a();
        let math_instruction = &mut instructions.last_mut().unwrap().0;
        let mut math_instruction_new = Instruction::with_operation(math_instruction.operation());

        math_instruction_new.set_a(set_local_register);
        math_instruction_new.set_b(math_instruction.b());
        math_instruction_new.set_c(math_instruction.c());

        if math_instruction.b_is_constant() {
            math_instruction_new.set_b_is_constant();
        }

        if math_instruction.c_is_constant() {
            math_instruction_new.set_c_is_constant();
        }

        *math_instruction = math_instruction_new;

        true
    }

    fn instructions_mut(&mut self) -> &mut Vec<(Instruction, Span)> {
        self.chunk.instructions_mut()
    }

    fn get_operations<const COUNT: usize>(&self) -> Option<[Operation; COUNT]> {
        if self.chunk.len() < COUNT {
            return None;
        }

        let mut n_operations = [Operation::Return; COUNT];

        for (nth, operation) in n_operations.iter_mut().rev().zip(
            self.chunk
                .instructions()
                .iter()
                .rev()
                .map(|(instruction, _)| instruction.operation()),
        ) {
            *nth = operation;
        }

        Some(n_operations)
    }
}

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

    /// Optimizes a comparison operation.
    ///
    /// Comparison instructions (which are always followed by a JUMP) can be optimized when the
    /// next instructions are two constant or boolean loaders. The first loader is set to skip an
    /// instruction if it is run while the second loader is modified to use the first's register.
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
    ///     - `Operation::Equal | Operation::Less | Operation::LessEqual`
    ///     - `Operation::Jump`
    ///     - `Operation::LoadBoolean | Operation::LoadConstant`
    ///     - `Operation::LoadBoolean | Operation::LoadConstant`
    pub fn optimize_comparison(&mut self) -> bool {
        if !matches!(
            self.get_operations(),
            Some([
                Operation::Equal | Operation::Less | Operation::LessEqual,
                Operation::Jump,
                Operation::LoadBoolean | Operation::LoadConstant,
                Operation::LoadBoolean | Operation::LoadConstant,
            ])
        ) {
            return false;
        }

        log::debug!("Optimizing comparison");

        let instructions = self.instructions_mut();
        let first_loader_register = {
            let first_loader = &mut instructions[2].0;

            first_loader.set_c_to_boolean(true);
            first_loader.a()
        };

        let second_loader = &mut instructions[3].0;
        let mut second_loader_new = Instruction::with_operation(second_loader.operation());

        second_loader_new.set_a(first_loader_register);
        second_loader_new.set_b(second_loader.b());
        second_loader_new.set_c(second_loader.c());
        second_loader_new.set_b_to_boolean(second_loader.b_is_constant());
        second_loader_new.set_c_to_boolean(second_loader.c_is_constant());

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

        self.instructions_mut().pop();

        log::debug!("Optimizing by removing redundant SetLocal");

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

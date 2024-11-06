//! Tools used by the compiler to optimize a chunk's bytecode.
use std::{iter::Map, slice::Iter};

use crate::{Instruction, Operation, Span};

type MapToOperation = fn(&(Instruction, Span)) -> Operation;

type OperationIter<'iter> = Map<Iter<'iter, (Instruction, Span)>, MapToOperation>;

/// Performs optimizations on a subset of instructions.
pub fn optimize(instructions: &mut [(Instruction, Span)]) -> usize {
    Optimizer::new(instructions).optimize()
}

/// An instruction optimizer that mutably borrows instructions from a chunk.
#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Optimizer<'chunk> {
    instructions: &'chunk mut [(Instruction, Span)],
}

impl<'chunk> Optimizer<'chunk> {
    /// Creates a new optimizer with a mutable reference to some of a chunk's instructions.
    pub fn new(instructions: &'chunk mut [(Instruction, Span)]) -> Self {
        Self { instructions }
    }

    /// Potentially mutates the instructions to optimize them.
    pub fn optimize(&mut self) -> usize {
        let mut optimizations = 0;

        if matches!(
            self.get_operations(),
            Some([
                Operation::Equal | Operation::Less | Operation::LessEqual,
                Operation::Jump,
                Operation::LoadBoolean | Operation::LoadConstant,
                Operation::LoadBoolean | Operation::LoadConstant,
            ])
        ) {
            self.optimize_comparison();

            optimizations += 1;
        }

        optimizations
    }

    /// Optimizes a comparison operation.
    ///
    /// The instructions must be in the following order:
    ///     - `Operation::Equal | Operation::Less | Operation::LessEqual`
    ///     - `Operation::Jump`
    ///     - `Operation::LoadBoolean | Operation::LoadConstant`
    ///     - `Operation::LoadBoolean | Operation::LoadConstant`
    fn optimize_comparison(&mut self) {
        log::debug!("Optimizing comparison");

        let first_loader_register = {
            let first_loader = &mut self.instructions[2].0;

            first_loader.set_c_to_boolean(true);
            first_loader.a()
        };

        let second_loader = &mut self.instructions[3].0;
        let mut second_loader_new = Instruction::with_operation(second_loader.operation());

        second_loader_new.set_a(first_loader_register);
        second_loader_new.set_b(second_loader.b());
        second_loader_new.set_c(second_loader.c());
        second_loader_new.set_b_to_boolean(second_loader.b_is_constant());
        second_loader_new.set_c_to_boolean(second_loader.c_is_constant());

        *second_loader = second_loader_new;
    }

    fn operations_iter(&self) -> OperationIter {
        self.instructions
            .iter()
            .map(|(instruction, _)| instruction.operation())
    }

    fn get_operations<const COUNT: usize>(&self) -> Option<[Operation; COUNT]> {
        if self.instructions.len() < COUNT {
            return None;
        }

        let mut n_operations = [Operation::Return; COUNT];

        for (nth, operation) in n_operations.iter_mut().zip(self.operations_iter()) {
            *nth = operation;
        }

        Some(n_operations)
    }
}

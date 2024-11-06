use std::{iter::Map, slice::Iter};

use crate::{Instruction, Operation, Span};

type MapToOperation = fn(&(Instruction, Span)) -> Operation;

type OperationIter<'iter> = Map<Iter<'iter, (Instruction, Span)>, MapToOperation>;

pub fn optimize(instructions: &mut [(Instruction, Span)]) {
    Optimizer::new(instructions).optimize();
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Optimizer<'chunk> {
    instructions: &'chunk mut [(Instruction, Span)],
}

impl<'chunk> Optimizer<'chunk> {
    pub fn new(instructions: &'chunk mut [(Instruction, Span)]) -> Self {
        Self { instructions }
    }

    pub fn set_instructions(&mut self, instructions: &'chunk mut [(Instruction, Span)]) {
        self.instructions = instructions;
    }

    pub fn optimize(&mut self) {
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
        }
    }

    fn optimize_comparison(&mut self) {
        log::trace!("Optimizing comparison");

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

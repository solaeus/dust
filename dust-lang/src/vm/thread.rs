use std::{sync::Arc, thread::current_id};

use crossbeam_channel::Sender;

use crate::{
    instruction::{MemoryKind, Move, OperandType, Operation},
    program::Program,
    vm::{call_frame::CallFrame, error::VmError, register::Register, thread_pool::ThreadMessage},
};

pub struct Thread {
    program: Arc<Program>,
    starting_prototype_id: u16,

    call_stack: Vec<CallFrame>,
    register_stack: Vec<Register>,

    message_sender: Arc<Sender<ThreadMessage>>,
}

impl Thread {
    pub fn new(
        program: Arc<Program>,
        prototype_id: u16,
        message_sender: Arc<Sender<ThreadMessage>>,
    ) -> Self {
        let call_stack_capacity = if program.prototypes.len() == 1 {
            0
        } else {
            256
        };
        let register_count = if program.prototypes.len() == 1 {
            program.prototypes[0].register_count as usize
        } else {
            1024
        };

        Thread {
            program,
            starting_prototype_id: prototype_id,
            call_stack: Vec::with_capacity(call_stack_capacity),
            register_stack: vec![Register(0); register_count],
            message_sender,
        }
    }

    pub fn run(mut self) {
        let result = self.run_inner();

        self.message_sender
            .send(ThreadMessage::RemoveThread {
                thread_id: current_id(),
                result,
            })
            .expect("Failed to send thread finished message");
    }

    pub fn run_inner(&mut self) -> Result<Vec<Register>, VmError> {
        let starting_prototype = self
            .program
            .prototypes
            .as_slice()
            .get(self.starting_prototype_id as usize)
            .ok_or(VmError::InvalidPrototypeId {
                prototype_id: self.starting_prototype_id,
            })?;
        let starting_call_frame = CallFrame {
            prototype_id: self.starting_prototype_id,
            regsiter_range_start: 0,
            register_range_end: starting_prototype.register_count,
            argument_count: starting_prototype.argument_count,
            return_count: starting_prototype.return_types.len() as u16,
            instruction_pointer: 0,
        };

        self.call_stack.push(starting_call_frame);

        'thread: loop {
            let current_call_frame = self.call_stack.pop().ok_or(VmError::CallStackUnderflow)?;
            let current_prototype = self
                .program
                .prototypes
                .as_slice()
                .get(current_call_frame.prototype_id as usize)
                .ok_or(VmError::InvalidPrototypeId {
                    prototype_id: current_call_frame.prototype_id,
                })?;
            let mut instruction_pointer = current_call_frame.instruction_pointer;

            'call: loop {
                assert!(
                    instruction_pointer < current_prototype.instructions.len(),
                    "Unrecoverable error: Instruction pointer out of bounds"
                );

                let instruction = &current_prototype.instructions[instruction_pointer];
                let operation = instruction.operation();

                match operation {
                    Operation::MOVE => {
                        let Move {
                            destination,
                            operand_type,
                            operand_memory,
                            operand_index,
                            jump_distance,
                            jump_forward,
                        } = Move::from(instruction);

                        match operand_type {
                            OperandType::I_32 => {
                                let register_value = match operand_memory {
                                    MemoryKind::CONSTANT => {
                                        self.program.constants.get_i32(operand_index)? as u32
                                    }
                                    MemoryKind::REGISTER => self.get_register(operand_index)?.0,
                                    _ => {
                                        return Err(VmError::UnsupportedMemoryKind {
                                            memory: operand_memory,
                                        });
                                    }
                                };

                                self.register_stack[destination as usize] =
                                    Register(register_value);
                            }
                            OperandType::F_64 => {
                                let (low_bits, high_bits) = match operand_memory {
                                    MemoryKind::CONSTANT => {
                                        let bits = self
                                            .program
                                            .constants
                                            .get_f64(operand_index)?
                                            .to_bits();

                                        (bits as u32, (bits >> 32) as u32)
                                    }
                                    MemoryKind::REGISTER => {
                                        let register_value = self.get_register(operand_index)?.0;
                                        let next_register_value =
                                            self.get_register(operand_index + 1)?.0;

                                        (register_value, next_register_value)
                                    }
                                    _ => {
                                        return Err(VmError::UnsupportedMemoryKind {
                                            memory: operand_memory,
                                        });
                                    }
                                };
                                let destination = destination as usize;
                                let next_destination = destination + 1;

                                self.register_stack[destination] = Register(low_bits);
                                self.register_stack[next_destination] = Register(high_bits);
                            }
                            _ => {
                                return Err(VmError::UnsupportedOperandType { operand_type });
                            }
                        }

                        if jump_distance > 0 {
                            if jump_forward {
                                instruction_pointer += jump_distance as usize;
                            } else {
                                instruction_pointer -= jump_distance as usize;
                            }
                        } else {
                            instruction_pointer += 1;
                        }
                    }
                    Operation::RETURN => {
                        if self.call_stack.is_empty() {
                            let return_register_range = current_call_frame.regsiter_range_start
                                as usize
                                ..current_call_frame.register_range_end as usize;
                            let return_registers =
                                self.register_stack[return_register_range].to_vec();

                            break 'thread Ok(return_registers);
                        } else {
                            break 'call;
                        }
                    }
                    _ => {
                        return Err(VmError::UnsupportedOperation { operation });
                    }
                }
            }
        }
    }

    fn get_register(&self, index: u16) -> Result<&Register, VmError> {
        self.register_stack
            .get(index as usize)
            .ok_or(VmError::InvalidRegisterIndex { index })
    }
}

use std::{sync::Arc, thread::JoinHandle};

use tracing::{info, warn};

use crate::{
    Chunk, ConcreteValue, DustString, Operation,
    instruction::{Add, AddressKind, Jump, Less, LoadConstant, Move, Return},
};

use super::{CallFrame, Memory, RegisterTable};

pub struct Thread {
    chunk: Arc<Chunk>,

    call_stack: Vec<CallFrame>,
    memory_stack: Vec<Memory>,

    _spawned_threads: Vec<JoinHandle<()>>,
}

impl Thread {
    pub fn new(chunk: Arc<Chunk>) -> Self {
        let mut call_stack = Vec::with_capacity(chunk.prototypes.len() + 1);
        let main_call = CallFrame::new(Arc::clone(&chunk), 0);

        call_stack.push(main_call);

        let mut memory_stack = Vec::with_capacity(chunk.prototypes.len() + 1);
        let main_memory = Memory {
            booleans: Vec::with_capacity(chunk.boolean_memory_length as usize),
            bytes: Vec::with_capacity(chunk.byte_memory_length as usize),
            characters: Vec::with_capacity(chunk.character_memory_length as usize),
            floats: Vec::with_capacity(chunk.float_memory_length as usize),
            integers: Vec::with_capacity(chunk.integer_memory_length as usize),
            strings: Vec::with_capacity(chunk.string_memory_length as usize),
            lists: Vec::with_capacity(chunk.list_memory_length as usize),
            functions: Vec::with_capacity(chunk.function_memory_length as usize),
            registers: RegisterTable::default(),
        };

        memory_stack.push(main_memory);

        Thread {
            chunk,
            call_stack,
            memory_stack,
            _spawned_threads: Vec::new(),
        }
    }

    pub fn run(mut self) -> Option<ConcreteValue> {
        info!(
            "Starting thread {}",
            self.chunk
                .name
                .clone()
                .unwrap_or_else(|| DustString::from("anonymous"))
        );

        let mut current_call = self.call_stack.pop().unwrap();
        let mut current_memory = self.memory_stack.pop().unwrap();
        let mut r#return = None;

        loop {
            let instructions = &current_call.chunk.instructions;
            let ip = current_call.ip;
            current_call.ip += 1;

            assert!(ip < instructions.len(), "IP out of bounds");

            let instruction = instructions[ip];
            let operation = instruction.operation();

            info!("IP = {ip} Run {operation}");

            match operation {
                Operation::NO_OP => {
                    warn!("Running NO_OP instruction");
                }
                Operation::MOVE => {
                    let Move {
                        destination: to,
                        operand: from,
                    } = Move::from(&instruction);

                    match from.kind {
                        AddressKind::BOOLEAN_MEMORY => {
                            let boolean = *current_memory
                                .booleans
                                .get(from.index as usize)
                                .unwrap()
                                .as_value();

                            *current_memory
                                .booleans
                                .get_mut(to.index as usize)
                                .unwrap()
                                .as_value_mut() = boolean;
                        }
                        AddressKind::BOOLEAN_REGISTER => {
                            let boolean = *current_memory.registers.booleans.get(from.index);

                            current_memory.registers.booleans.set(to.index, boolean);
                        }
                        _ => unimplemented!(),
                    }
                }
                Operation::CLOSE => todo!(),
                Operation::LOAD_ENCODED => todo!(),
                Operation::LOAD_CONSTANT => {
                    let LoadConstant {
                        destination,
                        constant,
                        jump_next,
                    } = LoadConstant::from(&instruction);
                    let constant_index = constant.index as usize;

                    match constant.kind {
                        AddressKind::CHARACTER_CONSTANT => {
                            let value = self.chunk.character_constants[constant_index];

                            if destination.is_register {
                                current_memory
                                    .registers
                                    .characters
                                    .set(destination.index, value);
                            } else {
                                let destination_index = destination.index as usize;

                                *current_memory.characters[destination_index].as_value_mut() =
                                    value;
                            }
                        }
                        AddressKind::FLOAT_CONSTANT => {
                            let value = self.chunk.float_constants[constant_index];

                            if destination.is_register {
                                current_memory
                                    .registers
                                    .floats
                                    .set(destination.index, value);
                            } else {
                                let destination_index = destination.index as usize;

                                *current_memory.floats[destination_index].as_value_mut() = value;
                            }
                        }
                        AddressKind::INTEGER_CONSTANT => {
                            let value = self.chunk.integer_constants[constant_index];

                            if destination.is_register {
                                current_memory
                                    .registers
                                    .integers
                                    .set(destination.index, value);
                            } else {
                                let destination_index = destination.index as usize;

                                *current_memory.integers[destination_index].as_value_mut() = value;
                            }
                        }
                        AddressKind::STRING_CONSTANT => {
                            let value = self.chunk.string_constants[constant_index].clone();

                            if destination.is_register {
                                current_memory
                                    .registers
                                    .strings
                                    .set(destination.index, value);
                            } else {
                                let destination_index = destination.index as usize;

                                *current_memory.strings[destination_index].as_value_mut() = value;
                            }
                        }
                        _ => unreachable!(),
                    };

                    if jump_next {
                        current_call.ip += 1;
                    }
                }
                Operation::LOAD_FUNCTION => todo!(),
                Operation::LOAD_LIST => todo!(),
                Operation::ADD => {
                    let Add {
                        destination,
                        left,
                        right,
                    } = Add::from(&instruction);

                    match left.kind {
                        AddressKind::INTEGER_CONSTANT => {
                            let left_value = self.chunk.integer_constants[left.index as usize];
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    &self.chunk.integer_constants[right.index as usize]
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    current_memory.integers[right.index as usize].as_value()
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    current_memory.registers.integers.get(right.index)
                                }
                                _ => unreachable!(),
                            };
                            let sum = left_value + right_value;

                            if destination.is_register {
                                current_memory
                                    .registers
                                    .integers
                                    .set(destination.index, sum);
                            } else {
                                *current_memory.integers[destination.index as usize]
                                    .as_value_mut() = sum;
                            }
                        }
                        AddressKind::INTEGER_MEMORY => {
                            let left_value =
                                current_memory.integers[left.index as usize].as_value();
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    &self.chunk.integer_constants[right.index as usize]
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    current_memory.integers[right.index as usize].as_value()
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    current_memory.registers.integers.get(right.index)
                                }
                                _ => unreachable!(),
                            };
                            let sum = left_value + right_value;

                            if destination.is_register {
                                current_memory
                                    .registers
                                    .integers
                                    .set(destination.index, sum);
                            } else {
                                *current_memory.integers[destination.index as usize]
                                    .as_value_mut() = sum;
                            }
                        }
                        AddressKind::INTEGER_REGISTER => {
                            let left_value = current_memory.registers.integers.get(left.index);
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    &self.chunk.integer_constants[right.index as usize]
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    current_memory.integers[right.index as usize].as_value()
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    current_memory.registers.integers.get(right.index)
                                }
                                _ => unreachable!(),
                            };
                            let sum = left_value + right_value;

                            if destination.is_register {
                                current_memory
                                    .registers
                                    .integers
                                    .set(destination.index, sum);
                            } else {
                                *current_memory.integers[destination.index as usize]
                                    .as_value_mut() = sum;
                            }
                        }
                        _ => todo!(),
                    }
                }
                Operation::SUBTRACT => todo!(),
                Operation::MULTIPLY => todo!(),
                Operation::DIVIDE => todo!(),
                Operation::MODULO => todo!(),
                Operation::EQUAL => todo!(),
                Operation::LESS => {
                    let Less {
                        comparator,
                        left,
                        right,
                    } = Less::from(&instruction);

                    let is_less_than = match left.kind {
                        AddressKind::INTEGER_MEMORY => {
                            let left_value =
                                current_memory.integers[left.index as usize].as_value();
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    &self.chunk.integer_constants[right.index as usize]
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    current_memory.integers[right.index as usize].as_value()
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    current_memory.registers.integers.get(right.index)
                                }
                                _ => unreachable!(),
                            };

                            left_value < right_value
                        }
                        AddressKind::INTEGER_REGISTER => {
                            let left_value = current_memory.registers.integers.get(left.index);
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    &self.chunk.integer_constants[right.index as usize]
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    current_memory.integers[right.index as usize].as_value()
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    current_memory.registers.integers.get(right.index)
                                }
                                _ => unreachable!(),
                            };

                            left_value < right_value
                        }
                        _ => todo!(),
                    };

                    if is_less_than != comparator {
                        current_call.ip += 1;
                    }
                }
                Operation::LESS_EQUAL => todo!(),
                Operation::NEGATE => todo!(),
                Operation::NOT => todo!(),
                Operation::TEST => todo!(),
                Operation::TEST_SET => todo!(),
                Operation::CALL => todo!(),
                Operation::CALL_NATIVE => todo!(),
                Operation::JUMP => {
                    let Jump {
                        offset,
                        is_positive,
                    } = Jump::from(&instruction);

                    if is_positive {
                        current_call.ip += offset as usize;
                    } else {
                        current_call.ip -= (offset + 1) as usize;
                    }
                }
                Operation::RETURN => {
                    let Return {
                        should_return_value,
                        return_address,
                    } = Return::from(&instruction);

                    if should_return_value {
                        match return_address.kind {
                            AddressKind::INTEGER_REGISTER => {
                                let integer =
                                    *current_memory.registers.integers.get(return_address.index);

                                r#return = Some(Some(ConcreteValue::Integer(integer)));
                            }
                            _ => todo!(),
                        };
                    } else {
                        r#return = Some(None);
                    }
                }
                _ => unreachable!(),
            }

            if let Some(return_option) = r#return {
                return return_option;
            }
        }
    }
}

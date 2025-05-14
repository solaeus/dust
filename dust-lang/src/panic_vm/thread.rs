use std::{mem::replace, sync::Arc, thread::JoinHandle};

use tracing::{Level, info, span, warn};

use crate::{
    Chunk, ConcreteValue, DustString, Operation,
    instruction::{Add, AddressKind, Call, Jump, Less, LoadConstant, LoadFunction, Move, Return},
};

use super::{CallFrame, Memory};

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
        let main_memory = Memory::new(&chunk);

        memory_stack.push(main_memory);

        Thread {
            chunk,
            call_stack,
            memory_stack,
            _spawned_threads: Vec::new(),
        }
    }

    pub fn run(&mut self) -> Option<ConcreteValue> {
        let span = span!(Level::INFO, "Thread");
        let _enter = span.enter();

        info!(
            "Starting thread {}",
            self.chunk
                .name
                .clone()
                .unwrap_or_else(|| DustString::from("anonymous"))
        );

        let mut call = self.call_stack.pop().unwrap();
        let mut memory = self.memory_stack.pop().unwrap();
        let mut r#return = None;

        loop {
            let instructions = &call.chunk.instructions;
            let ip = call.ip;
            call.ip += 1;

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
                            let boolean =
                                *memory.booleans.get(from.index as usize).unwrap().as_value();

                            *memory
                                .booleans
                                .get_mut(to.index as usize)
                                .unwrap()
                                .as_value_mut() = boolean;
                        }
                        AddressKind::BOOLEAN_REGISTER => {
                            let boolean = memory.registers.booleans[from.index as usize];

                            memory.registers.booleans[to.index as usize] = boolean;
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
                                memory.registers.characters[destination.index as usize] = value;
                            } else {
                                let destination_index = destination.index as usize;

                                *memory.characters[destination_index].as_value_mut() = value;
                            }
                        }
                        AddressKind::FLOAT_CONSTANT => {
                            let value = self.chunk.float_constants[constant_index];

                            if destination.is_register {
                                memory.registers.floats[destination.index as usize] = value;
                            } else {
                                let destination_index = destination.index as usize;

                                *memory.floats[destination_index].as_value_mut() = value;
                            }
                        }
                        AddressKind::INTEGER_CONSTANT => {
                            let value = self.chunk.integer_constants[constant_index];

                            if destination.is_register {
                                memory.registers.integers[destination.index as usize] = value;
                            } else {
                                let destination_index = destination.index as usize;

                                *memory.integers[destination_index].as_value_mut() = value;
                            }
                        }
                        AddressKind::STRING_CONSTANT => {
                            let value = self.chunk.string_constants[constant_index].clone();

                            if destination.is_register {
                                memory.registers.strings[destination.index as usize] = value;
                            } else {
                                let destination_index = destination.index as usize;

                                *memory.strings[destination_index].as_value_mut() = value;
                            }
                        }
                        _ => unreachable!(),
                    };

                    if jump_next {
                        call.ip += 1;
                    }
                }
                Operation::LOAD_FUNCTION => {
                    let LoadFunction {
                        destination,
                        prototype: prototype_address,
                        jump_next,
                    } = LoadFunction::from(&instruction);

                    let function = match prototype_address.kind {
                        AddressKind::FUNCTION_PROTOTYPE => {
                            self.chunk.prototypes[prototype_address.index as usize].as_function()
                        }
                        AddressKind::FUNCTION_SELF => self.chunk.as_function(),
                        _ => unreachable!(),
                    };

                    if destination.is_register {
                        memory.registers.functions[destination.index as usize] = function;
                    }

                    if jump_next {
                        call.ip += 1;
                    }
                }
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
                                    self.chunk.integer_constants[right.index as usize]
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    *memory.integers[right.index as usize].as_value()
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    memory.registers.integers[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let sum = left_value + right_value;

                            if destination.is_register {
                                memory.registers.integers[destination.index as usize] = sum;
                            } else {
                                *memory.integers[destination.index as usize].as_value_mut() = sum;
                            }
                        }
                        AddressKind::INTEGER_MEMORY => {
                            let left_value = memory.integers[left.index as usize].as_value();
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    self.chunk.integer_constants[right.index as usize]
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    *memory.integers[right.index as usize].as_value()
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    memory.registers.integers[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let sum = left_value + right_value;

                            if destination.is_register {
                                memory.registers.integers[destination.index as usize] = sum;
                            } else {
                                *memory.integers[destination.index as usize].as_value_mut() = sum;
                            }
                        }
                        AddressKind::INTEGER_REGISTER => {
                            let left_value = memory.registers.integers[left.index as usize];
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    self.chunk.integer_constants[right.index as usize]
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    *memory.integers[right.index as usize].as_value()
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    memory.registers.integers[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let sum = left_value + right_value;

                            if destination.is_register {
                                memory.registers.integers[destination.index as usize] = sum;
                            } else {
                                *memory.integers[destination.index as usize].as_value_mut() = sum;
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
                            let left_value = *memory.integers[left.index as usize].as_value();
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    self.chunk.integer_constants[right.index as usize]
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    *memory.integers[right.index as usize].as_value()
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    memory.registers.integers[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value < right_value
                        }
                        AddressKind::INTEGER_REGISTER => {
                            let left_value = memory.registers.integers[left.index as usize];
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    self.chunk.integer_constants[right.index as usize]
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    *memory.integers[right.index as usize].as_value()
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    memory.registers.integers[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value < right_value
                        }
                        _ => todo!(),
                    };

                    if is_less_than != comparator {
                        call.ip += 1;
                    }
                }
                Operation::LESS_EQUAL => todo!(),
                Operation::NEGATE => todo!(),
                Operation::NOT => todo!(),
                Operation::TEST => todo!(),
                Operation::TEST_SET => todo!(),
                Operation::CALL => {
                    let Call {
                        destination,
                        function: function_address,
                        argument_list_index,
                        return_type,
                    } = Call::from(&instruction);
                    let prototype_address = match function_address.kind {
                        AddressKind::FUNCTION_REGISTER => {
                            memory.registers.functions[function_address.index as usize]
                                .prototype_address
                        }
                        _ => unreachable!(),
                    };
                    let function = match prototype_address.kind {
                        AddressKind::FUNCTION_PROTOTYPE => {
                            &self.chunk.prototypes[prototype_address.index as usize]
                        }
                        AddressKind::FUNCTION_SELF => &self.chunk,
                        _ => unreachable!(),
                    };
                    let new_call = CallFrame::new(Arc::clone(function), destination.index);
                    let new_memory = Memory::new(function);

                    self.memory_stack.push(replace(&mut memory, new_memory));
                    self.call_stack.push(replace(&mut call, new_call));
                }
                Operation::CALL_NATIVE => todo!(),
                Operation::JUMP => {
                    let Jump {
                        offset,
                        is_positive,
                    } = Jump::from(&instruction);

                    if is_positive {
                        call.ip += offset as usize;
                    } else {
                        call.ip -= (offset + 1) as usize;
                    }
                }
                Operation::RETURN => {
                    let Return {
                        should_return_value,
                        return_address,
                    } = Return::from(&instruction);

                    let return_option = if should_return_value {
                        match return_address.kind {
                            AddressKind::INTEGER_REGISTER => {
                                let integer =
                                    memory.registers.integers[return_address.index as usize];

                                Some(ConcreteValue::Integer(integer))
                            }
                            AddressKind::FUNCTION_REGISTER => {
                                let prototype_address = memory.registers.functions
                                    [return_address.index as usize]
                                    .prototype_address;
                                let prototype = match prototype_address.kind {
                                    AddressKind::FUNCTION_PROTOTYPE => {
                                        &self.chunk.prototypes[prototype_address.index as usize]
                                    }
                                    AddressKind::FUNCTION_SELF => &self.chunk,
                                    _ => unreachable!(),
                                };

                                Some(ConcreteValue::Function(Arc::clone(prototype)))
                            }
                            _ => todo!(),
                        }
                    } else {
                        None
                    };

                    if self.call_stack.is_empty() {
                        r#return = Some(return_option);
                    } else {
                        call = self.call_stack.pop().unwrap();
                        memory = self.memory_stack.pop().unwrap();
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

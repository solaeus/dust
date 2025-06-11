use std::{
    mem::replace,
    sync::{Arc, Mutex},
    thread::{Builder, JoinHandle},
};

use tracing::{Level, info, span, warn};

use crate::{
    AbstractList, Address, Chunk, ConcreteValue, Destination, DustString, Operation,
    instruction::{
        Add, AddressKind, Call, CallNative, Close, Divide, Equal, Jump, Less, LessEqual,
        LoadConstant, LoadEncoded, LoadFunction, LoadList, Modulo, Move, Multiply, Negate, Not,
        Return, Subtract, Test,
    },
};

use super::{CallFrame, Memory, macros::*};

pub struct Thread<const REGISTER_COUNT: usize> {
    pub handle: JoinHandle<Option<ConcreteValue>>,
}

impl<const REGISTER_COUNT: usize> Thread<REGISTER_COUNT> {
    pub fn new(chunk: Arc<Chunk>, threads: Arc<Mutex<Vec<Thread<REGISTER_COUNT>>>>) -> Self {
        let mut runner = ThreadRunner {
            chunk: Arc::clone(&chunk),
            threads,
        };

        let handle = Builder::new()
            .name(
                chunk
                    .name
                    .as_ref()
                    .map(|name| name.to_string())
                    .unwrap_or_else(|| "anonymous".to_string()),
            )
            .spawn(move || runner.run())
            .expect("Failed to spawn thread");

        Thread { handle }
    }
}

#[derive(Clone)]
struct ThreadRunner<const REGISTER_COUNT: usize> {
    chunk: Arc<Chunk>,
    threads: Arc<Mutex<Vec<Thread<REGISTER_COUNT>>>>,
}

impl<const REGISTER_COUNT: usize> ThreadRunner<REGISTER_COUNT> {
    fn run(&mut self) -> Option<ConcreteValue> {
        let span = span!(Level::INFO, "Thread");
        let _enter = span.enter();

        info!(
            "Starting thread {}",
            self.chunk
                .name
                .as_ref()
                .map(|name| name.as_str())
                .unwrap_or_default()
        );

        let mut call_stack = Vec::new();
        let mut memory_stack = Vec::new();

        let mut call = CallFrame::new(Arc::clone(&self.chunk), Destination::memory(0));
        let mut memory = Memory::<REGISTER_COUNT>::new(&call.chunk);

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
                            let boolean = get_memory!(memory, booleans, from);

                            set!(memory, booleans, to, *boolean);
                        }
                        AddressKind::BOOLEAN_REGISTER => {
                            let boolean = get_register!(memory, booleans, from);

                            set!(memory, booleans, to, *boolean);
                        }
                        AddressKind::BYTE_MEMORY => {
                            let byte = get_memory!(memory, bytes, from);

                            set!(memory, bytes, to, *byte);
                        }
                        AddressKind::BYTE_REGISTER => {
                            let byte = get_register!(memory, bytes, from);

                            set!(memory, bytes, to, *byte);
                        }
                        AddressKind::CHARACTER_MEMORY => {
                            let character = get_memory!(memory, characters, from);

                            set!(memory, characters, to, *character);
                        }
                        AddressKind::CHARACTER_REGISTER => {
                            let character = get_register!(memory, characters, from);

                            set!(memory, characters, to, *character);
                        }
                        AddressKind::FLOAT_MEMORY => {
                            let float = get_memory!(memory, floats, from);

                            set!(memory, floats, to, *float);
                        }
                        AddressKind::FLOAT_REGISTER => {
                            let float = get_register!(memory, floats, from);

                            set!(memory, floats, to, *float);
                        }
                        AddressKind::INTEGER_MEMORY => {
                            let integer = get_memory!(memory, integers, from);

                            set!(memory, integers, to, *integer);
                        }
                        AddressKind::INTEGER_REGISTER => {
                            let integer = get_register!(memory, integers, from);

                            set!(memory, integers, to, *integer);
                        }
                        AddressKind::STRING_MEMORY => {
                            let string = get_memory!(memory, strings, from);

                            set!(memory, strings, to, string.clone());
                        }
                        AddressKind::STRING_REGISTER => {
                            let string = get_register!(memory, strings, from);

                            set!(memory, strings, to, string.clone());
                        }
                        AddressKind::LIST_MEMORY => {
                            let abstract_list = get_memory!(memory, lists, from);

                            set!(memory, lists, to, abstract_list.clone());
                        }
                        AddressKind::LIST_REGISTER => {
                            let abstract_list = get_register!(memory, lists, from);

                            set!(memory, lists, to, abstract_list.clone());
                        }
                        AddressKind::FUNCTION_PROTOTYPE => {
                            let function = get_constant!(call.chunk, prototypes, from);

                            set!(memory, functions, to, Arc::clone(function));
                        }
                        AddressKind::FUNCTION_SELF => {
                            set!(memory, functions, to, Arc::clone(&call.chunk));
                        }
                        AddressKind::FUNCTION_REGISTER => {
                            let function = get_register!(memory, functions, from);

                            set!(memory, functions, to, Arc::clone(function));
                        }
                        AddressKind::FUNCTION_MEMORY => {
                            let function = get_memory!(memory, functions, from);

                            set!(memory, functions, to, Arc::clone(function));
                        }

                        _ => malformed_instruction!(instruction, ip),
                    }
                }
                Operation::CLOSE => {
                    let Close { from, to } = Close::from(&instruction);

                    for i in from.index..=to.index {
                        let address = Address::new(i, from.kind);

                        memory.closed.insert(address);
                    }
                }
                Operation::LOAD_ENCODED => {
                    let LoadEncoded {
                        destination,
                        value,
                        r#type,
                        jump_next,
                    } = LoadEncoded::from(&instruction);

                    match r#type {
                        AddressKind::BOOLEAN_MEMORY => {
                            let boolean = value != 0;
                            if destination.is_register {
                                memory.registers.booleans[destination.index as usize] = boolean;
                            } else {
                                memory.booleans[destination.index as usize] = boolean;
                            }
                        }
                        AddressKind::BYTE_MEMORY => {
                            let byte = value as u8;
                            if destination.is_register {
                                memory.registers.bytes[destination.index as usize] = byte;
                            } else {
                                memory.bytes[destination.index as usize] = byte;
                            }
                        }
                        _ => malformed_instruction!(instruction, ip),
                    }

                    if jump_next {
                        call.ip += 1;
                    }
                }
                Operation::LOAD_CONSTANT => {
                    let LoadConstant {
                        destination,
                        constant,
                        jump_next,
                    } = LoadConstant::from(&instruction);
                    let constant_index = constant.index as usize;

                    match constant.kind {
                        AddressKind::CHARACTER_CONSTANT => {
                            let value = call.chunk.character_constants[constant_index];

                            if destination.is_register {
                                memory.registers.characters[destination.index as usize] = value;
                            } else {
                                let destination_index = destination.index as usize;

                                memory.characters[destination_index] = value;
                            }
                        }
                        AddressKind::FLOAT_CONSTANT => {
                            let value = call.chunk.float_constants[constant_index];

                            if destination.is_register {
                                memory.registers.floats[destination.index as usize] = value;
                            } else {
                                let destination_index = destination.index as usize;

                                memory.floats[destination_index] = value;
                            }
                        }
                        AddressKind::INTEGER_CONSTANT => {
                            let value = call.chunk.integer_constants[constant_index];

                            if destination.is_register {
                                memory.registers.integers[destination.index as usize] = value;
                            } else {
                                let destination_index = destination.index as usize;

                                memory.integers[destination_index] = value;
                            }
                        }
                        AddressKind::STRING_CONSTANT => {
                            let value = call.chunk.string_constants[constant_index].clone();

                            if destination.is_register {
                                memory.registers.strings[destination.index as usize] = value;
                            } else {
                                let destination_index = destination.index as usize;

                                memory.strings[destination_index] = value;
                            }
                        }
                        _ => malformed_instruction!(instruction, ip),
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
                            let chunk = &call.chunk.prototypes[prototype_address.index as usize];

                            Arc::clone(chunk)
                        }
                        AddressKind::FUNCTION_SELF => Arc::clone(&call.chunk),
                        _ => malformed_instruction!(instruction, ip),
                    };

                    if destination.is_register {
                        memory.registers.functions[destination.index as usize] = function;
                    }

                    if jump_next {
                        call.ip += 1;
                    }
                }
                Operation::LOAD_LIST => {
                    let LoadList {
                        destination,
                        start,
                        end,
                        jump_next,
                    } = LoadList::from(&instruction);
                    let mut abstract_list = AbstractList {
                        pointer_kind: start.kind,
                        indices: Vec::with_capacity((end - start.index + 1) as usize),
                    };

                    for i in start.index..=end {
                        let pointer = Address::new(i, start.kind);

                        if memory.closed.contains(&pointer) {
                            continue;
                        }

                        abstract_list.indices.push(i);
                    }

                    if destination.is_register {
                        memory.registers.lists[destination.index as usize] = abstract_list;
                    } else {
                        memory.lists[destination.index as usize] = abstract_list;
                    }

                    if jump_next {
                        call.ip += 1;
                    }
                }
                Operation::ADD => {
                    let Add {
                        destination,
                        left,
                        right,
                    } = Add::from(&instruction);

                    match left.kind {
                        AddressKind::BYTE_MEMORY => {
                            let left_value = get_memory!(memory, bytes, left);
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => get_memory!(memory, bytes, right),
                                AddressKind::BYTE_REGISTER => get_register!(memory, bytes, right),
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let sum = left_value + right_value;

                            set!(memory, bytes, destination, sum);
                        }
                        AddressKind::BYTE_REGISTER => {
                            let left_value = get_register!(memory, bytes, left);
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => get_memory!(memory, bytes, right),
                                AddressKind::BYTE_REGISTER => get_register!(memory, bytes, right),
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let sum = left_value + right_value;

                            set!(memory, bytes, destination, sum);
                        }
                        AddressKind::CHARACTER_CONSTANT => {
                            let left_value = get_constant!(call.chunk, character_constants, left);
                            let right_value = match right.kind {
                                AddressKind::CHARACTER_CONSTANT => {
                                    get_constant!(call.chunk, character_constants, right)
                                }
                                AddressKind::CHARACTER_MEMORY => {
                                    get_memory!(memory, characters, right)
                                }
                                AddressKind::CHARACTER_REGISTER => {
                                    get_register!(memory, characters, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let mut sum = DustString::new();

                            sum.push(*left_value);
                            sum.push(*right_value);

                            set!(memory, strings, destination, sum);
                        }
                        AddressKind::CHARACTER_MEMORY => {
                            let left_value = get_memory!(memory, characters, left);
                            let right_value = match right.kind {
                                AddressKind::CHARACTER_CONSTANT => {
                                    get_constant!(call.chunk, character_constants, right)
                                }
                                AddressKind::CHARACTER_MEMORY => {
                                    get_memory!(memory, characters, right)
                                }
                                AddressKind::CHARACTER_REGISTER => {
                                    get_register!(memory, characters, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let mut sum = DustString::new();

                            sum.push(*left_value);
                            sum.push(*right_value);

                            set!(memory, strings, destination, sum);
                        }
                        AddressKind::CHARACTER_REGISTER => {
                            let left_value = get_register!(memory, characters, left);
                            let right_value = match right.kind {
                                AddressKind::CHARACTER_CONSTANT => {
                                    get_constant!(call.chunk, character_constants, right)
                                }
                                AddressKind::CHARACTER_MEMORY => {
                                    get_memory!(memory, characters, right)
                                }
                                AddressKind::CHARACTER_REGISTER => {
                                    get_register!(memory, characters, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let mut sum = DustString::new();

                            sum.push(*left_value);
                            sum.push(*right_value);

                            set!(memory, strings, destination, sum);
                        }
                        AddressKind::FLOAT_CONSTANT => {
                            let left_value = get_constant!(call.chunk, float_constants, left);
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    get_constant!(call.chunk, float_constants, right)
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    get_memory!(memory, floats, right)
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    get_register!(memory, floats, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let sum = left_value + right_value;

                            set!(memory, floats, destination, sum);
                        }
                        AddressKind::FLOAT_MEMORY => {
                            let left_value = get_memory!(memory, floats, left);
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    get_constant!(call.chunk, float_constants, right)
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    get_memory!(memory, floats, right)
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    get_register!(memory, floats, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let sum = left_value + right_value;

                            set!(memory, floats, destination, sum);
                        }
                        AddressKind::FLOAT_REGISTER => {
                            let left_value = get_register!(memory, floats, left);
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    get_constant!(call.chunk, float_constants, right)
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    get_memory!(memory, floats, right)
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    get_register!(memory, floats, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let sum = left_value + right_value;

                            set!(memory, floats, destination, sum);
                        }
                        AddressKind::INTEGER_CONSTANT => {
                            let left_value = get_constant!(call.chunk, integer_constants, left);
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    get_constant!(call.chunk, integer_constants, right)
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    get_memory!(memory, integers, right)
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    get_register!(memory, integers, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let sum = left_value + right_value;

                            set!(memory, integers, destination, sum);
                        }
                        AddressKind::INTEGER_MEMORY => {
                            let left_value = get_memory!(memory, integers, left);
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    get_constant!(call.chunk, integer_constants, right)
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    get_memory!(memory, integers, right)
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    get_register!(memory, integers, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let sum = left_value + right_value;

                            set!(memory, integers, destination, sum);
                        }
                        AddressKind::INTEGER_REGISTER => {
                            let left_value = get_register!(memory, integers, left);
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    get_constant!(call.chunk, integer_constants, right)
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    get_memory!(memory, integers, right)
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    get_register!(memory, integers, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let sum = left_value + right_value;

                            set!(memory, integers, destination, sum);
                        }
                        AddressKind::STRING_CONSTANT => {
                            let left_value = get_constant!(call.chunk, string_constants, left);
                            let right_value = match right.kind {
                                AddressKind::STRING_CONSTANT => {
                                    get_constant!(call.chunk, string_constants, right)
                                }
                                AddressKind::STRING_MEMORY => {
                                    get_memory!(memory, strings, right)
                                }
                                AddressKind::STRING_REGISTER => {
                                    get_register!(memory, strings, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let mut sum = DustString::new();

                            sum.push_str(left_value);
                            sum.push_str(right_value);

                            set!(memory, strings, destination, sum);
                        }
                        AddressKind::STRING_MEMORY => {
                            let left_value = get_memory!(memory, strings, left);
                            let right_value = match right.kind {
                                AddressKind::STRING_CONSTANT => {
                                    get_constant!(call.chunk, string_constants, right)
                                }
                                AddressKind::STRING_MEMORY => {
                                    get_memory!(memory, strings, right)
                                }
                                AddressKind::STRING_REGISTER => {
                                    get_register!(memory, strings, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let mut sum = DustString::new();

                            sum.push_str(left_value);
                            sum.push_str(right_value);

                            set!(memory, strings, destination, sum);
                        }
                        AddressKind::STRING_REGISTER => {
                            let left_value = get_register!(memory, strings, left);
                            let right_value = match right.kind {
                                AddressKind::STRING_CONSTANT => {
                                    get_constant!(call.chunk, string_constants, right)
                                }
                                AddressKind::STRING_MEMORY => {
                                    get_memory!(memory, strings, right)
                                }
                                AddressKind::STRING_REGISTER => {
                                    get_register!(memory, strings, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let mut sum = DustString::new();

                            sum.push_str(left_value);
                            sum.push_str(right_value);

                            set!(memory, strings, destination, sum);
                        }
                        _ => malformed_instruction!(instruction, ip),
                    }
                }
                Operation::SUBTRACT => {
                    let Subtract {
                        destination,
                        left,
                        right,
                    } = Subtract::from(&instruction);

                    match left.kind {
                        AddressKind::BYTE_MEMORY => {
                            let left_value = get_memory!(memory, bytes, left);
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => get_memory!(memory, bytes, right),
                                AddressKind::BYTE_REGISTER => get_register!(memory, bytes, right),
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let difference = left_value - right_value;

                            set!(memory, bytes, destination, difference);
                        }
                        AddressKind::BYTE_REGISTER => {
                            let left_value = get_register!(memory, bytes, left);
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => get_memory!(memory, bytes, right),
                                AddressKind::BYTE_REGISTER => get_register!(memory, bytes, right),
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let difference = left_value - right_value;

                            set!(memory, bytes, destination, difference);
                        }
                        AddressKind::FLOAT_CONSTANT => {
                            let left_value = get_constant!(call.chunk, float_constants, left);
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    get_constant!(call.chunk, float_constants, right)
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    get_memory!(memory, floats, right)
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    get_register!(memory, floats, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let difference = left_value - right_value;

                            set!(memory, floats, destination, difference);
                        }
                        AddressKind::FLOAT_MEMORY => {
                            let left_value = get_memory!(memory, floats, left);
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    get_constant!(call.chunk, float_constants, right)
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    get_memory!(memory, floats, right)
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    get_register!(memory, floats, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let difference = left_value - right_value;

                            set!(memory, floats, destination, difference);
                        }
                        AddressKind::FLOAT_REGISTER => {
                            let left_value = get_register!(memory, floats, left);
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    get_constant!(call.chunk, float_constants, right)
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    get_memory!(memory, floats, right)
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    get_register!(memory, floats, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let difference = left_value - right_value;

                            set!(memory, floats, destination, difference);
                        }
                        AddressKind::INTEGER_CONSTANT => {
                            let left_value = get_constant!(call.chunk, integer_constants, left);
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    get_constant!(call.chunk, integer_constants, right)
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    get_memory!(memory, integers, right)
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    get_register!(memory, integers, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let difference = left_value - right_value;

                            set!(memory, integers, destination, difference);
                        }
                        AddressKind::INTEGER_MEMORY => {
                            let left_value = get_memory!(memory, integers, left);
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    get_constant!(call.chunk, integer_constants, right)
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    get_memory!(memory, integers, right)
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    get_register!(memory, integers, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let difference = left_value - right_value;

                            set!(memory, integers, destination, difference);
                        }
                        AddressKind::INTEGER_REGISTER => {
                            let left_value = get_register!(memory, integers, left);
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    get_constant!(call.chunk, integer_constants, right)
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    get_memory!(memory, integers, right)
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    get_register!(memory, integers, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let difference = left_value - right_value;

                            set!(memory, integers, destination, difference);
                        }
                        _ => todo!(),
                    }
                }
                Operation::MULTIPLY => {
                    let Multiply {
                        destination,
                        left,
                        right,
                    } = Multiply::from(&instruction);

                    match left.kind {
                        AddressKind::BYTE_MEMORY => {
                            let left_value = get_memory!(memory, bytes, left);
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => get_memory!(memory, bytes, right),
                                AddressKind::BYTE_REGISTER => get_register!(memory, bytes, right),
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let product = left_value * right_value;

                            set!(memory, bytes, destination, product);
                        }
                        AddressKind::BYTE_REGISTER => {
                            let left_value = get_register!(memory, bytes, left);
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => get_memory!(memory, bytes, right),
                                AddressKind::BYTE_REGISTER => get_register!(memory, bytes, right),
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let product = left_value * right_value;

                            set!(memory, bytes, destination, product);
                        }
                        AddressKind::FLOAT_CONSTANT => {
                            let left_value = get_constant!(call.chunk, float_constants, left);
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    get_constant!(call.chunk, float_constants, right)
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    get_memory!(memory, floats, right)
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    get_register!(memory, floats, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let product = left_value * right_value;

                            set!(memory, floats, destination, product);
                        }
                        AddressKind::FLOAT_MEMORY => {
                            let left_value = get_memory!(memory, floats, left);
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    get_constant!(call.chunk, float_constants, right)
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    get_memory!(memory, floats, right)
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    get_register!(memory, floats, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let product = left_value * right_value;

                            set!(memory, floats, destination, product);
                        }
                        AddressKind::FLOAT_REGISTER => {
                            let left_value = get_register!(memory, floats, left);
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    get_constant!(call.chunk, float_constants, right)
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    get_memory!(memory, floats, right)
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    get_register!(memory, floats, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let product = left_value * right_value;

                            set!(memory, floats, destination, product);
                        }
                        AddressKind::INTEGER_CONSTANT => {
                            let left_value = get_constant!(call.chunk, integer_constants, left);
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    get_constant!(call.chunk, integer_constants, right)
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    get_memory!(memory, integers, right)
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    get_register!(memory, integers, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let product = left_value * right_value;

                            set!(memory, integers, destination, product);
                        }
                        AddressKind::INTEGER_MEMORY => {
                            let left_value = get_memory!(memory, integers, left);
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    get_constant!(call.chunk, integer_constants, right)
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    get_memory!(memory, integers, right)
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    get_register!(memory, integers, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let product = left_value * right_value;

                            set!(memory, integers, destination, product);
                        }
                        AddressKind::INTEGER_REGISTER => {
                            let left_value = get_register!(memory, integers, left);
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    get_constant!(call.chunk, integer_constants, right)
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    get_memory!(memory, integers, right)
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    get_register!(memory, integers, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let product = left_value * right_value;

                            set!(memory, integers, destination, product);
                        }
                        _ => malformed_instruction!(instruction, ip),
                    }
                }
                Operation::DIVIDE => {
                    let Divide {
                        destination,
                        left,
                        right,
                    } = Divide::from(&instruction);

                    match left.kind {
                        AddressKind::BYTE_MEMORY => {
                            let left_value = get_memory!(memory, bytes, left);
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => get_memory!(memory, bytes, right),
                                AddressKind::BYTE_REGISTER => get_register!(memory, bytes, right),
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let quotient = left_value / right_value;

                            set!(memory, bytes, destination, quotient);
                        }
                        AddressKind::BYTE_REGISTER => {
                            let left_value = get_register!(memory, bytes, left);
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => get_memory!(memory, bytes, right),
                                AddressKind::BYTE_REGISTER => get_register!(memory, bytes, right),
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let quotient = left_value / right_value;

                            set!(memory, bytes, destination, quotient);
                        }
                        AddressKind::FLOAT_CONSTANT => {
                            let left_value = get_constant!(call.chunk, float_constants, left);
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    get_constant!(call.chunk, float_constants, right)
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    get_memory!(memory, floats, right)
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    get_register!(memory, floats, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let quotient = left_value / right_value;

                            set!(memory, floats, destination, quotient);
                        }
                        AddressKind::FLOAT_MEMORY => {
                            let left_value = get_memory!(memory, floats, left);
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    get_constant!(call.chunk, float_constants, right)
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    get_memory!(memory, floats, right)
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    get_register!(memory, floats, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let quotient = left_value / right_value;

                            set!(memory, floats, destination, quotient);
                        }
                        AddressKind::FLOAT_REGISTER => {
                            let left_value = get_register!(memory, floats, left);
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    get_constant!(call.chunk, float_constants, right)
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    get_memory!(memory, floats, right)
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    get_register!(memory, floats, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let quotient = left_value / right_value;

                            set!(memory, floats, destination, quotient);
                        }
                        AddressKind::INTEGER_CONSTANT => {
                            let left_value = get_constant!(call.chunk, integer_constants, left);
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    get_constant!(call.chunk, integer_constants, right)
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    get_memory!(memory, integers, right)
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    get_register!(memory, integers, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let quotient = left_value / right_value;

                            set!(memory, integers, destination, quotient);
                        }
                        AddressKind::INTEGER_MEMORY => {
                            let left_value = get_memory!(memory, integers, left);
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    get_constant!(call.chunk, integer_constants, right)
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    get_memory!(memory, integers, right)
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    get_register!(memory, integers, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let quotient = left_value / right_value;

                            set!(memory, integers, destination, quotient);
                        }
                        AddressKind::INTEGER_REGISTER => {
                            let left_value = get_register!(memory, integers, left);
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    get_constant!(call.chunk, integer_constants, right)
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    get_memory!(memory, integers, right)
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    get_register!(memory, integers, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let quotient = left_value / right_value;

                            set!(memory, integers, destination, quotient);
                        }
                        _ => malformed_instruction!(instruction, ip),
                    }
                }
                Operation::MODULO => {
                    let Modulo {
                        destination,
                        left,
                        right,
                    } = Modulo::from(&instruction);

                    match left.kind {
                        AddressKind::BYTE_MEMORY => {
                            let left_value = get_memory!(memory, bytes, left);
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => get_memory!(memory, bytes, right),
                                AddressKind::BYTE_REGISTER => get_register!(memory, bytes, right),
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let remainder = left_value % right_value;

                            set!(memory, bytes, destination, remainder);
                        }
                        AddressKind::BYTE_REGISTER => {
                            let left_value = get_register!(memory, bytes, left);
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => get_memory!(memory, bytes, right),
                                AddressKind::BYTE_REGISTER => get_register!(memory, bytes, right),
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let remainder = left_value % right_value;

                            set!(memory, bytes, destination, remainder);
                        }
                        AddressKind::FLOAT_CONSTANT => {
                            let left_value = get_constant!(call.chunk, float_constants, left);
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    get_constant!(call.chunk, float_constants, right)
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    get_memory!(memory, floats, right)
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    get_register!(memory, floats, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let remainder = left_value % right_value;

                            set!(memory, floats, destination, remainder);
                        }
                        AddressKind::FLOAT_MEMORY => {
                            let left_value = get_memory!(memory, floats, left);
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    get_constant!(call.chunk, float_constants, right)
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    get_memory!(memory, floats, right)
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    get_register!(memory, floats, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let remainder = left_value % right_value;

                            set!(memory, floats, destination, remainder);
                        }
                        AddressKind::FLOAT_REGISTER => {
                            let left_value = get_register!(memory, floats, left);
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    get_constant!(call.chunk, float_constants, right)
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    get_memory!(memory, floats, right)
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    get_register!(memory, floats, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let remainder = left_value % right_value;

                            set!(memory, floats, destination, remainder);
                        }
                        AddressKind::INTEGER_CONSTANT => {
                            let left_value = get_constant!(call.chunk, integer_constants, left);
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    get_constant!(call.chunk, integer_constants, right)
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    get_memory!(memory, integers, right)
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    get_register!(memory, integers, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let remainder = left_value % right_value;

                            set!(memory, integers, destination, remainder);
                        }
                        AddressKind::INTEGER_MEMORY => {
                            let left_value = get_memory!(memory, integers, left);
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    get_constant!(call.chunk, integer_constants, right)
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    get_memory!(memory, integers, right)
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    get_register!(memory, integers, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let remainder = left_value % right_value;

                            set!(memory, integers, destination, remainder);
                        }
                        AddressKind::INTEGER_REGISTER => {
                            let left_value = get_register!(memory, integers, left);
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    get_constant!(call.chunk, integer_constants, right)
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    get_memory!(memory, integers, right)
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    get_register!(memory, integers, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };
                            let remainder = left_value % right_value;

                            set!(memory, integers, destination, remainder);
                        }
                        _ => malformed_instruction!(instruction, ip),
                    }
                }
                Operation::EQUAL => {
                    let Equal {
                        comparator,
                        left,
                        right,
                    } = Equal::from(&instruction);

                    let is_equal = match left.kind {
                        AddressKind::BOOLEAN_MEMORY => {
                            let left_value = get_memory!(memory, booleans, left);
                            let right_value = match right.kind {
                                AddressKind::BOOLEAN_MEMORY => get_memory!(memory, booleans, right),
                                AddressKind::BOOLEAN_REGISTER => {
                                    get_register!(memory, booleans, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value == right_value
                        }
                        AddressKind::BOOLEAN_REGISTER => {
                            let left_value = get_register!(memory, booleans, left);
                            let right_value = match right.kind {
                                AddressKind::BOOLEAN_MEMORY => get_memory!(memory, booleans, right),
                                AddressKind::BOOLEAN_REGISTER => {
                                    get_register!(memory, booleans, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value == right_value
                        }
                        AddressKind::BYTE_MEMORY => {
                            let left_value = get_memory!(memory, bytes, left);
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => get_memory!(memory, bytes, right),
                                AddressKind::BYTE_REGISTER => get_register!(memory, bytes, right),
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value == right_value
                        }
                        AddressKind::BYTE_REGISTER => {
                            let left_value = get_register!(memory, bytes, left);
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => get_memory!(memory, bytes, right),
                                AddressKind::BYTE_REGISTER => get_register!(memory, bytes, right),
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value == right_value
                        }
                        AddressKind::CHARACTER_CONSTANT => {
                            let left_value = get_constant!(call.chunk, character_constants, left);
                            let right_value = match right.kind {
                                AddressKind::CHARACTER_CONSTANT => {
                                    get_constant!(call.chunk, character_constants, right)
                                }
                                AddressKind::CHARACTER_MEMORY => {
                                    get_memory!(memory, characters, right)
                                }
                                AddressKind::CHARACTER_REGISTER => {
                                    get_register!(memory, characters, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value == right_value
                        }
                        AddressKind::CHARACTER_MEMORY => {
                            let left_value = get_memory!(memory, characters, left);
                            let right_value = match right.kind {
                                AddressKind::CHARACTER_CONSTANT => {
                                    get_constant!(call.chunk, character_constants, right)
                                }
                                AddressKind::CHARACTER_MEMORY => {
                                    get_memory!(memory, characters, right)
                                }
                                AddressKind::CHARACTER_REGISTER => {
                                    get_register!(memory, characters, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value == right_value
                        }
                        AddressKind::CHARACTER_REGISTER => {
                            let left_value = get_register!(memory, characters, left);
                            let right_value = match right.kind {
                                AddressKind::CHARACTER_CONSTANT => {
                                    get_constant!(call.chunk, character_constants, right)
                                }
                                AddressKind::CHARACTER_MEMORY => {
                                    get_memory!(memory, characters, right)
                                }
                                AddressKind::CHARACTER_REGISTER => {
                                    get_register!(memory, characters, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value == right_value
                        }
                        AddressKind::FLOAT_CONSTANT => {
                            let left_value = get_constant!(call.chunk, float_constants, left);
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    get_constant!(call.chunk, float_constants, right)
                                }
                                AddressKind::FLOAT_MEMORY => get_memory!(memory, floats, right),
                                AddressKind::FLOAT_REGISTER => get_register!(memory, floats, right),
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value == right_value
                        }
                        AddressKind::FLOAT_MEMORY => {
                            let left_value = get_memory!(memory, floats, left);
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    get_constant!(call.chunk, float_constants, right)
                                }
                                AddressKind::FLOAT_MEMORY => get_memory!(memory, floats, right),
                                AddressKind::FLOAT_REGISTER => get_register!(memory, floats, right),
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value == right_value
                        }
                        AddressKind::FLOAT_REGISTER => {
                            let left_value = get_register!(memory, floats, left);
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    get_constant!(call.chunk, float_constants, right)
                                }
                                AddressKind::FLOAT_MEMORY => get_memory!(memory, floats, right),
                                AddressKind::FLOAT_REGISTER => get_register!(memory, floats, right),
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value == right_value
                        }
                        AddressKind::INTEGER_CONSTANT => {
                            let left_value = get_constant!(call.chunk, integer_constants, left);
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    get_constant!(call.chunk, integer_constants, right)
                                }
                                AddressKind::INTEGER_MEMORY => get_memory!(memory, integers, right),
                                AddressKind::INTEGER_REGISTER => {
                                    get_register!(memory, integers, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value == right_value
                        }
                        AddressKind::INTEGER_MEMORY => {
                            let left_value = get_memory!(memory, integers, left);
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    get_constant!(call.chunk, integer_constants, right)
                                }
                                AddressKind::INTEGER_MEMORY => get_memory!(memory, integers, right),
                                AddressKind::INTEGER_REGISTER => {
                                    get_register!(memory, integers, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value == right_value
                        }
                        AddressKind::INTEGER_REGISTER => {
                            let left_value = get_register!(memory, integers, left);
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    get_constant!(call.chunk, integer_constants, right)
                                }
                                AddressKind::INTEGER_MEMORY => get_memory!(memory, integers, right),
                                AddressKind::INTEGER_REGISTER => {
                                    get_register!(memory, integers, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value == right_value
                        }
                        AddressKind::STRING_CONSTANT => {
                            let left_value = get_constant!(call.chunk, string_constants, left);
                            let right_value = match right.kind {
                                AddressKind::STRING_CONSTANT => {
                                    get_constant!(call.chunk, string_constants, right)
                                }
                                AddressKind::STRING_MEMORY => {
                                    get_memory!(memory, strings, right)
                                }
                                AddressKind::STRING_REGISTER => {
                                    get_register!(memory, strings, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value == right_value
                        }
                        AddressKind::STRING_MEMORY => {
                            let left_value = get_memory!(memory, strings, left);
                            let right_value = match right.kind {
                                AddressKind::STRING_CONSTANT => {
                                    get_constant!(call.chunk, string_constants, right)
                                }
                                AddressKind::STRING_MEMORY => {
                                    get_memory!(memory, strings, right)
                                }
                                AddressKind::STRING_REGISTER => {
                                    get_register!(memory, strings, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value == right_value
                        }
                        AddressKind::STRING_REGISTER => {
                            let left_value = get_register!(memory, strings, left);
                            let right_value = match right.kind {
                                AddressKind::STRING_CONSTANT => {
                                    get_constant!(call.chunk, string_constants, right)
                                }
                                AddressKind::STRING_MEMORY => {
                                    get_memory!(memory, strings, right)
                                }
                                AddressKind::STRING_REGISTER => {
                                    get_register!(memory, strings, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value == right_value
                        }
                        AddressKind::LIST_MEMORY => {
                            let left_value = get_memory!(memory, lists, left);
                            let right_value = match right.kind {
                                AddressKind::LIST_MEMORY => get_memory!(memory, lists, right),
                                AddressKind::LIST_REGISTER => get_register!(memory, lists, right),
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value == right_value
                        }
                        AddressKind::LIST_REGISTER => {
                            let left_value = get_register!(memory, lists, left);
                            let right_value = match right.kind {
                                AddressKind::LIST_MEMORY => get_memory!(memory, lists, right),
                                AddressKind::LIST_REGISTER => get_register!(memory, lists, right),
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value == right_value
                        }
                        AddressKind::FUNCTION_PROTOTYPE => {
                            let left_value = get_constant!(call.chunk, prototypes, left);
                            let right_value = match right.kind {
                                AddressKind::FUNCTION_PROTOTYPE => {
                                    get_constant!(call.chunk, prototypes, right)
                                }
                                AddressKind::FUNCTION_SELF => &call.chunk,
                                AddressKind::FUNCTION_REGISTER => {
                                    get_register!(memory, functions, right)
                                }
                                AddressKind::FUNCTION_MEMORY => {
                                    get_memory!(memory, functions, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value == right_value
                        }
                        AddressKind::FUNCTION_SELF => {
                            let left_value = &call.chunk;
                            let right_value = match right.kind {
                                AddressKind::FUNCTION_PROTOTYPE => {
                                    get_constant!(call.chunk, prototypes, right)
                                }
                                AddressKind::FUNCTION_SELF => &call.chunk,
                                AddressKind::FUNCTION_REGISTER => {
                                    get_register!(memory, functions, right)
                                }
                                AddressKind::FUNCTION_MEMORY => {
                                    get_memory!(memory, functions, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value == right_value
                        }
                        AddressKind::FUNCTION_REGISTER => {
                            let left_value = get_register!(memory, functions, left);
                            let right_value = match right.kind {
                                AddressKind::FUNCTION_PROTOTYPE => {
                                    get_constant!(call.chunk, prototypes, right)
                                }
                                AddressKind::FUNCTION_SELF => &call.chunk,
                                AddressKind::FUNCTION_REGISTER => {
                                    get_register!(memory, functions, right)
                                }
                                AddressKind::FUNCTION_MEMORY => {
                                    get_memory!(memory, functions, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value == right_value
                        }
                        AddressKind::FUNCTION_MEMORY => {
                            let left_value = get_memory!(memory, functions, left);
                            let right_value = match right.kind {
                                AddressKind::FUNCTION_PROTOTYPE => {
                                    get_constant!(call.chunk, prototypes, right)
                                }
                                AddressKind::FUNCTION_SELF => &call.chunk,
                                AddressKind::FUNCTION_REGISTER => {
                                    get_register!(memory, functions, right)
                                }
                                AddressKind::FUNCTION_MEMORY => {
                                    get_memory!(memory, functions, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value == right_value
                        }
                        _ => malformed_instruction!(instruction, ip),
                    };

                    if is_equal == comparator {
                        call.ip += 1;
                    }
                }
                Operation::LESS => {
                    let Less {
                        comparator,
                        left,
                        right,
                    } = Less::from(&instruction);

                    let is_less_than = match left.kind {
                        AddressKind::BOOLEAN_MEMORY => {
                            let left_value = get_memory!(memory, booleans, left);
                            let right_value = match right.kind {
                                AddressKind::BOOLEAN_MEMORY => get_memory!(memory, booleans, right),
                                AddressKind::BOOLEAN_REGISTER => {
                                    get_register!(memory, booleans, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value < right_value
                        }
                        AddressKind::BOOLEAN_REGISTER => {
                            let left_value = get_register!(memory, booleans, left);
                            let right_value = match right.kind {
                                AddressKind::BOOLEAN_MEMORY => get_memory!(memory, booleans, right),
                                AddressKind::BOOLEAN_REGISTER => {
                                    get_register!(memory, booleans, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value < right_value
                        }
                        AddressKind::BYTE_MEMORY => {
                            let left_value = get_memory!(memory, bytes, left);
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => get_memory!(memory, bytes, right),
                                AddressKind::BYTE_REGISTER => get_register!(memory, bytes, right),
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value < right_value
                        }
                        AddressKind::BYTE_REGISTER => {
                            let left_value = get_register!(memory, bytes, left);
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => get_memory!(memory, bytes, right),
                                AddressKind::BYTE_REGISTER => get_register!(memory, bytes, right),
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value < right_value
                        }
                        AddressKind::CHARACTER_CONSTANT => {
                            let left_value = get_constant!(call.chunk, character_constants, left);
                            let right_value = match right.kind {
                                AddressKind::CHARACTER_CONSTANT => {
                                    get_constant!(call.chunk, character_constants, right)
                                }
                                AddressKind::CHARACTER_MEMORY => {
                                    get_memory!(memory, characters, right)
                                }
                                AddressKind::CHARACTER_REGISTER => {
                                    get_register!(memory, characters, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value < right_value
                        }
                        AddressKind::CHARACTER_MEMORY => {
                            let left_value = get_memory!(memory, characters, left);
                            let right_value = match right.kind {
                                AddressKind::CHARACTER_CONSTANT => {
                                    get_constant!(call.chunk, character_constants, right)
                                }
                                AddressKind::CHARACTER_MEMORY => {
                                    get_memory!(memory, characters, right)
                                }
                                AddressKind::CHARACTER_REGISTER => {
                                    get_register!(memory, characters, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value < right_value
                        }
                        AddressKind::CHARACTER_REGISTER => {
                            let left_value = get_register!(memory, characters, left);
                            let right_value = match right.kind {
                                AddressKind::CHARACTER_CONSTANT => {
                                    get_constant!(call.chunk, character_constants, right)
                                }
                                AddressKind::CHARACTER_MEMORY => {
                                    get_memory!(memory, characters, right)
                                }
                                AddressKind::CHARACTER_REGISTER => {
                                    get_register!(memory, characters, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value < right_value
                        }
                        AddressKind::FLOAT_CONSTANT => {
                            let left_value = get_constant!(call.chunk, float_constants, left);
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    get_constant!(call.chunk, float_constants, right)
                                }
                                AddressKind::FLOAT_MEMORY => get_memory!(memory, floats, right),
                                AddressKind::FLOAT_REGISTER => get_register!(memory, floats, right),
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value < right_value
                        }
                        AddressKind::FLOAT_MEMORY => {
                            let left_value = get_memory!(memory, floats, left);
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    get_constant!(call.chunk, float_constants, right)
                                }
                                AddressKind::FLOAT_MEMORY => get_memory!(memory, floats, right),
                                AddressKind::FLOAT_REGISTER => get_register!(memory, floats, right),
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value < right_value
                        }
                        AddressKind::FLOAT_REGISTER => {
                            let left_value = get_register!(memory, floats, left);
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    get_constant!(call.chunk, float_constants, right)
                                }
                                AddressKind::FLOAT_MEMORY => get_memory!(memory, floats, right),
                                AddressKind::FLOAT_REGISTER => get_register!(memory, floats, right),
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value < right_value
                        }
                        AddressKind::INTEGER_CONSTANT => {
                            let left_value = get_constant!(call.chunk, integer_constants, left);
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    get_constant!(call.chunk, integer_constants, right)
                                }
                                AddressKind::INTEGER_MEMORY => get_memory!(memory, integers, right),
                                AddressKind::INTEGER_REGISTER => {
                                    get_register!(memory, integers, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value < right_value
                        }
                        AddressKind::INTEGER_MEMORY => {
                            let left_value = get_memory!(memory, integers, left);
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    &call.chunk.integer_constants[right.index as usize]
                                }
                                AddressKind::INTEGER_MEMORY => get_memory!(memory, integers, right),
                                AddressKind::INTEGER_REGISTER => {
                                    get_register!(memory, integers, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value < right_value
                        }
                        AddressKind::INTEGER_REGISTER => {
                            let left_value = get_register!(memory, integers, left);
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    &call.chunk.integer_constants[right.index as usize]
                                }
                                AddressKind::INTEGER_MEMORY => get_memory!(memory, integers, right),
                                AddressKind::INTEGER_REGISTER => {
                                    get_register!(memory, integers, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value < right_value
                        }
                        AddressKind::STRING_CONSTANT => {
                            let left_value = get_constant!(call.chunk, string_constants, left);
                            let right_value = match right.kind {
                                AddressKind::STRING_CONSTANT => {
                                    get_constant!(call.chunk, string_constants, right)
                                }
                                AddressKind::STRING_MEMORY => {
                                    get_memory!(memory, strings, right)
                                }
                                AddressKind::STRING_REGISTER => {
                                    get_register!(memory, strings, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value < right_value
                        }
                        AddressKind::STRING_MEMORY => {
                            let left_value = get_memory!(memory, strings, left);
                            let right_value = match right.kind {
                                AddressKind::STRING_CONSTANT => {
                                    &call.chunk.string_constants[right.index as usize].clone()
                                }
                                AddressKind::STRING_MEMORY => {
                                    get_memory!(memory, strings, right)
                                }
                                AddressKind::STRING_REGISTER => {
                                    get_register!(memory, strings, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value < right_value
                        }
                        AddressKind::STRING_REGISTER => {
                            let left_value = get_register!(memory, strings, left);
                            let right_value = match right.kind {
                                AddressKind::STRING_CONSTANT => {
                                    &call.chunk.string_constants[right.index as usize].clone()
                                }
                                AddressKind::STRING_MEMORY => {
                                    get_memory!(memory, strings, right)
                                }
                                AddressKind::STRING_REGISTER => {
                                    get_register!(memory, strings, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value < right_value
                        }
                        AddressKind::LIST_MEMORY => {
                            let left_value = get_memory!(memory, lists, left);
                            let right_value = match right.kind {
                                AddressKind::LIST_MEMORY => get_memory!(memory, lists, right),
                                AddressKind::LIST_REGISTER => get_register!(memory, lists, right),
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value < right_value
                        }
                        AddressKind::LIST_REGISTER => {
                            let left_value = get_register!(memory, lists, left);
                            let right_value = match right.kind {
                                AddressKind::LIST_MEMORY => get_memory!(memory, lists, right),
                                AddressKind::LIST_REGISTER => get_register!(memory, lists, right),
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value < right_value
                        }
                        AddressKind::FUNCTION_PROTOTYPE => {
                            let left_value = get_constant!(call.chunk, prototypes, left);
                            let right_value = match right.kind {
                                AddressKind::FUNCTION_PROTOTYPE => {
                                    get_constant!(call.chunk, prototypes, right)
                                }
                                AddressKind::FUNCTION_SELF => &call.chunk,
                                AddressKind::FUNCTION_REGISTER => {
                                    get_register!(memory, functions, right)
                                }
                                AddressKind::FUNCTION_MEMORY => {
                                    get_memory!(memory, functions, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value < right_value
                        }
                        AddressKind::FUNCTION_SELF => {
                            let left_value = &call.chunk;
                            let right_value = match right.kind {
                                AddressKind::FUNCTION_PROTOTYPE => {
                                    get_constant!(call.chunk, prototypes, right)
                                }
                                AddressKind::FUNCTION_SELF => &call.chunk,
                                AddressKind::FUNCTION_REGISTER => {
                                    get_register!(memory, functions, right)
                                }
                                AddressKind::FUNCTION_MEMORY => {
                                    get_memory!(memory, functions, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value < right_value
                        }
                        AddressKind::FUNCTION_REGISTER => {
                            let left_value = get_register!(memory, functions, left);
                            let right_value = match right.kind {
                                AddressKind::FUNCTION_PROTOTYPE => {
                                    get_constant!(call.chunk, prototypes, right)
                                }
                                AddressKind::FUNCTION_SELF => &call.chunk,
                                AddressKind::FUNCTION_REGISTER => {
                                    get_register!(memory, functions, right)
                                }
                                AddressKind::FUNCTION_MEMORY => {
                                    get_memory!(memory, functions, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value < right_value
                        }
                        AddressKind::FUNCTION_MEMORY => {
                            let left_value = get_memory!(memory, functions, left);
                            let right_value = match right.kind {
                                AddressKind::FUNCTION_PROTOTYPE => {
                                    get_constant!(call.chunk, prototypes, right)
                                }
                                AddressKind::FUNCTION_SELF => &call.chunk,
                                AddressKind::FUNCTION_REGISTER => {
                                    get_register!(memory, functions, right)
                                }
                                AddressKind::FUNCTION_MEMORY => {
                                    get_memory!(memory, functions, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value < right_value
                        }
                        _ => malformed_instruction!(instruction, ip),
                    };

                    if is_less_than == comparator {
                        call.ip += 1;
                    }
                }
                Operation::LESS_EQUAL => {
                    let LessEqual {
                        comparator,
                        left,
                        right,
                    } = LessEqual::from(&instruction);

                    let is_less_than_or_equal = match left.kind {
                        AddressKind::BOOLEAN_MEMORY => {
                            let left_value = get_memory!(memory, booleans, left);
                            let right_value = match right.kind {
                                AddressKind::BOOLEAN_MEMORY => get_memory!(memory, booleans, right),
                                AddressKind::BOOLEAN_REGISTER => {
                                    get_register!(memory, booleans, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value <= right_value
                        }
                        AddressKind::BOOLEAN_REGISTER => {
                            let left_value = get_register!(memory, booleans, left);
                            let right_value = match right.kind {
                                AddressKind::BOOLEAN_MEMORY => get_memory!(memory, booleans, right),
                                AddressKind::BOOLEAN_REGISTER => {
                                    get_register!(memory, booleans, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value <= right_value
                        }
                        AddressKind::BYTE_MEMORY => {
                            let left_value = get_memory!(memory, bytes, left);
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => get_memory!(memory, bytes, right),
                                AddressKind::BYTE_REGISTER => get_register!(memory, bytes, right),
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value <= right_value
                        }
                        AddressKind::BYTE_REGISTER => {
                            let left_value = get_register!(memory, bytes, left);
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => get_memory!(memory, bytes, right),
                                AddressKind::BYTE_REGISTER => get_register!(memory, bytes, right),
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value <= right_value
                        }
                        AddressKind::CHARACTER_CONSTANT => {
                            let left_value = get_constant!(call.chunk, character_constants, left);
                            let right_value = match right.kind {
                                AddressKind::CHARACTER_CONSTANT => {
                                    get_constant!(call.chunk, character_constants, right)
                                }
                                AddressKind::CHARACTER_MEMORY => {
                                    get_memory!(memory, characters, right)
                                }
                                AddressKind::CHARACTER_REGISTER => {
                                    get_register!(memory, characters, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value <= right_value
                        }
                        AddressKind::CHARACTER_MEMORY => {
                            let left_value = get_memory!(memory, characters, left);
                            let right_value = match right.kind {
                                AddressKind::CHARACTER_CONSTANT => {
                                    &call.chunk.character_constants[right.index as usize]
                                }
                                AddressKind::CHARACTER_MEMORY => {
                                    get_memory!(memory, characters, right)
                                }
                                AddressKind::CHARACTER_REGISTER => {
                                    get_register!(memory, characters, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value <= right_value
                        }
                        AddressKind::CHARACTER_REGISTER => {
                            let left_value = get_register!(memory, characters, left);
                            let right_value = match right.kind {
                                AddressKind::CHARACTER_CONSTANT => {
                                    &call.chunk.character_constants[right.index as usize]
                                }
                                AddressKind::CHARACTER_MEMORY => {
                                    get_memory!(memory, characters, right)
                                }
                                AddressKind::CHARACTER_REGISTER => {
                                    get_register!(memory, characters, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value <= right_value
                        }
                        AddressKind::FLOAT_CONSTANT => {
                            let left_value = get_constant!(call.chunk, float_constants, left);
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    get_constant!(call.chunk, float_constants, right)
                                }
                                AddressKind::FLOAT_MEMORY => get_memory!(memory, floats, right),
                                AddressKind::FLOAT_REGISTER => get_register!(memory, floats, right),
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value <= right_value
                        }
                        AddressKind::FLOAT_MEMORY => {
                            let left_value = get_memory!(memory, floats, left);
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    &call.chunk.float_constants[right.index as usize]
                                }
                                AddressKind::FLOAT_MEMORY => get_memory!(memory, floats, right),
                                AddressKind::FLOAT_REGISTER => get_register!(memory, floats, right),
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value <= right_value
                        }
                        AddressKind::FLOAT_REGISTER => {
                            let left_value = get_register!(memory, floats, left);
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    &call.chunk.float_constants[right.index as usize]
                                }
                                AddressKind::FLOAT_MEMORY => get_memory!(memory, floats, right),
                                AddressKind::FLOAT_REGISTER => get_register!(memory, floats, right),
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value <= right_value
                        }
                        AddressKind::INTEGER_CONSTANT => {
                            let left_value = get_constant!(call.chunk, integer_constants, left);
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    get_constant!(call.chunk, integer_constants, right)
                                }
                                AddressKind::INTEGER_MEMORY => get_memory!(memory, integers, right),
                                AddressKind::INTEGER_REGISTER => {
                                    get_register!(memory, integers, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value <= right_value
                        }
                        AddressKind::INTEGER_MEMORY => {
                            let left_value = get_memory!(memory, integers, left);
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    &call.chunk.integer_constants[right.index as usize]
                                }
                                AddressKind::INTEGER_MEMORY => get_memory!(memory, integers, right),
                                AddressKind::INTEGER_REGISTER => {
                                    get_register!(memory, integers, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value <= right_value
                        }
                        AddressKind::INTEGER_REGISTER => {
                            let left_value = get_register!(memory, integers, left);
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    &call.chunk.integer_constants[right.index as usize]
                                }
                                AddressKind::INTEGER_MEMORY => get_memory!(memory, integers, right),
                                AddressKind::INTEGER_REGISTER => {
                                    get_register!(memory, integers, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value <= right_value
                        }
                        AddressKind::STRING_CONSTANT => {
                            let left_value = get_constant!(call.chunk, string_constants, left);
                            let right_value = match right.kind {
                                AddressKind::STRING_CONSTANT => {
                                    get_constant!(call.chunk, string_constants, right)
                                }
                                AddressKind::STRING_MEMORY => {
                                    get_memory!(memory, strings, right)
                                }
                                AddressKind::STRING_REGISTER => {
                                    get_register!(memory, strings, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value <= right_value
                        }
                        AddressKind::STRING_MEMORY => {
                            let left_value = get_memory!(memory, strings, left);
                            let right_value = match right.kind {
                                AddressKind::STRING_CONSTANT => {
                                    &call.chunk.string_constants[right.index as usize].clone()
                                }
                                AddressKind::STRING_MEMORY => {
                                    get_memory!(memory, strings, right)
                                }
                                AddressKind::STRING_REGISTER => {
                                    get_register!(memory, strings, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value <= right_value
                        }
                        AddressKind::STRING_REGISTER => {
                            let left_value = get_register!(memory, strings, left);
                            let right_value = match right.kind {
                                AddressKind::STRING_CONSTANT => {
                                    &call.chunk.string_constants[right.index as usize].clone()
                                }
                                AddressKind::STRING_MEMORY => {
                                    get_memory!(memory, strings, right)
                                }
                                AddressKind::STRING_REGISTER => {
                                    get_register!(memory, strings, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value <= right_value
                        }
                        AddressKind::LIST_MEMORY => {
                            let left_value = get_memory!(memory, lists, left);
                            let right_value = match right.kind {
                                AddressKind::LIST_MEMORY => get_memory!(memory, lists, right),
                                AddressKind::LIST_REGISTER => get_register!(memory, lists, right),
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value <= right_value
                        }
                        AddressKind::LIST_REGISTER => {
                            let left_value = get_register!(memory, lists, left);
                            let right_value = match right.kind {
                                AddressKind::LIST_MEMORY => get_memory!(memory, lists, right),
                                AddressKind::LIST_REGISTER => get_register!(memory, lists, right),
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value <= right_value
                        }
                        AddressKind::FUNCTION_PROTOTYPE => {
                            let left_value = get_constant!(call.chunk, prototypes, left);
                            let right_value = match right.kind {
                                AddressKind::FUNCTION_PROTOTYPE => {
                                    get_constant!(call.chunk, prototypes, right)
                                }
                                AddressKind::FUNCTION_SELF => &call.chunk,
                                AddressKind::FUNCTION_REGISTER => {
                                    get_register!(memory, functions, right)
                                }
                                AddressKind::FUNCTION_MEMORY => {
                                    get_memory!(memory, functions, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value < right_value
                        }
                        AddressKind::FUNCTION_SELF => {
                            let left_value = &call.chunk;
                            let right_value = match right.kind {
                                AddressKind::FUNCTION_PROTOTYPE => {
                                    get_constant!(call.chunk, prototypes, right)
                                }
                                AddressKind::FUNCTION_SELF => &call.chunk,
                                AddressKind::FUNCTION_REGISTER => {
                                    get_register!(memory, functions, right)
                                }
                                AddressKind::FUNCTION_MEMORY => {
                                    get_memory!(memory, functions, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value < right_value
                        }
                        AddressKind::FUNCTION_REGISTER => {
                            let left_value = get_register!(memory, functions, left);
                            let right_value = match right.kind {
                                AddressKind::FUNCTION_PROTOTYPE => {
                                    get_constant!(call.chunk, prototypes, right)
                                }
                                AddressKind::FUNCTION_SELF => &call.chunk,
                                AddressKind::FUNCTION_REGISTER => {
                                    get_register!(memory, functions, right)
                                }
                                AddressKind::FUNCTION_MEMORY => {
                                    get_memory!(memory, functions, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value < right_value
                        }
                        AddressKind::FUNCTION_MEMORY => {
                            let left_value = get_memory!(memory, functions, left);
                            let right_value = match right.kind {
                                AddressKind::FUNCTION_PROTOTYPE => {
                                    get_constant!(call.chunk, prototypes, right)
                                }
                                AddressKind::FUNCTION_SELF => &call.chunk,
                                AddressKind::FUNCTION_REGISTER => {
                                    get_register!(memory, functions, right)
                                }
                                AddressKind::FUNCTION_MEMORY => {
                                    get_memory!(memory, functions, right)
                                }
                                _ => malformed_instruction!(instruction, ip),
                            };

                            left_value < right_value
                        }
                        _ => malformed_instruction!(instruction, ip),
                    };

                    if is_less_than_or_equal == comparator {
                        call.ip += 1;
                    }
                }
                Operation::NEGATE => {
                    let Negate {
                        destination,
                        operand,
                    } = Negate::from(&instruction);

                    match operand.kind {
                        AddressKind::FLOAT_CONSTANT => {
                            let float = get_constant!(call.chunk, float_constants, operand);

                            set!(memory, floats, destination, -(*float));
                        }
                        AddressKind::FLOAT_MEMORY => {
                            let float = get_memory!(memory, floats, operand);

                            set!(memory, floats, destination, -(*float));
                        }
                        AddressKind::FLOAT_REGISTER => {
                            let float = get_register!(memory, floats, operand);

                            set!(memory, floats, destination, -(*float));
                        }
                        AddressKind::INTEGER_CONSTANT => {
                            let integer = get_constant!(call.chunk, integer_constants, operand);

                            set!(memory, integers, destination, -(*integer));
                        }
                        AddressKind::INTEGER_MEMORY => {
                            let integer = get_memory!(memory, integers, operand);

                            set!(memory, integers, destination, -(*integer));
                        }
                        AddressKind::INTEGER_REGISTER => {
                            let integer = get_register!(memory, integers, operand);

                            set!(memory, integers, destination, -(*integer));
                        }
                        _ => malformed_instruction!(instruction, ip),
                    }
                }
                Operation::NOT => {
                    let Not {
                        destination,
                        operand,
                    } = Not::from(&instruction);

                    let boolean = match operand.kind {
                        AddressKind::BOOLEAN_MEMORY => get_memory!(memory, booleans, operand),
                        AddressKind::BOOLEAN_REGISTER => get_register!(memory, booleans, operand),
                        _ => malformed_instruction!(instruction, ip),
                    };

                    set!(memory, booleans, destination, !(*boolean));
                }
                Operation::TEST => {
                    let Test {
                        comparator,
                        operand,
                    } = Test::from(&instruction);
                    let is_true = match operand.kind {
                        AddressKind::BOOLEAN_MEMORY => get_memory!(memory, booleans, operand),
                        AddressKind::BOOLEAN_REGISTER => get_register!(memory, booleans, operand),
                        _ => malformed_instruction!(instruction, ip),
                    };

                    if *is_true == comparator {
                        call.ip += 1;
                    }
                }
                Operation::TEST_SET => todo!(),
                Operation::CALL_NATIVE => {
                    let CallNative {
                        destination,
                        function,
                        argument_list_index,
                    } = CallNative::from(&instruction);
                    let arguments = &call.chunk.arguments[argument_list_index as usize].clone();

                    function.call(
                        destination,
                        arguments,
                        &mut call,
                        &mut memory,
                        &self.threads,
                    );
                }
                Operation::CALL => {
                    let Call {
                        destination,
                        function: function_address,
                        argument_list_index,
                        return_type: _,
                    } = Call::from(&instruction);

                    let function = match function_address.kind {
                        AddressKind::FUNCTION_REGISTER => {
                            get_register!(memory, functions, function_address)
                        }
                        AddressKind::FUNCTION_MEMORY => {
                            get_memory!(memory, functions, function_address)
                        }
                        AddressKind::FUNCTION_PROTOTYPE => {
                            get_constant!(call.chunk, prototypes, function_address)
                        }
                        AddressKind::FUNCTION_SELF => &call.chunk,
                        _ => malformed_instruction!(instruction, ip),
                    };

                    let arguments_list = &call.chunk.arguments[argument_list_index as usize];
                    let parameters_list = function.locals.iter().map(|local| local.address);
                    let new_call = CallFrame::new(Arc::clone(function), destination);
                    let mut new_memory = Memory::new(function);

                    for (argument, parameter) in arguments_list.iter().zip(parameters_list) {
                        match argument.kind {
                            AddressKind::BOOLEAN_MEMORY => {
                                let boolean = get_memory!(memory, booleans, argument);

                                match parameter.kind {
                                    AddressKind::BOOLEAN_REGISTER => {
                                        set_register!(
                                            new_memory,
                                            booleans,
                                            parameter.index,
                                            *boolean
                                        );
                                    }
                                    AddressKind::BOOLEAN_MEMORY => {
                                        set_memory!(
                                            new_memory,
                                            booleans,
                                            parameter.index,
                                            *boolean
                                        );
                                    }
                                    _ => malformed_instruction!(instruction, ip),
                                }
                            }
                            AddressKind::BOOLEAN_REGISTER => {
                                let boolean = get_register!(memory, booleans, argument);

                                match parameter.kind {
                                    AddressKind::BOOLEAN_REGISTER => {
                                        set_register!(
                                            new_memory,
                                            booleans,
                                            parameter.index,
                                            *boolean
                                        );
                                    }
                                    AddressKind::BOOLEAN_MEMORY => {
                                        set_memory!(
                                            new_memory,
                                            booleans,
                                            parameter.index,
                                            *boolean
                                        );
                                    }
                                    _ => malformed_instruction!(instruction, ip),
                                }
                            }
                            AddressKind::BYTE_MEMORY => {
                                let byte = get_memory!(memory, bytes, argument);

                                match parameter.kind {
                                    AddressKind::BYTE_REGISTER => {
                                        set_register!(new_memory, bytes, parameter.index, *byte);
                                    }
                                    AddressKind::BYTE_MEMORY => {
                                        set_memory!(new_memory, bytes, parameter.index, *byte);
                                    }
                                    _ => malformed_instruction!(instruction, ip),
                                }
                            }
                            AddressKind::BYTE_REGISTER => {
                                let byte = get_register!(memory, bytes, argument);

                                match parameter.kind {
                                    AddressKind::BYTE_REGISTER => {
                                        set_register!(new_memory, bytes, parameter.index, *byte);
                                    }
                                    AddressKind::BYTE_MEMORY => {
                                        set_memory!(new_memory, bytes, parameter.index, *byte);
                                    }
                                    _ => malformed_instruction!(instruction, ip),
                                }
                            }
                            AddressKind::CHARACTER_CONSTANT => {
                                let character =
                                    get_constant!(call.chunk, character_constants, argument);

                                match parameter.kind {
                                    AddressKind::CHARACTER_REGISTER => {
                                        set_register!(
                                            new_memory,
                                            characters,
                                            parameter.index,
                                            *character
                                        );
                                    }
                                    AddressKind::CHARACTER_MEMORY => {
                                        set_memory!(
                                            new_memory,
                                            characters,
                                            parameter.index,
                                            *character
                                        );
                                    }
                                    _ => malformed_instruction!(instruction, ip),
                                }
                            }
                            AddressKind::CHARACTER_MEMORY => {
                                let character = get_memory!(memory, characters, argument);

                                match parameter.kind {
                                    AddressKind::CHARACTER_REGISTER => {
                                        set_register!(
                                            new_memory,
                                            characters,
                                            parameter.index,
                                            *character
                                        );
                                    }
                                    AddressKind::CHARACTER_MEMORY => {
                                        set_memory!(
                                            new_memory,
                                            characters,
                                            parameter.index,
                                            *character
                                        );
                                    }
                                    _ => malformed_instruction!(instruction, ip),
                                }
                            }
                            AddressKind::CHARACTER_REGISTER => {
                                let character = get_register!(memory, characters, argument);

                                match parameter.kind {
                                    AddressKind::CHARACTER_REGISTER => {
                                        set_register!(
                                            new_memory,
                                            characters,
                                            parameter.index,
                                            *character
                                        );
                                    }
                                    AddressKind::CHARACTER_MEMORY => {
                                        set_memory!(
                                            new_memory,
                                            characters,
                                            parameter.index,
                                            *character
                                        );
                                    }
                                    _ => malformed_instruction!(instruction, ip),
                                }
                            }
                            AddressKind::FLOAT_CONSTANT => {
                                let float = get_constant!(call.chunk, float_constants, argument);

                                match parameter.kind {
                                    AddressKind::FLOAT_REGISTER => {
                                        set_register!(new_memory, floats, parameter.index, *float);
                                    }
                                    AddressKind::FLOAT_MEMORY => {
                                        set_memory!(new_memory, floats, parameter.index, *float);
                                    }
                                    _ => malformed_instruction!(instruction, ip),
                                }
                            }
                            AddressKind::FLOAT_MEMORY => {
                                let float = get_memory!(memory, floats, argument);

                                match parameter.kind {
                                    AddressKind::FLOAT_REGISTER => {
                                        set_register!(new_memory, floats, parameter.index, *float);
                                    }
                                    AddressKind::FLOAT_MEMORY => {
                                        set_memory!(new_memory, floats, parameter.index, *float);
                                    }
                                    _ => malformed_instruction!(instruction, ip),
                                }
                            }
                            AddressKind::FLOAT_REGISTER => {
                                let float = get_register!(memory, floats, argument);

                                match parameter.kind {
                                    AddressKind::FLOAT_REGISTER => {
                                        set_register!(new_memory, floats, parameter.index, *float);
                                    }
                                    AddressKind::FLOAT_MEMORY => {
                                        set_memory!(new_memory, floats, parameter.index, *float);
                                    }
                                    _ => malformed_instruction!(instruction, ip),
                                }
                            }
                            AddressKind::INTEGER_CONSTANT => {
                                let integer =
                                    get_constant!(call.chunk, integer_constants, argument);

                                match parameter.kind {
                                    AddressKind::INTEGER_REGISTER => {
                                        set_register!(
                                            new_memory,
                                            integers,
                                            parameter.index,
                                            *integer
                                        );
                                    }
                                    AddressKind::INTEGER_MEMORY => {
                                        set_memory!(
                                            new_memory,
                                            integers,
                                            parameter.index,
                                            *integer
                                        );
                                    }
                                    _ => malformed_instruction!(instruction, ip),
                                }
                            }
                            AddressKind::INTEGER_REGISTER => {
                                let integer = get_register!(memory, integers, argument);

                                match parameter.kind {
                                    AddressKind::INTEGER_REGISTER => {
                                        set_register!(
                                            new_memory,
                                            integers,
                                            parameter.index,
                                            *integer
                                        );
                                    }
                                    AddressKind::INTEGER_MEMORY => {
                                        set_memory!(
                                            new_memory,
                                            integers,
                                            parameter.index,
                                            *integer
                                        );
                                    }
                                    _ => malformed_instruction!(instruction, ip),
                                }
                            }
                            AddressKind::INTEGER_MEMORY => {
                                let integer = get_memory!(memory, integers, argument);

                                match parameter.kind {
                                    AddressKind::INTEGER_REGISTER => {
                                        set_register!(
                                            new_memory,
                                            integers,
                                            parameter.index,
                                            *integer
                                        );
                                    }
                                    AddressKind::INTEGER_MEMORY => {
                                        set_memory!(
                                            new_memory,
                                            integers,
                                            parameter.index,
                                            *integer
                                        );
                                    }
                                    _ => malformed_instruction!(instruction, ip),
                                }
                            }
                            AddressKind::STRING_CONSTANT => {
                                let string =
                                    get_constant!(call.chunk, string_constants, argument).clone();

                                match parameter.kind {
                                    AddressKind::STRING_REGISTER => {
                                        set_register!(new_memory, strings, parameter.index, string);
                                    }
                                    AddressKind::STRING_MEMORY => {
                                        set_memory!(new_memory, strings, parameter.index, string);
                                    }
                                    _ => malformed_instruction!(instruction, ip),
                                }
                            }
                            AddressKind::STRING_MEMORY => {
                                let string = get_memory!(memory, strings, argument).clone();

                                match parameter.kind {
                                    AddressKind::STRING_REGISTER => {
                                        set_register!(new_memory, strings, parameter.index, string);
                                    }
                                    AddressKind::STRING_MEMORY => {
                                        set_memory!(new_memory, strings, parameter.index, string);
                                    }
                                    _ => malformed_instruction!(instruction, ip),
                                }
                            }
                            AddressKind::STRING_REGISTER => {
                                let string = get_register!(memory, strings, argument).clone();

                                match parameter.kind {
                                    AddressKind::STRING_REGISTER => {
                                        set_register!(new_memory, strings, parameter.index, string);
                                    }
                                    AddressKind::STRING_MEMORY => {
                                        set_memory!(new_memory, strings, parameter.index, string);
                                    }
                                    _ => malformed_instruction!(instruction, ip),
                                }
                            }
                            AddressKind::LIST_MEMORY => {
                                let abstract_list = get_memory!(memory, lists, argument).clone();

                                match parameter.kind {
                                    AddressKind::LIST_REGISTER => {
                                        set_register!(
                                            new_memory,
                                            lists,
                                            parameter.index,
                                            abstract_list
                                        );
                                    }
                                    AddressKind::LIST_MEMORY => {
                                        set_memory!(
                                            new_memory,
                                            lists,
                                            parameter.index,
                                            abstract_list
                                        );
                                    }
                                    _ => malformed_instruction!(instruction, ip),
                                }
                            }
                            AddressKind::LIST_REGISTER => {
                                let abstract_list = get_register!(memory, lists, argument).clone();

                                match parameter.kind {
                                    AddressKind::LIST_REGISTER => {
                                        set_register!(
                                            new_memory,
                                            lists,
                                            parameter.index,
                                            abstract_list
                                        );
                                    }
                                    AddressKind::LIST_MEMORY => {
                                        set_memory!(
                                            new_memory,
                                            lists,
                                            parameter.index,
                                            abstract_list
                                        );
                                    }
                                    _ => malformed_instruction!(instruction, ip),
                                }
                            }
                            AddressKind::FUNCTION_REGISTER => {
                                let function = get_register!(memory, functions, argument);

                                match parameter.kind {
                                    AddressKind::FUNCTION_REGISTER => {
                                        set_register!(
                                            new_memory,
                                            functions,
                                            parameter.index,
                                            Arc::clone(function)
                                        );
                                    }
                                    AddressKind::FUNCTION_MEMORY => {
                                        set_memory!(
                                            new_memory,
                                            functions,
                                            parameter.index,
                                            Arc::clone(function)
                                        );
                                    }
                                    _ => malformed_instruction!(instruction, ip),
                                }
                            }
                            AddressKind::FUNCTION_MEMORY => {
                                let function = get_memory!(memory, functions, argument);

                                match parameter.kind {
                                    AddressKind::FUNCTION_REGISTER => {
                                        set_register!(
                                            new_memory,
                                            functions,
                                            parameter.index,
                                            Arc::clone(function)
                                        );
                                    }
                                    AddressKind::FUNCTION_MEMORY => {
                                        set_memory!(
                                            new_memory,
                                            functions,
                                            parameter.index,
                                            Arc::clone(function)
                                        );
                                    }
                                    _ => malformed_instruction!(instruction, ip),
                                }
                            }
                            AddressKind::FUNCTION_PROTOTYPE => {
                                let function = get_constant!(call.chunk, prototypes, argument);

                                match parameter.kind {
                                    AddressKind::FUNCTION_REGISTER => {
                                        set_register!(
                                            new_memory,
                                            functions,
                                            parameter.index,
                                            Arc::clone(function)
                                        );
                                    }
                                    AddressKind::FUNCTION_MEMORY => {
                                        set_memory!(
                                            new_memory,
                                            functions,
                                            parameter.index,
                                            Arc::clone(function)
                                        );
                                    }
                                    _ => malformed_instruction!(instruction, ip),
                                }
                            }
                            AddressKind::FUNCTION_SELF => {
                                let function = &call.chunk;

                                match parameter.kind {
                                    AddressKind::FUNCTION_REGISTER => {
                                        set_register!(
                                            new_memory,
                                            functions,
                                            parameter.index,
                                            Arc::clone(function)
                                        );
                                    }
                                    AddressKind::FUNCTION_MEMORY => {
                                        set_memory!(
                                            new_memory,
                                            functions,
                                            parameter.index,
                                            Arc::clone(function)
                                        );
                                    }
                                    _ => malformed_instruction!(instruction, ip),
                                }
                            }
                            _ => malformed_instruction!(instruction, ip),
                        }
                    }

                    memory_stack.push(replace(&mut memory, new_memory));
                    call_stack.push(replace(&mut call, new_call));
                }
                Operation::JUMP => {
                    let Jump {
                        offset,
                        is_positive,
                    } = Jump::from(&instruction);

                    if is_positive {
                        call.ip += offset as usize;
                    } else {
                        call.ip -= offset as usize + 1;
                    }
                }
                Operation::RETURN => {
                    let Return {
                        should_return_value,
                        return_value_address,
                    } = Return::from(&instruction);

                    let (new_call, new_memory) = match return_value_address.kind {
                        AddressKind::NONE => {
                            if call_stack.is_empty() {
                                return None;
                            }

                            (call_stack.pop().unwrap(), memory_stack.pop().unwrap())
                        }
                        AddressKind::BOOLEAN_MEMORY => {
                            let boolean = get_memory!(memory, booleans, return_value_address);

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::Boolean(*boolean));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            set!(new_memory, booleans, call.return_address, *boolean);

                            (new_call, new_memory)
                        }
                        AddressKind::BOOLEAN_REGISTER => {
                            let boolean = get_register!(memory, booleans, return_value_address);

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::Boolean(*boolean));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            set!(new_memory, booleans, call.return_address, *boolean);

                            (new_call, new_memory)
                        }
                        AddressKind::BYTE_MEMORY => {
                            let byte = memory.bytes[return_value_address.index as usize];

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::Byte(byte));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            set!(new_memory, bytes, call.return_address, byte);

                            (new_call, new_memory)
                        }
                        AddressKind::BYTE_REGISTER => {
                            let byte = memory.registers.bytes[return_value_address.index as usize];

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::Byte(byte));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            set!(new_memory, bytes, call.return_address, byte);

                            (new_call, new_memory)
                        }
                        AddressKind::CHARACTER_CONSTANT => {
                            let character = get_constant!(
                                call.chunk,
                                character_constants,
                                return_value_address
                            );

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::Character(*character));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            set!(new_memory, characters, call.return_address, *character);

                            (new_call, new_memory)
                        }
                        AddressKind::CHARACTER_MEMORY => {
                            let character = get_memory!(memory, characters, return_value_address);

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::Character(*character));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            set!(new_memory, characters, call.return_address, *character);

                            (new_call, new_memory)
                        }
                        AddressKind::CHARACTER_REGISTER => {
                            let character = get_register!(memory, characters, return_value_address);

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::Character(*character));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            set!(new_memory, characters, call.return_address, *character);

                            (new_call, new_memory)
                        }
                        AddressKind::FLOAT_CONSTANT => {
                            let float =
                                get_constant!(call.chunk, float_constants, return_value_address);

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::Float(*float));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            set!(new_memory, floats, call.return_address, *float);

                            (new_call, new_memory)
                        }
                        AddressKind::FLOAT_MEMORY => {
                            let float = get_memory!(memory, floats, return_value_address);

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::Float(*float));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            set!(new_memory, floats, call.return_address, *float);

                            (new_call, new_memory)
                        }
                        AddressKind::FLOAT_REGISTER => {
                            let float = get_register!(memory, floats, return_value_address);

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::Float(*float));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            set!(new_memory, floats, call.return_address, *float);

                            (new_call, new_memory)
                        }
                        AddressKind::INTEGER_CONSTANT => {
                            let integer =
                                get_constant!(call.chunk, integer_constants, return_value_address);

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::Integer(*integer));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            set!(new_memory, integers, call.return_address, *integer);

                            (new_call, new_memory)
                        }
                        AddressKind::INTEGER_REGISTER => {
                            let integer = get_register!(memory, integers, return_value_address);

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::Integer(*integer));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            set!(new_memory, integers, call.return_address, *integer);

                            (new_call, new_memory)
                        }
                        AddressKind::INTEGER_MEMORY => {
                            let integer = get_memory!(memory, integers, return_value_address);

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::Integer(*integer));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            set!(new_memory, integers, call.return_address, *integer);

                            (new_call, new_memory)
                        }
                        AddressKind::STRING_CONSTANT => {
                            let string =
                                get_constant!(call.chunk, string_constants, return_value_address)
                                    .clone();

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::String(string));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            set!(new_memory, strings, call.return_address, string);

                            (new_call, new_memory)
                        }
                        AddressKind::STRING_MEMORY => {
                            let string = get_memory!(memory, strings, return_value_address).clone();

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::String(string));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            set!(new_memory, strings, call.return_address, string);

                            (new_call, new_memory)
                        }
                        AddressKind::STRING_REGISTER => {
                            let string =
                                get_register!(memory, strings, return_value_address).clone();

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::String(string));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            set!(new_memory, strings, call.return_address, string);

                            (new_call, new_memory)
                        }
                        AddressKind::LIST_MEMORY => {
                            let abstract_list = get_memory!(memory, lists, return_value_address);
                            let concrete_list = memory.make_list_concrete(abstract_list);

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::List(concrete_list));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            set!(
                                new_memory,
                                lists,
                                call.return_address,
                                abstract_list.clone()
                            );

                            (new_call, new_memory)
                        }
                        AddressKind::LIST_REGISTER => {
                            let abstract_list = get_register!(memory, lists, return_value_address);
                            let concrete_list = memory.make_list_concrete(abstract_list);

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::List(concrete_list));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            set!(
                                new_memory,
                                lists,
                                call.return_address,
                                abstract_list.clone()
                            );

                            (new_call, new_memory)
                        }
                        AddressKind::FUNCTION_REGISTER => {
                            let function = get_register!(memory, functions, return_value_address);

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::Function(Arc::clone(function)));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            set!(
                                new_memory,
                                functions,
                                call.return_address,
                                Arc::clone(function)
                            );

                            (new_call, new_memory)
                        }
                        AddressKind::FUNCTION_MEMORY => {
                            let function = get_memory!(memory, functions, return_value_address);

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::Function(Arc::clone(function)));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            set!(
                                new_memory,
                                functions,
                                call.return_address,
                                Arc::clone(function)
                            );

                            (new_call, new_memory)
                        }
                        AddressKind::FUNCTION_PROTOTYPE => {
                            let function =
                                get_constant!(call.chunk, prototypes, return_value_address);

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::Function(Arc::clone(function)));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            set!(
                                new_memory,
                                functions,
                                call.return_address,
                                Arc::clone(function)
                            );

                            (new_call, new_memory)
                        }
                        AddressKind::FUNCTION_SELF => {
                            let function = &call.chunk;

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::Function(Arc::clone(function)));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            set!(
                                new_memory,
                                functions,
                                call.return_address,
                                Arc::clone(function)
                            );

                            (new_call, new_memory)
                        }
                        _ => malformed_instruction!(instruction, ip),
                    };

                    call = new_call;
                    memory = new_memory;
                }
                _ => malformed_instruction!(instruction, ip),
            }
        }
    }
}

use std::{mem::replace, sync::Arc, thread::JoinHandle};

use tracing::{Level, info, span, warn};

use crate::{
    AbstractList, Address, Chunk, ConcreteValue, DustString, Operation,
    instruction::{
        Add, AddressKind, Call, CallNative, Close, Divide, Equal, Jump, Less, LessEqual,
        LoadConstant, LoadEncoded, LoadFunction, LoadList, Modulo, Move, Multiply, Return,
        Subtract, Test,
    },
};

use super::{CallFrame, Memory};

pub struct Thread {
    chunk: Arc<Chunk>,

    _spawned_threads: Vec<JoinHandle<()>>,
}

impl Thread {
    pub fn new(chunk: Arc<Chunk>) -> Self {
        Thread {
            chunk,
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
                .as_ref()
                .map(|name| name.as_str())
                .unwrap_or_default()
        );

        let mut call_stack = Vec::with_capacity(self.chunk.prototypes.len() + 1);
        let mut memory_stack = Vec::with_capacity(self.chunk.prototypes.len() + 1);

        let mut call = CallFrame::new(Arc::clone(&self.chunk), Address::default());
        let mut memory = Memory::new(&call.chunk);

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

                            if to.is_register {
                                memory.registers.booleans[to.index as usize] = boolean;
                            } else {
                                *memory
                                    .booleans
                                    .get_mut(to.index as usize)
                                    .unwrap()
                                    .as_value_mut() = boolean;
                            }
                        }
                        AddressKind::BOOLEAN_REGISTER => {
                            let boolean = memory.registers.booleans[from.index as usize];

                            if to.is_register {
                                memory.registers.booleans[to.index as usize] = boolean;
                            } else {
                                *memory
                                    .booleans
                                    .get_mut(to.index as usize)
                                    .unwrap()
                                    .as_value_mut() = boolean;
                            }
                        }
                        AddressKind::BYTE_MEMORY => {
                            let byte = *memory.bytes.get(from.index as usize).unwrap().as_value();

                            if to.is_register {
                                memory.registers.bytes[to.index as usize] = byte;
                            } else {
                                *memory
                                    .bytes
                                    .get_mut(to.index as usize)
                                    .unwrap()
                                    .as_value_mut() = byte;
                            }
                        }
                        AddressKind::BYTE_REGISTER => {
                            let byte = memory.registers.bytes[from.index as usize];

                            if to.is_register {
                                memory.registers.bytes[to.index as usize] = byte;
                            } else {
                                *memory
                                    .bytes
                                    .get_mut(to.index as usize)
                                    .unwrap()
                                    .as_value_mut() = byte;
                            }
                        }
                        AddressKind::CHARACTER_MEMORY => {
                            let character = *memory
                                .characters
                                .get(from.index as usize)
                                .unwrap()
                                .as_value();

                            if to.is_register {
                                memory.registers.characters[to.index as usize] = character;
                            } else {
                                *memory
                                    .characters
                                    .get_mut(to.index as usize)
                                    .unwrap()
                                    .as_value_mut() = character;
                            }
                        }
                        AddressKind::CHARACTER_REGISTER => {
                            let character = memory.registers.characters[from.index as usize];

                            if to.is_register {
                                memory.registers.characters[to.index as usize] = character;
                            } else {
                                *memory
                                    .characters
                                    .get_mut(to.index as usize)
                                    .unwrap()
                                    .as_value_mut() = character;
                            }
                        }
                        AddressKind::FLOAT_MEMORY => {
                            let float = *memory.floats.get(from.index as usize).unwrap().as_value();

                            if to.is_register {
                                memory.registers.floats[to.index as usize] = float;
                            } else {
                                *memory
                                    .floats
                                    .get_mut(to.index as usize)
                                    .unwrap()
                                    .as_value_mut() = float;
                            }
                        }
                        AddressKind::FLOAT_REGISTER => {
                            let float = memory.registers.floats[from.index as usize];

                            if to.is_register {
                                memory.registers.floats[to.index as usize] = float;
                            } else {
                                *memory
                                    .floats
                                    .get_mut(to.index as usize)
                                    .unwrap()
                                    .as_value_mut() = float;
                            }
                        }
                        AddressKind::INTEGER_MEMORY => {
                            let integer =
                                *memory.integers.get(from.index as usize).unwrap().as_value();

                            if to.is_register {
                                memory.registers.integers[to.index as usize] = integer;
                            } else {
                                *memory
                                    .integers
                                    .get_mut(to.index as usize)
                                    .unwrap()
                                    .as_value_mut() = integer;
                            }
                        }
                        AddressKind::INTEGER_REGISTER => {
                            let integer = memory.registers.integers[from.index as usize];

                            if to.is_register {
                                memory.registers.integers[to.index as usize] = integer;
                            } else {
                                *memory
                                    .integers
                                    .get_mut(to.index as usize)
                                    .unwrap()
                                    .as_value_mut() = integer;
                            }
                        }
                        AddressKind::STRING_MEMORY => {
                            let string = memory.strings[from.index as usize].as_value().clone();

                            if to.is_register {
                                memory.registers.strings[to.index as usize] = string;
                            } else {
                                *memory
                                    .strings
                                    .get_mut(to.index as usize)
                                    .unwrap()
                                    .as_value_mut() = string;
                            }
                        }
                        AddressKind::STRING_REGISTER => {
                            let string = memory.registers.strings[from.index as usize].clone();

                            if to.is_register {
                                memory.registers.strings[to.index as usize] = string;
                            } else {
                                *memory
                                    .strings
                                    .get_mut(to.index as usize)
                                    .unwrap()
                                    .as_value_mut() = string;
                            }
                        }
                        AddressKind::LIST_MEMORY => {
                            let list = memory.lists[from.index as usize].as_value().clone();

                            if to.is_register {
                                memory.registers.lists[to.index as usize] = list;
                            } else {
                                *memory
                                    .lists
                                    .get_mut(to.index as usize)
                                    .unwrap()
                                    .as_value_mut() = list;
                            }
                        }
                        AddressKind::LIST_REGISTER => {
                            let list = memory.registers.lists[from.index as usize].clone();

                            if to.is_register {
                                memory.registers.lists[to.index as usize] = list;
                            } else {
                                *memory
                                    .lists
                                    .get_mut(to.index as usize)
                                    .unwrap()
                                    .as_value_mut() = list;
                            }
                        }
                        AddressKind::FUNCTION_MEMORY => {
                            let function = memory.functions[from.index as usize].clone_value();

                            if to.is_register {
                                memory.registers.functions[to.index as usize] = function;
                            } else {
                                *memory
                                    .functions
                                    .get_mut(to.index as usize)
                                    .unwrap()
                                    .as_value_mut() = function;
                            }
                        }
                        AddressKind::FUNCTION_REGISTER => {
                            let function =
                                Arc::clone(&memory.registers.functions[from.index as usize]);

                            if to.is_register {
                                memory.registers.functions[to.index as usize] = function;
                            } else {
                                *memory
                                    .functions
                                    .get_mut(to.index as usize)
                                    .unwrap()
                                    .as_value_mut() = function;
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                Operation::CLOSE => {
                    let Close { from, to } = Close::from(&instruction);

                    match from.kind {
                        AddressKind::BOOLEAN_MEMORY => {
                            for i in from.index as usize..=to.index as usize {
                                memory.booleans[i].close();
                            }
                        }
                        AddressKind::BYTE_MEMORY => {
                            for i in from.index as usize..=to.index as usize {
                                memory.bytes[i].close();
                            }
                        }
                        AddressKind::CHARACTER_MEMORY => {
                            for i in from.index as usize..=to.index as usize {
                                memory.characters[i].close();
                            }
                        }
                        AddressKind::FLOAT_MEMORY => {
                            for i in from.index as usize..=to.index as usize {
                                memory.floats[i].close();
                            }
                        }
                        AddressKind::INTEGER_MEMORY => {
                            for i in from.index as usize..=to.index as usize {
                                memory.integers[i].close();
                            }
                        }
                        AddressKind::STRING_MEMORY => {
                            for i in from.index as usize..=to.index as usize {
                                memory.strings[i].close();
                            }
                        }
                        AddressKind::LIST_MEMORY => {
                            for i in from.index as usize..=to.index as usize {
                                memory.lists[i].close();
                            }
                        }
                        AddressKind::FUNCTION_MEMORY => {
                            for i in from.index as usize..=to.index as usize {
                                memory.functions[i].close();
                            }
                        }
                        _ => unreachable!(),
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
                                *memory.booleans[destination.index as usize].as_value_mut() =
                                    boolean;
                            }
                        }
                        AddressKind::BYTE_MEMORY => {
                            let byte = value as u8;
                            if destination.is_register {
                                memory.registers.bytes[destination.index as usize] = byte;
                            } else {
                                *memory.bytes[destination.index as usize].as_value_mut() = byte;
                            }
                        }
                        _ => unreachable!(),
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

                                *memory.characters[destination_index].as_value_mut() = value;
                            }
                        }
                        AddressKind::FLOAT_CONSTANT => {
                            let value = call.chunk.float_constants[constant_index];

                            if destination.is_register {
                                memory.registers.floats[destination.index as usize] = value;
                            } else {
                                let destination_index = destination.index as usize;

                                *memory.floats[destination_index].as_value_mut() = value;
                            }
                        }
                        AddressKind::INTEGER_CONSTANT => {
                            let value = call.chunk.integer_constants[constant_index];

                            if destination.is_register {
                                memory.registers.integers[destination.index as usize] = value;
                            } else {
                                let destination_index = destination.index as usize;

                                *memory.integers[destination_index].as_value_mut() = value;
                            }
                        }
                        AddressKind::STRING_CONSTANT => {
                            let value = call.chunk.string_constants[constant_index].clone();

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
                            let chunk = &call.chunk.prototypes[prototype_address.index as usize];

                            Arc::clone(chunk)
                        }
                        AddressKind::FUNCTION_SELF => Arc::clone(&call.chunk),
                        _ => unreachable!(),
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
                        item_pointers: Vec::with_capacity(end as usize - start.index as usize),
                    };

                    for i in start.index as usize..=end as usize {
                        match start.kind {
                            AddressKind::BOOLEAN_MEMORY => {
                                if memory.booleans[i].is_closed() {
                                    continue;
                                }
                            }
                            AddressKind::BYTE_MEMORY => {
                                if memory.bytes[i].is_closed() {
                                    continue;
                                }
                            }
                            AddressKind::CHARACTER_MEMORY => {
                                if memory.characters[i].is_closed() {
                                    continue;
                                }
                            }
                            AddressKind::FLOAT_MEMORY => {
                                if memory.floats[i].is_closed() {
                                    continue;
                                }
                            }
                            AddressKind::INTEGER_MEMORY => {
                                if memory.integers[i].is_closed() {
                                    continue;
                                }
                            }
                            AddressKind::STRING_MEMORY => {
                                if memory.strings[i].is_closed() {
                                    continue;
                                }
                            }
                            AddressKind::LIST_MEMORY => {
                                if memory.lists[i].is_closed() {
                                    continue;
                                }
                            }
                            AddressKind::FUNCTION_MEMORY => {
                                if memory.functions[i].is_closed() {
                                    continue;
                                }
                            }
                            _ => unreachable!(),
                        }

                        let pointer = Address::new(i as u16, start.kind);

                        abstract_list.item_pointers.push(pointer);
                    }

                    if destination.is_register {
                        memory.registers.lists[destination.index as usize] = abstract_list;
                    } else {
                        *memory.lists[destination.index as usize].as_value_mut() = abstract_list;
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
                    let left_index = left.index as usize;

                    match left.kind {
                        AddressKind::BYTE_MEMORY => {
                            let left_value = *memory.bytes[left_index].as_value();
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => {
                                    *memory.bytes[right.index as usize].as_value()
                                }
                                AddressKind::BYTE_REGISTER => {
                                    memory.registers.bytes[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let sum = left_value + right_value;

                            if destination.is_register {
                                memory.registers.bytes[destination.index as usize] = sum;
                            } else {
                                *memory.bytes[destination.index as usize].as_value_mut() = sum;
                            }
                        }
                        AddressKind::BYTE_REGISTER => {
                            let left_value = memory.registers.bytes[left_index];
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => {
                                    *memory.bytes[right.index as usize].as_value()
                                }
                                AddressKind::BYTE_REGISTER => {
                                    memory.registers.bytes[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let sum = left_value + right_value;

                            if destination.is_register {
                                memory.registers.bytes[destination.index as usize] = sum;
                            } else {
                                *memory.bytes[destination.index as usize].as_value_mut() = sum;
                            }
                        }
                        AddressKind::CHARACTER_CONSTANT => {
                            let left_value = call.chunk.character_constants[left_index];
                            let right_value = match right.kind {
                                AddressKind::CHARACTER_CONSTANT => {
                                    call.chunk.character_constants[right.index as usize]
                                }
                                AddressKind::CHARACTER_MEMORY => {
                                    *memory.characters[right.index as usize].as_value()
                                }
                                AddressKind::CHARACTER_REGISTER => {
                                    memory.registers.characters[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let mut sum = DustString::new();

                            sum.push(left_value);
                            sum.push(right_value);

                            if destination.is_register {
                                memory.registers.strings[destination.index as usize] = sum;
                            } else {
                                *memory.strings[destination.index as usize].as_value_mut() = sum;
                            }
                        }
                        AddressKind::CHARACTER_MEMORY => {
                            let left_value = *memory.characters[left_index].as_value();
                            let right_value = match right.kind {
                                AddressKind::CHARACTER_CONSTANT => {
                                    call.chunk.character_constants[right.index as usize]
                                }
                                AddressKind::CHARACTER_MEMORY => {
                                    *memory.characters[right.index as usize].as_value()
                                }
                                AddressKind::CHARACTER_REGISTER => {
                                    memory.registers.characters[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let mut sum = DustString::new();

                            sum.push(left_value);
                            sum.push(right_value);

                            if destination.is_register {
                                memory.registers.strings[destination.index as usize] = sum;
                            } else {
                                *memory.strings[destination.index as usize].as_value_mut() = sum;
                            }
                        }
                        AddressKind::CHARACTER_REGISTER => {
                            let left_value = memory.registers.characters[left_index];
                            let right_index = right.index as usize;
                            let right_value = match right.kind {
                                AddressKind::CHARACTER_CONSTANT => {
                                    call.chunk.character_constants[right_index]
                                }
                                AddressKind::CHARACTER_MEMORY => {
                                    *memory.characters[right.index as usize].as_value()
                                }
                                AddressKind::CHARACTER_REGISTER => {
                                    memory.registers.characters[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let mut sum = DustString::new();

                            sum.push(left_value);
                            sum.push(right_value);

                            if destination.is_register {
                                memory.registers.strings[destination.index as usize] = sum;
                            } else {
                                *memory.strings[destination.index as usize].as_value_mut() = sum;
                            }
                        }
                        AddressKind::FLOAT_CONSTANT => {
                            let left_value = call.chunk.float_constants[left_index];
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    call.chunk.float_constants[right.index as usize]
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    *memory.floats[right.index as usize].as_value()
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    memory.registers.floats[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let sum = left_value + right_value;

                            if destination.is_register {
                                memory.registers.floats[destination.index as usize] = sum;
                            } else {
                                *memory.floats[destination.index as usize].as_value_mut() = sum;
                            }
                        }
                        AddressKind::FLOAT_MEMORY => {
                            let left_value = *memory.floats[left_index].as_value();
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    call.chunk.float_constants[right.index as usize]
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    *memory.floats[right.index as usize].as_value()
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    memory.registers.floats[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let sum = left_value + right_value;

                            if destination.is_register {
                                memory.registers.floats[destination.index as usize] = sum;
                            } else {
                                *memory.floats[destination.index as usize].as_value_mut() = sum;
                            }
                        }
                        AddressKind::FLOAT_REGISTER => {
                            let left_value = memory.registers.floats[left_index];
                            let right_index = right.index as usize;
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    call.chunk.float_constants[right_index]
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    *memory.floats[right.index as usize].as_value()
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    memory.registers.floats[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let sum = left_value + right_value;

                            if destination.is_register {
                                memory.registers.floats[destination.index as usize] = sum;
                            } else {
                                *memory.floats[destination.index as usize].as_value_mut() = sum;
                            }
                        }
                        AddressKind::INTEGER_CONSTANT => {
                            let left_value = call.chunk.integer_constants[left_index];
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    call.chunk.integer_constants[right.index as usize]
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
                            let left_value = memory.integers[left_index].as_value();
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    call.chunk.integer_constants[right.index as usize]
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
                            let left_value = memory.registers.integers[left_index];
                            let right_index = right.index as usize;
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    call.chunk.integer_constants[right_index]
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
                        AddressKind::STRING_CONSTANT => {
                            let left_value = call.chunk.string_constants[left_index].clone();
                            let right_value = match right.kind {
                                AddressKind::STRING_CONSTANT => {
                                    call.chunk.string_constants[right.index as usize].clone()
                                }
                                AddressKind::STRING_MEMORY => {
                                    memory.strings[right.index as usize].as_value().clone()
                                }
                                AddressKind::STRING_REGISTER => {
                                    memory.registers.strings[right.index as usize].clone()
                                }
                                _ => unreachable!(),
                            };
                            let mut sum = DustString::new();

                            sum.push_str(&left_value);
                            sum.push_str(&right_value);

                            if destination.is_register {
                                memory.registers.strings[destination.index as usize] = sum;
                            } else {
                                *memory.strings[destination.index as usize].as_value_mut() = sum;
                            }
                        }
                        AddressKind::STRING_MEMORY => {
                            let left_value = memory.strings[left_index].as_value().clone();
                            let right_value = match right.kind {
                                AddressKind::STRING_CONSTANT => {
                                    call.chunk.string_constants[right.index as usize].clone()
                                }
                                AddressKind::STRING_MEMORY => {
                                    memory.strings[right.index as usize].as_value().clone()
                                }
                                AddressKind::STRING_REGISTER => {
                                    memory.registers.strings[right.index as usize].clone()
                                }
                                _ => unreachable!(),
                            };
                            let mut sum = DustString::new();

                            sum.push_str(&left_value);
                            sum.push_str(&right_value);

                            if destination.is_register {
                                memory.registers.strings[destination.index as usize] = sum;
                            } else {
                                *memory.strings[destination.index as usize].as_value_mut() = sum;
                            }
                        }
                        AddressKind::STRING_REGISTER => {
                            let left_value = memory.registers.strings[left_index].clone();
                            let right_index = right.index as usize;
                            let right_value = match right.kind {
                                AddressKind::STRING_CONSTANT => {
                                    call.chunk.string_constants[right_index].clone()
                                }
                                AddressKind::STRING_MEMORY => {
                                    memory.strings[right.index as usize].as_value().clone()
                                }
                                AddressKind::STRING_REGISTER => {
                                    memory.registers.strings[right.index as usize].clone()
                                }
                                _ => unreachable!(),
                            };
                            let mut sum = DustString::new();

                            sum.push_str(&left_value);
                            sum.push_str(&right_value);

                            if destination.is_register {
                                memory.registers.strings[destination.index as usize] = sum;
                            } else {
                                *memory.strings[destination.index as usize].as_value_mut() = sum;
                            }
                        }
                        _ => todo!(),
                    }
                }
                Operation::SUBTRACT => {
                    let Subtract {
                        destination,
                        left,
                        right,
                    } = Subtract::from(&instruction);
                    let left_index = left.index as usize;

                    match left.kind {
                        AddressKind::BYTE_MEMORY => {
                            let left_value = *memory.bytes[left_index].as_value();
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => {
                                    *memory.bytes[right.index as usize].as_value()
                                }
                                AddressKind::BYTE_REGISTER => {
                                    memory.registers.bytes[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let difference = left_value - right_value;

                            if destination.is_register {
                                memory.registers.bytes[destination.index as usize] = difference;
                            } else {
                                *memory.bytes[destination.index as usize].as_value_mut() =
                                    difference;
                            }
                        }
                        AddressKind::BYTE_REGISTER => {
                            let left_value = memory.registers.bytes[left_index];
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => {
                                    *memory.bytes[right.index as usize].as_value()
                                }
                                AddressKind::BYTE_REGISTER => {
                                    memory.registers.bytes[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let difference = left_value - right_value;

                            if destination.is_register {
                                memory.registers.bytes[destination.index as usize] = difference;
                            } else {
                                *memory.bytes[destination.index as usize].as_value_mut() =
                                    difference;
                            }
                        }
                        AddressKind::FLOAT_CONSTANT => {
                            let left_value = call.chunk.float_constants[left_index];
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    call.chunk.float_constants[right.index as usize]
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    *memory.floats[right.index as usize].as_value()
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    memory.registers.floats[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let difference = left_value - right_value;

                            if destination.is_register {
                                memory.registers.floats[destination.index as usize] = difference;
                            } else {
                                *memory.floats[destination.index as usize].as_value_mut() =
                                    difference;
                            }
                        }
                        AddressKind::FLOAT_MEMORY => {
                            let left_value = *memory.floats[left_index].as_value();
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    call.chunk.float_constants[right.index as usize]
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    *memory.floats[right.index as usize].as_value()
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    memory.registers.floats[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let difference = left_value - right_value;

                            if destination.is_register {
                                memory.registers.floats[destination.index as usize] = difference;
                            } else {
                                *memory.floats[destination.index as usize].as_value_mut() =
                                    difference;
                            }
                        }
                        AddressKind::FLOAT_REGISTER => {
                            let left_value = memory.registers.floats[left_index];
                            let right_index = right.index as usize;
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    call.chunk.float_constants[right_index]
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    *memory.floats[right.index as usize].as_value()
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    memory.registers.floats[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let difference = left_value - right_value;

                            if destination.is_register {
                                memory.registers.floats[destination.index as usize] = difference;
                            } else {
                                *memory.floats[destination.index as usize].as_value_mut() =
                                    difference;
                            }
                        }
                        AddressKind::INTEGER_CONSTANT => {
                            let left_value = call.chunk.integer_constants[left_index];
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    call.chunk.integer_constants[right.index as usize]
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    *memory.integers[right.index as usize].as_value()
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    memory.registers.integers[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let difference = left_value - right_value;

                            if destination.is_register {
                                memory.registers.integers[destination.index as usize] = difference;
                            } else {
                                *memory.integers[destination.index as usize].as_value_mut() =
                                    difference;
                            }
                        }
                        AddressKind::INTEGER_MEMORY => {
                            let left_value = memory.integers[left_index].as_value();
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    call.chunk.integer_constants[right.index as usize]
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    *memory.integers[right.index as usize].as_value()
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    memory.registers.integers[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let difference = left_value - right_value;

                            if destination.is_register {
                                memory.registers.integers[destination.index as usize] = difference;
                            } else {
                                *memory.integers[destination.index as usize].as_value_mut() =
                                    difference;
                            }
                        }
                        AddressKind::INTEGER_REGISTER => {
                            let left_value = memory.registers.integers[left_index];
                            let right_index = right.index as usize;
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    call.chunk.integer_constants[right_index]
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    *memory.integers[right.index as usize].as_value()
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    memory.registers.integers[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let difference = left_value - right_value;

                            if destination.is_register {
                                memory.registers.integers[destination.index as usize] = difference;
                            } else {
                                *memory.integers[destination.index as usize].as_value_mut() =
                                    difference;
                            }
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
                    let left_index = left.index as usize;

                    match left.kind {
                        AddressKind::BYTE_MEMORY => {
                            let left_value = *memory.bytes[left_index].as_value();
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => {
                                    *memory.bytes[right.index as usize].as_value()
                                }
                                AddressKind::BYTE_REGISTER => {
                                    memory.registers.bytes[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let product = left_value * right_value;

                            if destination.is_register {
                                memory.registers.bytes[destination.index as usize] = product;
                            } else {
                                *memory.bytes[destination.index as usize].as_value_mut() = product;
                            }
                        }
                        AddressKind::BYTE_REGISTER => {
                            let left_value = memory.registers.bytes[left_index];
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => {
                                    *memory.bytes[right.index as usize].as_value()
                                }
                                AddressKind::BYTE_REGISTER => {
                                    memory.registers.bytes[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let product = left_value * right_value;

                            if destination.is_register {
                                memory.registers.bytes[destination.index as usize] = product;
                            } else {
                                *memory.bytes[destination.index as usize].as_value_mut() = product;
                            }
                        }
                        AddressKind::FLOAT_CONSTANT => {
                            let left_value = call.chunk.float_constants[left_index];
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    call.chunk.float_constants[right.index as usize]
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    *memory.floats[right.index as usize].as_value()
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    memory.registers.floats[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let product = left_value * right_value;

                            if destination.is_register {
                                memory.registers.floats[destination.index as usize] = product;
                            } else {
                                *memory.floats[destination.index as usize].as_value_mut() = product;
                            }
                        }
                        AddressKind::FLOAT_MEMORY => {
                            let left_value = *memory.floats[left_index].as_value();
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    call.chunk.float_constants[right.index as usize]
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    *memory.floats[right.index as usize].as_value()
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    memory.registers.floats[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let product = left_value * right_value;

                            if destination.is_register {
                                memory.registers.floats[destination.index as usize] = product;
                            } else {
                                *memory.floats[destination.index as usize].as_value_mut() = product;
                            }
                        }
                        AddressKind::FLOAT_REGISTER => {
                            let left_value = memory.registers.floats[left_index];
                            let right_index = right.index as usize;
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    call.chunk.float_constants[right_index]
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    *memory.floats[right.index as usize].as_value()
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    memory.registers.floats[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let product = left_value * right_value;

                            if destination.is_register {
                                memory.registers.floats[destination.index as usize] = product;
                            } else {
                                *memory.floats[destination.index as usize].as_value_mut() = product;
                            }
                        }
                        AddressKind::INTEGER_CONSTANT => {
                            let left_value = call.chunk.integer_constants[left_index];
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    call.chunk.integer_constants[right.index as usize]
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    *memory.integers[right.index as usize].as_value()
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    memory.registers.integers[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let product = left_value * right_value;

                            if destination.is_register {
                                memory.registers.integers[destination.index as usize] = product;
                            } else {
                                *memory.integers[destination.index as usize].as_value_mut() =
                                    product;
                            }
                        }
                        AddressKind::INTEGER_MEMORY => {
                            let left_value = memory.integers[left_index].as_value();
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    call.chunk.integer_constants[right.index as usize]
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    *memory.integers[right.index as usize].as_value()
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    memory.registers.integers[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let product = left_value * right_value;

                            if destination.is_register {
                                memory.registers.integers[destination.index as usize] = product;
                            } else {
                                *memory.integers[destination.index as usize].as_value_mut() =
                                    product;
                            }
                        }
                        AddressKind::INTEGER_REGISTER => {
                            let left_value = memory.registers.integers[left_index];
                            let right_index = right.index as usize;
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    call.chunk.integer_constants[right_index]
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    *memory.integers[right.index as usize].as_value()
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    memory.registers.integers[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let product = left_value * right_value;

                            if destination.is_register {
                                memory.registers.integers[destination.index as usize] = product;
                            } else {
                                *memory.integers[destination.index as usize].as_value_mut() =
                                    product;
                            }
                        }
                        _ => todo!(),
                    }
                }
                Operation::DIVIDE => {
                    let Divide {
                        destination,
                        left,
                        right,
                    } = Divide::from(&instruction);
                    let left_index = left.index as usize;

                    match left.kind {
                        AddressKind::BYTE_MEMORY => {
                            let left_value = *memory.bytes[left_index].as_value();
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => {
                                    *memory.bytes[right.index as usize].as_value()
                                }
                                AddressKind::BYTE_REGISTER => {
                                    memory.registers.bytes[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let quotient = left_value / right_value;

                            if destination.is_register {
                                memory.registers.bytes[destination.index as usize] = quotient;
                            } else {
                                *memory.bytes[destination.index as usize].as_value_mut() = quotient;
                            }
                        }
                        AddressKind::BYTE_REGISTER => {
                            let left_value = memory.registers.bytes[left_index];
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => {
                                    *memory.bytes[right.index as usize].as_value()
                                }
                                AddressKind::BYTE_REGISTER => {
                                    memory.registers.bytes[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let quotient = left_value / right_value;

                            if destination.is_register {
                                memory.registers.bytes[destination.index as usize] = quotient;
                            } else {
                                *memory.bytes[destination.index as usize].as_value_mut() = quotient;
                            }
                        }
                        AddressKind::FLOAT_CONSTANT => {
                            let left_value = call.chunk.float_constants[left_index];
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    call.chunk.float_constants[right.index as usize]
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    *memory.floats[right.index as usize].as_value()
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    memory.registers.floats[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let quotient = left_value / right_value;

                            if destination.is_register {
                                memory.registers.floats[destination.index as usize] = quotient;
                            } else {
                                *memory.floats[destination.index as usize].as_value_mut() =
                                    quotient;
                            }
                        }
                        AddressKind::FLOAT_MEMORY => {
                            let left_value = *memory.floats[left_index].as_value();
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    call.chunk.float_constants[right.index as usize]
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    *memory.floats[right.index as usize].as_value()
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    memory.registers.floats[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let quotient = left_value / right_value;

                            if destination.is_register {
                                memory.registers.floats[destination.index as usize] = quotient;
                            } else {
                                *memory.floats[destination.index as usize].as_value_mut() =
                                    quotient;
                            }
                        }
                        AddressKind::FLOAT_REGISTER => {
                            let left_value = memory.registers.floats[left_index];
                            let right_index = right.index as usize;
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    call.chunk.float_constants[right_index]
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    *memory.floats[right.index as usize].as_value()
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    memory.registers.floats[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let quotient = left_value / right_value;

                            if destination.is_register {
                                memory.registers.floats[destination.index as usize] = quotient;
                            } else {
                                *memory.floats[destination.index as usize].as_value_mut() =
                                    quotient;
                            }
                        }
                        AddressKind::INTEGER_CONSTANT => {
                            let left_value = call.chunk.integer_constants[left_index];
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    call.chunk.integer_constants[right.index as usize]
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    *memory.integers[right.index as usize].as_value()
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    memory.registers.integers[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let quotient = left_value / right_value;

                            if destination.is_register {
                                memory.registers.integers[destination.index as usize] = quotient;
                            } else {
                                *memory.integers[destination.index as usize].as_value_mut() =
                                    quotient;
                            }
                        }
                        AddressKind::INTEGER_MEMORY => {
                            let left_value = memory.integers[left_index].as_value();
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    call.chunk.integer_constants[right.index as usize]
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    *memory.integers[right.index as usize].as_value()
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    memory.registers.integers[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let quotient = left_value / right_value;

                            if destination.is_register {
                                memory.registers.integers[destination.index as usize] = quotient;
                            } else {
                                *memory.integers[destination.index as usize].as_value_mut() =
                                    quotient;
                            }
                        }
                        AddressKind::INTEGER_REGISTER => {
                            let left_value = memory.registers.integers[left_index];
                            let right_index = right.index as usize;
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    call.chunk.integer_constants[right_index]
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    *memory.integers[right.index as usize].as_value()
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    memory.registers.integers[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let quotient = left_value / right_value;

                            if destination.is_register {
                                memory.registers.integers[destination.index as usize] = quotient;
                            } else {
                                *memory.integers[destination.index as usize].as_value_mut() =
                                    quotient;
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                Operation::MODULO => {
                    let Modulo {
                        destination,
                        left,
                        right,
                    } = Modulo::from(&instruction);
                    let left_index = left.index as usize;

                    match left.kind {
                        AddressKind::BYTE_MEMORY => {
                            let left_value = *memory.bytes[left_index].as_value();
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => {
                                    *memory.bytes[right.index as usize].as_value()
                                }
                                AddressKind::BYTE_REGISTER => {
                                    memory.registers.bytes[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let remainder = left_value % right_value;

                            if destination.is_register {
                                memory.registers.bytes[destination.index as usize] = remainder;
                            } else {
                                *memory.bytes[destination.index as usize].as_value_mut() =
                                    remainder;
                            }
                        }
                        AddressKind::BYTE_REGISTER => {
                            let left_value = memory.registers.bytes[left_index];
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => {
                                    *memory.bytes[right.index as usize].as_value()
                                }
                                AddressKind::BYTE_REGISTER => {
                                    memory.registers.bytes[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let remainder = left_value % right_value;

                            if destination.is_register {
                                memory.registers.bytes[destination.index as usize] = remainder;
                            } else {
                                *memory.bytes[destination.index as usize].as_value_mut() =
                                    remainder;
                            }
                        }
                        AddressKind::FLOAT_CONSTANT => {
                            let left_value = call.chunk.float_constants[left_index];
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    call.chunk.float_constants[right.index as usize]
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    *memory.floats[right.index as usize].as_value()
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    memory.registers.floats[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let remainder = left_value % right_value;

                            if destination.is_register {
                                memory.registers.floats[destination.index as usize] = remainder;
                            } else {
                                *memory.floats[destination.index as usize].as_value_mut() =
                                    remainder;
                            }
                        }
                        AddressKind::FLOAT_MEMORY => {
                            let left_value = *memory.floats[left_index].as_value();
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    call.chunk.float_constants[right.index as usize]
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    *memory.floats[right.index as usize].as_value()
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    memory.registers.floats[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let remainder = left_value % right_value;

                            if destination.is_register {
                                memory.registers.floats[destination.index as usize] = remainder;
                            } else {
                                *memory.floats[destination.index as usize].as_value_mut() =
                                    remainder;
                            }
                        }
                        AddressKind::FLOAT_REGISTER => {
                            let left_value = memory.registers.floats[left_index];
                            let right_index = right.index as usize;
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    call.chunk.float_constants[right_index]
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    *memory.floats[right.index as usize].as_value()
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    memory.registers.floats[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let remainder = left_value % right_value;

                            if destination.is_register {
                                memory.registers.floats[destination.index as usize] = remainder;
                            } else {
                                *memory.floats[destination.index as usize].as_value_mut() =
                                    remainder;
                            }
                        }
                        AddressKind::INTEGER_CONSTANT => {
                            let left_value = call.chunk.integer_constants[left_index];
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    call.chunk.integer_constants[right.index as usize]
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    *memory.integers[right.index as usize].as_value()
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    memory.registers.integers[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let remainder = left_value % right_value;

                            if destination.is_register {
                                memory.registers.integers[destination.index as usize] = remainder;
                            } else {
                                *memory.integers[destination.index as usize].as_value_mut() =
                                    remainder;
                            }
                        }
                        AddressKind::INTEGER_MEMORY => {
                            let left_value = memory.integers[left_index].as_value();
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    call.chunk.integer_constants[right.index as usize]
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    *memory.integers[right.index as usize].as_value()
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    memory.registers.integers[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let remainder = left_value % right_value;

                            if destination.is_register {
                                memory.registers.integers[destination.index as usize] = remainder;
                            } else {
                                *memory.integers[destination.index as usize].as_value_mut() =
                                    remainder;
                            }
                        }
                        AddressKind::INTEGER_REGISTER => {
                            let left_value = memory.registers.integers[left_index];
                            let right_index = right.index as usize;
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    call.chunk.integer_constants[right_index]
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    *memory.integers[right.index as usize].as_value()
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    memory.registers.integers[right.index as usize]
                                }
                                _ => unreachable!(),
                            };
                            let remainder = left_value % right_value;

                            if destination.is_register {
                                memory.registers.integers[destination.index as usize] = remainder;
                            } else {
                                *memory.integers[destination.index as usize].as_value_mut() =
                                    remainder;
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                Operation::EQUAL => {
                    let Equal {
                        comparator,
                        left,
                        right,
                    } = Equal::from(&instruction);

                    let left_index = left.index as usize;
                    let is_equal = match left.kind {
                        AddressKind::BOOLEAN_MEMORY => {
                            let left_value = *memory.booleans[left_index].as_value();
                            let right_value = match right.kind {
                                AddressKind::BOOLEAN_MEMORY => {
                                    *memory.booleans[right.index as usize].as_value()
                                }
                                AddressKind::BOOLEAN_REGISTER => {
                                    memory.registers.booleans[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value == right_value
                        }
                        AddressKind::BOOLEAN_REGISTER => {
                            let left_value = memory.registers.booleans[left_index];
                            let right_value = match right.kind {
                                AddressKind::BOOLEAN_MEMORY => {
                                    *memory.booleans[right.index as usize].as_value()
                                }
                                AddressKind::BOOLEAN_REGISTER => {
                                    memory.registers.booleans[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value == right_value
                        }
                        AddressKind::BYTE_MEMORY => {
                            let left_value = *memory.bytes[left_index].as_value();
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => {
                                    *memory.bytes[right.index as usize].as_value()
                                }
                                AddressKind::BYTE_REGISTER => {
                                    memory.registers.bytes[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value == right_value
                        }
                        AddressKind::BYTE_REGISTER => {
                            let left_value = memory.registers.bytes[left_index];
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => {
                                    *memory.bytes[right.index as usize].as_value()
                                }
                                AddressKind::BYTE_REGISTER => {
                                    memory.registers.bytes[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value == right_value
                        }
                        AddressKind::CHARACTER_CONSTANT => {
                            let left_value = call.chunk.character_constants[left_index];
                            let right_value = match right.kind {
                                AddressKind::CHARACTER_CONSTANT => {
                                    call.chunk.character_constants[right.index as usize]
                                }
                                AddressKind::CHARACTER_MEMORY => {
                                    *memory.characters[right.index as usize].as_value()
                                }
                                AddressKind::CHARACTER_REGISTER => {
                                    memory.registers.characters[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value == right_value
                        }
                        AddressKind::CHARACTER_MEMORY => {
                            let left_value = *memory.characters[left_index].as_value();
                            let right_value = match right.kind {
                                AddressKind::CHARACTER_MEMORY => {
                                    *memory.characters[right.index as usize].as_value()
                                }
                                AddressKind::CHARACTER_REGISTER => {
                                    memory.registers.characters[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value == right_value
                        }
                        AddressKind::CHARACTER_REGISTER => {
                            let left_value = memory.registers.characters[left_index];
                            let right_value = match right.kind {
                                AddressKind::CHARACTER_MEMORY => {
                                    *memory.characters[right.index as usize].as_value()
                                }
                                AddressKind::CHARACTER_REGISTER => {
                                    memory.registers.characters[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value == right_value
                        }
                        AddressKind::FLOAT_CONSTANT => {
                            let left_value = call.chunk.float_constants[left_index];
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    call.chunk.float_constants[right.index as usize]
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    *memory.floats[right.index as usize].as_value()
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    memory.registers.floats[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value == right_value
                        }
                        AddressKind::FLOAT_MEMORY => {
                            let left_value = *memory.floats[left_index].as_value();
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    call.chunk.float_constants[right.index as usize]
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    *memory.floats[right.index as usize].as_value()
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    memory.registers.floats[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value == right_value
                        }
                        AddressKind::FLOAT_REGISTER => {
                            let left_value = memory.registers.floats[left_index];
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    call.chunk.float_constants[right.index as usize]
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    *memory.floats[right.index as usize].as_value()
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    memory.registers.floats[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value == right_value
                        }
                        AddressKind::INTEGER_CONSTANT => {
                            let left_value = call.chunk.integer_constants[left_index];
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    call.chunk.integer_constants[right.index as usize]
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    *memory.integers[right.index as usize].as_value()
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    memory.registers.integers[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value == right_value
                        }
                        AddressKind::INTEGER_MEMORY => {
                            let left_value = *memory.integers[left_index].as_value();
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    call.chunk.integer_constants[right.index as usize]
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    *memory.integers[right.index as usize].as_value()
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    memory.registers.integers[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value == right_value
                        }
                        AddressKind::INTEGER_REGISTER => {
                            let left_value = memory.registers.integers[left_index];
                            let right_index = right.index as usize;
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    call.chunk.integer_constants[right_index]
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    *memory.integers[right.index as usize].as_value()
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    memory.registers.integers[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value == right_value
                        }
                        AddressKind::STRING_CONSTANT => {
                            let left_value = call.chunk.string_constants[left_index].clone();
                            let right_value = match right.kind {
                                AddressKind::STRING_CONSTANT => {
                                    call.chunk.string_constants[right.index as usize].clone()
                                }
                                AddressKind::STRING_MEMORY => {
                                    memory.strings[right.index as usize].clone_value()
                                }
                                AddressKind::STRING_REGISTER => {
                                    memory.registers.strings[right.index as usize].clone()
                                }
                                _ => unreachable!(),
                            };

                            left_value == right_value
                        }
                        AddressKind::STRING_MEMORY => {
                            let left_value = memory.strings[left_index].clone_value();
                            let right_value = match right.kind {
                                AddressKind::STRING_CONSTANT => {
                                    call.chunk.string_constants[right.index as usize].clone()
                                }
                                AddressKind::STRING_MEMORY => {
                                    memory.strings[right.index as usize].clone_value()
                                }
                                AddressKind::STRING_REGISTER => {
                                    memory.registers.strings[right.index as usize].clone()
                                }
                                _ => unreachable!(),
                            };

                            left_value == right_value
                        }
                        AddressKind::STRING_REGISTER => {
                            let left_value = memory.registers.strings[left_index].clone();
                            let right_value = match right.kind {
                                AddressKind::STRING_CONSTANT => {
                                    call.chunk.string_constants[right.index as usize].clone()
                                }
                                AddressKind::STRING_MEMORY => {
                                    memory.strings[right.index as usize].clone_value()
                                }
                                AddressKind::STRING_REGISTER => {
                                    memory.registers.strings[right.index as usize].clone()
                                }
                                _ => unreachable!(),
                            };

                            left_value == right_value
                        }
                        AddressKind::LIST_MEMORY => {
                            let left_value = memory.lists[left_index].clone_value();
                            let right_value = match right.kind {
                                AddressKind::LIST_MEMORY => {
                                    memory.lists[right.index as usize].clone_value()
                                }
                                AddressKind::LIST_REGISTER => {
                                    memory.registers.lists[right.index as usize].clone()
                                }
                                _ => unreachable!(),
                            };

                            left_value == right_value
                        }
                        AddressKind::LIST_REGISTER => {
                            let left_value = memory.registers.lists[left_index].clone();
                            let right_value = match right.kind {
                                AddressKind::LIST_MEMORY => {
                                    memory.lists[right.index as usize].clone_value()
                                }
                                AddressKind::LIST_REGISTER => {
                                    memory.registers.lists[right.index as usize].clone()
                                }
                                _ => unreachable!(),
                            };

                            left_value == right_value
                        }
                        AddressKind::FUNCTION_MEMORY => {
                            let left_value = memory.functions[left_index].clone_value();
                            let right_value = match right.kind {
                                AddressKind::FUNCTION_MEMORY => {
                                    memory.functions[right.index as usize].clone_value()
                                }
                                AddressKind::FUNCTION_REGISTER => {
                                    Arc::clone(&memory.registers.functions[right.index as usize])
                                }
                                _ => unreachable!(),
                            };

                            left_value == right_value
                        }
                        AddressKind::FUNCTION_REGISTER => {
                            let left_value = Arc::clone(&memory.registers.functions[left_index]);
                            let right_value = match right.kind {
                                AddressKind::FUNCTION_MEMORY => {
                                    memory.functions[right.index as usize].clone_value()
                                }
                                AddressKind::FUNCTION_REGISTER => {
                                    Arc::clone(&memory.registers.functions[right.index as usize])
                                }
                                _ => unreachable!(),
                            };

                            left_value == right_value
                        }
                        _ => unreachable!(),
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

                    let left_index = left.index as usize;
                    #[expect(clippy::bool_comparison)]
                    let is_less_than = match left.kind {
                        AddressKind::BOOLEAN_MEMORY => {
                            let left_value = *memory.booleans[left_index].as_value();
                            let right_value = match right.kind {
                                AddressKind::BOOLEAN_MEMORY => {
                                    *memory.booleans[right.index as usize].as_value()
                                }
                                AddressKind::BOOLEAN_REGISTER => {
                                    memory.registers.booleans[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value < right_value
                        }
                        AddressKind::BOOLEAN_REGISTER => {
                            let left_value = memory.registers.booleans[left_index];
                            let right_value = match right.kind {
                                AddressKind::BOOLEAN_MEMORY => {
                                    *memory.booleans[right.index as usize].as_value()
                                }
                                AddressKind::BOOLEAN_REGISTER => {
                                    memory.registers.booleans[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value < right_value
                        }
                        AddressKind::BYTE_MEMORY => {
                            let left_value = *memory.bytes[left_index].as_value();
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => {
                                    *memory.bytes[right.index as usize].as_value()
                                }
                                AddressKind::BYTE_REGISTER => {
                                    memory.registers.bytes[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value < right_value
                        }
                        AddressKind::BYTE_REGISTER => {
                            let left_value = memory.registers.bytes[left_index];
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => {
                                    *memory.bytes[right.index as usize].as_value()
                                }
                                AddressKind::BYTE_REGISTER => {
                                    memory.registers.bytes[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value < right_value
                        }
                        AddressKind::CHARACTER_CONSTANT => {
                            let left_value = call.chunk.character_constants[left_index];
                            let right_value = match right.kind {
                                AddressKind::CHARACTER_CONSTANT => {
                                    call.chunk.character_constants[right.index as usize]
                                }
                                AddressKind::CHARACTER_MEMORY => {
                                    *memory.characters[right.index as usize].as_value()
                                }
                                AddressKind::CHARACTER_REGISTER => {
                                    memory.registers.characters[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value < right_value
                        }
                        AddressKind::CHARACTER_MEMORY => {
                            let left_value = *memory.characters[left_index].as_value();
                            let right_value = match right.kind {
                                AddressKind::CHARACTER_MEMORY => {
                                    *memory.characters[right.index as usize].as_value()
                                }
                                AddressKind::CHARACTER_REGISTER => {
                                    memory.registers.characters[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value < right_value
                        }
                        AddressKind::CHARACTER_REGISTER => {
                            let left_value = memory.registers.characters[left_index];
                            let right_value = match right.kind {
                                AddressKind::CHARACTER_MEMORY => {
                                    *memory.characters[right.index as usize].as_value()
                                }
                                AddressKind::CHARACTER_REGISTER => {
                                    memory.registers.characters[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value < right_value
                        }
                        AddressKind::FLOAT_CONSTANT => {
                            let left_value = call.chunk.float_constants[left_index];
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    call.chunk.float_constants[right.index as usize]
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    *memory.floats[right.index as usize].as_value()
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    memory.registers.floats[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value < right_value
                        }
                        AddressKind::FLOAT_MEMORY => {
                            let left_value = *memory.floats[left_index].as_value();
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    call.chunk.float_constants[right.index as usize]
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    *memory.floats[right.index as usize].as_value()
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    memory.registers.floats[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value < right_value
                        }
                        AddressKind::FLOAT_REGISTER => {
                            let left_value = memory.registers.floats[left_index];
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    call.chunk.float_constants[right.index as usize]
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    *memory.floats[right.index as usize].as_value()
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    memory.registers.floats[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value < right_value
                        }
                        AddressKind::INTEGER_CONSTANT => {
                            let left_value = call.chunk.integer_constants[left_index];
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    call.chunk.integer_constants[right.index as usize]
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
                        AddressKind::INTEGER_MEMORY => {
                            let left_value = *memory.integers[left_index].as_value();
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    call.chunk.integer_constants[right.index as usize]
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
                            let left_value = memory.registers.integers[left_index];
                            let right_index = right.index as usize;
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    call.chunk.integer_constants[right_index]
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
                        AddressKind::STRING_CONSTANT => {
                            let left_value = call.chunk.string_constants[left_index].clone();
                            let right_value = match right.kind {
                                AddressKind::STRING_CONSTANT => {
                                    call.chunk.string_constants[right.index as usize].clone()
                                }
                                AddressKind::STRING_MEMORY => {
                                    memory.strings[right.index as usize].clone_value()
                                }
                                AddressKind::STRING_REGISTER => {
                                    memory.registers.strings[right.index as usize].clone()
                                }
                                _ => unreachable!(),
                            };

                            left_value < right_value
                        }
                        AddressKind::STRING_MEMORY => {
                            let left_value = memory.strings[left_index].clone_value();
                            let right_value = match right.kind {
                                AddressKind::STRING_CONSTANT => {
                                    call.chunk.string_constants[right.index as usize].clone()
                                }
                                AddressKind::STRING_MEMORY => {
                                    memory.strings[right.index as usize].clone_value()
                                }
                                AddressKind::STRING_REGISTER => {
                                    memory.registers.strings[right.index as usize].clone()
                                }
                                _ => unreachable!(),
                            };

                            left_value < right_value
                        }
                        AddressKind::STRING_REGISTER => {
                            let left_value = memory.registers.strings[left_index].clone();
                            let right_value = match right.kind {
                                AddressKind::STRING_CONSTANT => {
                                    call.chunk.string_constants[right.index as usize].clone()
                                }
                                AddressKind::STRING_MEMORY => {
                                    memory.strings[right.index as usize].clone_value()
                                }
                                AddressKind::STRING_REGISTER => {
                                    memory.registers.strings[right.index as usize].clone()
                                }
                                _ => unreachable!(),
                            };

                            left_value < right_value
                        }
                        AddressKind::LIST_MEMORY => {
                            let left_value = memory.lists[left_index].clone_value();
                            let right_value = match right.kind {
                                AddressKind::LIST_MEMORY => {
                                    memory.lists[right.index as usize].clone_value()
                                }
                                AddressKind::LIST_REGISTER => {
                                    memory.registers.lists[right.index as usize].clone()
                                }
                                _ => unreachable!(),
                            };

                            left_value < right_value
                        }
                        AddressKind::LIST_REGISTER => {
                            let left_value = memory.registers.lists[left_index].clone();
                            let right_value = match right.kind {
                                AddressKind::LIST_MEMORY => {
                                    memory.lists[right.index as usize].clone_value()
                                }
                                AddressKind::LIST_REGISTER => {
                                    memory.registers.lists[right.index as usize].clone()
                                }
                                _ => unreachable!(),
                            };

                            left_value < right_value
                        }
                        AddressKind::FUNCTION_MEMORY => {
                            let left_value = memory.functions[left_index].clone_value();
                            let right_value = match right.kind {
                                AddressKind::FUNCTION_MEMORY => {
                                    memory.functions[right.index as usize].clone_value()
                                }
                                AddressKind::FUNCTION_REGISTER => {
                                    Arc::clone(&memory.registers.functions[right.index as usize])
                                }
                                _ => unreachable!(),
                            };

                            left_value < right_value
                        }
                        AddressKind::FUNCTION_REGISTER => {
                            let left_value = Arc::clone(&memory.registers.functions[left_index]);
                            let right_value = match right.kind {
                                AddressKind::FUNCTION_MEMORY => {
                                    memory.functions[right.index as usize].clone_value()
                                }
                                AddressKind::FUNCTION_REGISTER => {
                                    Arc::clone(&memory.registers.functions[right.index as usize])
                                }
                                _ => unreachable!(),
                            };

                            left_value < right_value
                        }
                        _ => unreachable!(),
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

                    let left_index = left.index as usize;
                    let is_less_than_or_equal = match left.kind {
                        AddressKind::BOOLEAN_MEMORY => {
                            let left_value = *memory.booleans[left_index].as_value();
                            let right_value = match right.kind {
                                AddressKind::BOOLEAN_MEMORY => {
                                    *memory.booleans[right.index as usize].as_value()
                                }
                                AddressKind::BOOLEAN_REGISTER => {
                                    memory.registers.booleans[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value <= right_value
                        }
                        AddressKind::BOOLEAN_REGISTER => {
                            let left_value = memory.registers.booleans[left_index];
                            let right_value = match right.kind {
                                AddressKind::BOOLEAN_MEMORY => {
                                    *memory.booleans[right.index as usize].as_value()
                                }
                                AddressKind::BOOLEAN_REGISTER => {
                                    memory.registers.booleans[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value <= right_value
                        }
                        AddressKind::BYTE_MEMORY => {
                            let left_value = *memory.bytes[left_index].as_value();
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => {
                                    *memory.bytes[right.index as usize].as_value()
                                }
                                AddressKind::BYTE_REGISTER => {
                                    memory.registers.bytes[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value <= right_value
                        }
                        AddressKind::BYTE_REGISTER => {
                            let left_value = memory.registers.bytes[left_index];
                            let right_value = match right.kind {
                                AddressKind::BYTE_MEMORY => {
                                    *memory.bytes[right.index as usize].as_value()
                                }
                                AddressKind::BYTE_REGISTER => {
                                    memory.registers.bytes[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value <= right_value
                        }
                        AddressKind::CHARACTER_CONSTANT => {
                            let left_value = call.chunk.character_constants[left_index];
                            let right_value = match right.kind {
                                AddressKind::CHARACTER_CONSTANT => {
                                    call.chunk.character_constants[right.index as usize]
                                }
                                AddressKind::CHARACTER_MEMORY => {
                                    *memory.characters[right.index as usize].as_value()
                                }
                                AddressKind::CHARACTER_REGISTER => {
                                    memory.registers.characters[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value <= right_value
                        }
                        AddressKind::CHARACTER_MEMORY => {
                            let left_value = *memory.characters[left_index].as_value();
                            let right_value = match right.kind {
                                AddressKind::CHARACTER_MEMORY => {
                                    *memory.characters[right.index as usize].as_value()
                                }
                                AddressKind::CHARACTER_REGISTER => {
                                    memory.registers.characters[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value <= right_value
                        }
                        AddressKind::CHARACTER_REGISTER => {
                            let left_value = memory.registers.characters[left_index];
                            let right_value = match right.kind {
                                AddressKind::CHARACTER_MEMORY => {
                                    *memory.characters[right.index as usize].as_value()
                                }
                                AddressKind::CHARACTER_REGISTER => {
                                    memory.registers.characters[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value <= right_value
                        }
                        AddressKind::FLOAT_CONSTANT => {
                            let left_value = call.chunk.float_constants[left_index];
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    call.chunk.float_constants[right.index as usize]
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    *memory.floats[right.index as usize].as_value()
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    memory.registers.floats[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value <= right_value
                        }
                        AddressKind::FLOAT_MEMORY => {
                            let left_value = *memory.floats[left_index].as_value();
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    call.chunk.float_constants[right.index as usize]
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    *memory.floats[right.index as usize].as_value()
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    memory.registers.floats[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value <= right_value
                        }
                        AddressKind::FLOAT_REGISTER => {
                            let left_value = memory.registers.floats[left_index];
                            let right_value = match right.kind {
                                AddressKind::FLOAT_CONSTANT => {
                                    call.chunk.float_constants[right.index as usize]
                                }
                                AddressKind::FLOAT_MEMORY => {
                                    *memory.floats[right.index as usize].as_value()
                                }
                                AddressKind::FLOAT_REGISTER => {
                                    memory.registers.floats[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value <= right_value
                        }
                        AddressKind::INTEGER_CONSTANT => {
                            let left_value = call.chunk.integer_constants[left_index];
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    call.chunk.integer_constants[right.index as usize]
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    *memory.integers[right.index as usize].as_value()
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    memory.registers.integers[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value <= right_value
                        }
                        AddressKind::INTEGER_MEMORY => {
                            let left_value = *memory.integers[left_index].as_value();
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    call.chunk.integer_constants[right.index as usize]
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    *memory.integers[right.index as usize].as_value()
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    memory.registers.integers[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value <= right_value
                        }
                        AddressKind::INTEGER_REGISTER => {
                            let left_value = memory.registers.integers[left_index];
                            let right_index = right.index as usize;
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    call.chunk.integer_constants[right_index]
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    *memory.integers[right.index as usize].as_value()
                                }
                                AddressKind::INTEGER_REGISTER => {
                                    memory.registers.integers[right.index as usize]
                                }
                                _ => unreachable!(),
                            };

                            left_value <= right_value
                        }
                        AddressKind::STRING_CONSTANT => {
                            let left_value = call.chunk.string_constants[left_index].clone();
                            let right_value = match right.kind {
                                AddressKind::STRING_CONSTANT => {
                                    call.chunk.string_constants[right.index as usize].clone()
                                }
                                AddressKind::STRING_MEMORY => {
                                    memory.strings[right.index as usize].clone_value()
                                }
                                AddressKind::STRING_REGISTER => {
                                    memory.registers.strings[right.index as usize].clone()
                                }
                                _ => unreachable!(),
                            };

                            left_value <= right_value
                        }
                        AddressKind::STRING_MEMORY => {
                            let left_value = memory.strings[left_index].clone_value();
                            let right_value = match right.kind {
                                AddressKind::STRING_CONSTANT => {
                                    call.chunk.string_constants[right.index as usize].clone()
                                }
                                AddressKind::STRING_MEMORY => {
                                    memory.strings[right.index as usize].clone_value()
                                }
                                AddressKind::STRING_REGISTER => {
                                    memory.registers.strings[right.index as usize].clone()
                                }
                                _ => unreachable!(),
                            };

                            left_value <= right_value
                        }
                        AddressKind::STRING_REGISTER => {
                            let left_value = memory.registers.strings[left_index].clone();
                            let right_value = match right.kind {
                                AddressKind::STRING_CONSTANT => {
                                    call.chunk.string_constants[right.index as usize].clone()
                                }
                                AddressKind::STRING_MEMORY => {
                                    memory.strings[right.index as usize].clone_value()
                                }
                                AddressKind::STRING_REGISTER => {
                                    memory.registers.strings[right.index as usize].clone()
                                }
                                _ => unreachable!(),
                            };

                            left_value <= right_value
                        }
                        AddressKind::LIST_MEMORY => {
                            let left_value = memory.lists[left_index].clone_value();
                            let right_value = match right.kind {
                                AddressKind::LIST_MEMORY => {
                                    memory.lists[right.index as usize].clone_value()
                                }
                                AddressKind::LIST_REGISTER => {
                                    memory.registers.lists[right.index as usize].clone()
                                }
                                _ => unreachable!(),
                            };

                            left_value <= right_value
                        }
                        AddressKind::LIST_REGISTER => {
                            let left_value = memory.registers.lists[left_index].clone();
                            let right_value = match right.kind {
                                AddressKind::LIST_MEMORY => {
                                    memory.lists[right.index as usize].clone_value()
                                }
                                AddressKind::LIST_REGISTER => {
                                    memory.registers.lists[right.index as usize].clone()
                                }
                                _ => unreachable!(),
                            };

                            left_value <= right_value
                        }
                        AddressKind::FUNCTION_MEMORY => {
                            let left_value = memory.functions[left_index].clone_value();
                            let right_value = match right.kind {
                                AddressKind::FUNCTION_MEMORY => {
                                    memory.functions[right.index as usize].clone_value()
                                }
                                AddressKind::FUNCTION_REGISTER => {
                                    Arc::clone(&memory.registers.functions[right.index as usize])
                                }
                                _ => unreachable!(),
                            };

                            left_value <= right_value
                        }
                        AddressKind::FUNCTION_REGISTER => {
                            let left_value = Arc::clone(&memory.registers.functions[left_index]);
                            let right_value = match right.kind {
                                AddressKind::FUNCTION_MEMORY => {
                                    memory.functions[right.index as usize].clone_value()
                                }
                                AddressKind::FUNCTION_REGISTER => {
                                    Arc::clone(&memory.registers.functions[right.index as usize])
                                }
                                _ => unreachable!(),
                            };

                            left_value <= right_value
                        }
                        _ => unreachable!(),
                    };

                    if is_less_than_or_equal == comparator {
                        call.ip += 1;
                    }
                }
                Operation::NEGATE => todo!(),
                Operation::NOT => todo!(),
                Operation::TEST => {
                    let Test {
                        comparator,
                        operand,
                    } = Test::from(&instruction);

                    let operand_index = operand.index as usize;
                    let is_true = match operand.kind {
                        AddressKind::BOOLEAN_MEMORY => *memory.booleans[operand_index].as_value(),
                        AddressKind::BOOLEAN_REGISTER => memory.registers.booleans[operand_index],
                        _ => unreachable!(),
                    };

                    if is_true == comparator {
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

                    function.call(destination, arguments, &mut call, &mut memory);
                }
                Operation::CALL => {
                    let Call {
                        destination,
                        function: function_address,
                        argument_list_index,
                        return_type,
                    } = Call::from(&instruction);

                    let function = match function_address.kind {
                        AddressKind::FUNCTION_PROTOTYPE => {
                            Arc::clone(&call.chunk.prototypes[function_address.index as usize])
                        }
                        AddressKind::FUNCTION_SELF => Arc::clone(&call.chunk),
                        AddressKind::FUNCTION_REGISTER => {
                            let abstract_function =
                                &memory.registers.functions[function_address.index as usize];

                            Arc::clone(abstract_function)
                        }
                        AddressKind::FUNCTION_MEMORY => {
                            let abstract_function =
                                memory.functions[function_address.index as usize].as_value();

                            Arc::clone(abstract_function)
                        }
                        _ => unreachable!(),
                    };

                    let arguments_list = &call.chunk.arguments[argument_list_index as usize];
                    let parameters_list = function.locals.iter().map(|local| local.address);
                    let new_call = CallFrame::new(
                        Arc::clone(&function),
                        destination.as_address(return_type.r#type()),
                    );
                    let mut new_memory = Memory::new(&function);

                    for (argument, parameter) in arguments_list.values.iter().zip(parameters_list) {
                        match argument.kind {
                            AddressKind::INTEGER_REGISTER => {
                                let integer = memory.registers.integers[argument.index as usize];

                                match parameter.kind {
                                    AddressKind::INTEGER_REGISTER => {
                                        new_memory.registers.integers[parameter.index as usize] =
                                            integer;
                                    }
                                    AddressKind::INTEGER_MEMORY => {
                                        *new_memory.integers[parameter.index as usize]
                                            .as_value_mut() = integer;
                                    }
                                    _ => unreachable!(),
                                }
                            }
                            AddressKind::INTEGER_MEMORY => {
                                let integer = *memory.integers[argument.index as usize].as_value();

                                match parameter.kind {
                                    AddressKind::INTEGER_REGISTER => {
                                        new_memory.registers.integers[parameter.index as usize] =
                                            integer;
                                    }
                                    AddressKind::INTEGER_MEMORY => {
                                        *new_memory.integers[parameter.index as usize]
                                            .as_value_mut() = integer;
                                    }
                                    _ => unreachable!(),
                                }
                            }
                            _ => unreachable!(),
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
                        AddressKind::BOOLEAN_REGISTER => {
                            let boolean =
                                memory.registers.booleans[return_value_address.index as usize];

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::Boolean(boolean));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            match call.return_address.kind {
                                AddressKind::NONE => {}
                                AddressKind::INTEGER_REGISTER => {
                                    new_memory.registers.booleans
                                        [call.return_address.index as usize] = boolean;
                                }
                                _ => unreachable!(),
                            }

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

                            match call.return_address.kind {
                                AddressKind::NONE => {}
                                AddressKind::INTEGER_REGISTER => {
                                    new_memory.registers.bytes
                                        [call.return_address.index as usize] = byte;
                                }
                                _ => unreachable!(),
                            }

                            (new_call, new_memory)
                        }
                        AddressKind::CHARACTER_REGISTER => {
                            let character =
                                memory.registers.characters[return_value_address.index as usize];

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::Character(character));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            match call.return_address.kind {
                                AddressKind::NONE => {}
                                AddressKind::INTEGER_REGISTER => {
                                    new_memory.registers.characters
                                        [call.return_address.index as usize] = character;
                                }
                                _ => unreachable!(),
                            }

                            (new_call, new_memory)
                        }
                        AddressKind::FLOAT_REGISTER => {
                            let float =
                                memory.registers.floats[return_value_address.index as usize];

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::Float(float));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            match call.return_address.kind {
                                AddressKind::NONE => {}
                                AddressKind::INTEGER_REGISTER => {
                                    new_memory.registers.floats
                                        [call.return_address.index as usize] = float;
                                }
                                _ => unreachable!(),
                            }

                            (new_call, new_memory)
                        }
                        AddressKind::INTEGER_REGISTER => {
                            let integer =
                                memory.registers.integers[return_value_address.index as usize];

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::Integer(integer));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            match call.return_address.kind {
                                AddressKind::NONE => {}
                                AddressKind::INTEGER_REGISTER => {
                                    new_memory.registers.integers
                                        [call.return_address.index as usize] = integer;
                                }
                                AddressKind::INTEGER_MEMORY => {
                                    *new_memory.integers[call.return_address.index as usize]
                                        .as_value_mut() = integer;
                                }
                                _ => unreachable!(),
                            }

                            (new_call, new_memory)
                        }
                        AddressKind::STRING_REGISTER => {
                            let string = memory.registers.strings
                                [return_value_address.index as usize]
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

                            match call.return_address.kind {
                                AddressKind::NONE => {}
                                AddressKind::INTEGER_REGISTER => {
                                    new_memory.registers.strings
                                        [call.return_address.index as usize] = string;
                                }
                                _ => unreachable!(),
                            }

                            (new_call, new_memory)
                        }
                        AddressKind::LIST_REGISTER => {
                            let abstract_list =
                                memory.registers.lists[return_value_address.index as usize].clone();
                            let concrete_list = memory.make_list_concrete(&abstract_list);

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::List(concrete_list));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            match call.return_address.kind {
                                AddressKind::NONE => {}
                                AddressKind::INTEGER_REGISTER => {
                                    new_memory.registers.lists
                                        [call.return_address.index as usize] = abstract_list;
                                }
                                _ => unreachable!(),
                            }

                            (new_call, new_memory)
                        }
                        AddressKind::FUNCTION_REGISTER => {
                            let function = Arc::clone(
                                &memory.registers.functions[return_value_address.index as usize],
                            );

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::Function(function));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            match call.return_address.kind {
                                AddressKind::NONE => {}
                                AddressKind::INTEGER_REGISTER => {
                                    new_memory.registers.functions
                                        [call.return_address.index as usize] = function;
                                }
                                _ => unreachable!(),
                            }

                            (new_call, new_memory)
                        }
                        _ => todo!(),
                    };

                    call = new_call;
                    memory = new_memory;
                }
                _ => unreachable!(),
            }
        }
    }
}

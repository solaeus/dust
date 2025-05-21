use std::{mem::replace, thread::JoinHandle};

use tracing::{Level, info, span, warn};

use crate::{
    AbstractList, Address, Chunk, ConcreteList, ConcreteValue, Operation,
    instruction::{
        Add, AddressKind, Call, Close, Jump, Less, LoadConstant, LoadEncoded, LoadFunction,
        LoadList, Move, Multiply, Return,
    },
};

use super::{CallFrame, Memory};

pub struct Thread<'a> {
    chunk: &'a Chunk,

    _spawned_threads: Vec<JoinHandle<()>>,
}

impl<'a> Thread<'a> {
    pub fn new(chunk: &'a Chunk) -> Self {
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
                .unwrap_or_else(|| "anonymous")
        );

        let mut call_stack = Vec::with_capacity(self.chunk.prototypes.len() + 1);
        let mut memory_stack = Vec::with_capacity(self.chunk.prototypes.len() + 1);

        let mut call = CallFrame::new(self.chunk, Address::default());
        let mut memory = Memory::new(self.chunk);

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
                        AddressKind::INTEGER_CONSTANT => {
                            assert!(left_index < self.chunk.integer_constants.len());

                            let left_value = self.chunk.integer_constants[left_index];
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
                            assert!(left_index < memory.integers.len());

                            let left_value = memory.integers[left_index].as_value();
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
                            assert!(left_index < memory.registers.integers.len());

                            let left_value = memory.registers.integers[left_index];
                            let right_index = right.index as usize;
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    assert!(right_index < self.chunk.integer_constants.len());

                                    self.chunk.integer_constants[right_index]
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
                Operation::MULTIPLY => {
                    let Multiply {
                        destination,
                        left,
                        right,
                    } = Multiply::from(&instruction);
                    let left_index = left.index as usize;

                    match left.kind {
                        AddressKind::INTEGER_CONSTANT => {
                            assert!(left_index < self.chunk.integer_constants.len());

                            let left_value = self.chunk.integer_constants[left_index];
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
                            let product = left_value * right_value;

                            if destination.is_register {
                                memory.registers.integers[destination.index as usize] = product;
                            } else {
                                *memory.integers[destination.index as usize].as_value_mut() =
                                    product;
                            }
                        }
                        AddressKind::INTEGER_MEMORY => {
                            assert!(left_index < memory.integers.len());

                            let left_value = memory.integers[left_index].as_value();
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
                            let product = left_value * right_value;

                            if destination.is_register {
                                memory.registers.integers[destination.index as usize] = product;
                            } else {
                                *memory.integers[destination.index as usize].as_value_mut() =
                                    product;
                            }
                        }
                        AddressKind::INTEGER_REGISTER => {
                            assert!(left_index < memory.registers.integers.len());

                            let left_value = memory.registers.integers[left_index];
                            let right_index = right.index as usize;
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    assert!(right_index < self.chunk.integer_constants.len());

                                    self.chunk.integer_constants[right_index]
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
                Operation::DIVIDE => todo!(),
                Operation::MODULO => todo!(),
                Operation::EQUAL => todo!(),
                Operation::LESS => {
                    let Less {
                        comparator,
                        left,
                        right,
                    } = Less::from(&instruction);

                    let left_index = left.index as usize;
                    let is_less_than = match left.kind {
                        AddressKind::INTEGER_MEMORY => {
                            assert!(left_index < memory.integers.len());

                            let left_value = *memory.integers[left_index].as_value();
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
                            assert!(left_index < memory.registers.integers.len());

                            let left_value = memory.registers.integers[left_index];
                            let right_index = right.index as usize;
                            let right_value = match right.kind {
                                AddressKind::INTEGER_CONSTANT => {
                                    assert!(right_index < self.chunk.integer_constants.len());

                                    self.chunk.integer_constants[right_index]
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
                        argument_list_index_and_return_type:
                            Address {
                                index: argument_list_index,
                                kind: return_type,
                            },
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
                            &call.chunk.prototypes[prototype_address.index as usize]
                        }
                        AddressKind::FUNCTION_SELF => self.chunk,
                        _ => unreachable!(),
                    };
                    let arguments_list = &call.chunk.arguments[argument_list_index as usize];
                    let parameters_list = function.locals.iter().map(|local| local.address);
                    let new_call =
                        CallFrame::new(function, destination.as_address(return_type.r#type()));
                    let mut new_memory = Memory::new(function);

                    for (argument, parameter) in arguments_list.values.iter().zip(parameters_list) {
                        match argument.kind {
                            AddressKind::INTEGER_REGISTER => {
                                let integer = memory.registers.integers[argument.index as usize];

                                match parameter.kind {
                                    AddressKind::INTEGER_REGISTER => {
                                        new_memory.registers.integers[parameter.index as usize] =
                                            integer;
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
                Operation::CALL_NATIVE => todo!(),
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
                            let abstract_function =
                                memory.registers.functions[return_value_address.index as usize];
                            let function_prototype = call.chunk.prototypes
                                [abstract_function.prototype_address.index as usize]
                                .clone();

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::Function(function_prototype));
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
                                        [call.return_address.index as usize] = abstract_function;
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

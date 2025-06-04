use std::{mem::replace, sync::Arc, thread::JoinHandle};

use tracing::{Level, info, span, warn};

use crate::{
    AbstractList, Address, Chunk, ConcreteValue, DustString, Operation,
    instruction::{
        Add, AddressKind, Call, CallNative, Close, Divide, Equal, Jump, Less, LessEqual,
        LoadConstant, LoadEncoded, LoadFunction, LoadList, Modulo, Move, Multiply, Return,
        Subtract, Test,
    },
    r#type::TypeKind,
};

use super::{CallFrame, Memory};

pub struct Thread<const REGISTER_COUNT: usize> {
    chunk: Arc<Chunk>,

    _spawned_threads: Vec<JoinHandle<()>>,
}

impl<const REGISTER_COUNT: usize> Thread<REGISTER_COUNT> {
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

                    match from.kind.r#type() {
                        TypeKind::Boolean => {
                            let boolean = memory.get_boolean(from.is_register(), from.index);

                            memory.set_boolean(to.is_register, to.index, boolean);
                        }
                        TypeKind::Byte => {
                            let byte = memory.get_byte(from.is_register(), from.index);

                            memory.set_byte(to.is_register, to.index, byte);
                        }
                        TypeKind::Character => {
                            let character = if from.is_constant() {
                                call.chunk.character_constants[from.index as usize]
                            } else {
                                memory.get_character(from.is_register(), from.index)
                            };

                            memory.set_character(to.is_register, to.index, character);
                        }
                        TypeKind::Float => {
                            let float = if from.is_constant() {
                                call.chunk.float_constants[from.index as usize]
                            } else {
                                memory.get_float(from.is_register(), from.index)
                            };

                            memory.set_float(to.is_register, to.index, float);
                        }
                        TypeKind::Integer => {
                            let integer = if from.is_constant() {
                                call.chunk.integer_constants[from.index as usize]
                            } else {
                                memory.get_integer(from.is_register(), from.index)
                            };

                            memory.set_integer(to.is_register, to.index, integer);
                        }
                        TypeKind::String => {
                            let string = if from.is_constant() {
                                call.chunk.string_constants[from.index as usize].clone()
                            } else {
                                memory.get_string(from.is_register(), from.index).clone()
                            };

                            memory.set_string(to.is_register, to.index, string);
                        }
                        TypeKind::List => {
                            let abstract_list =
                                memory.get_list(from.is_register(), from.index).clone();

                            memory.set_list(to.is_register, to.index, abstract_list);
                        }
                        TypeKind::Function => {
                            let function = if from.is_constant() {
                                Arc::clone(&call.chunk.prototypes[from.index as usize])
                            } else {
                                Arc::clone(memory.get_function(from.is_register(), from.index))
                            };

                            memory.set_function(to.is_register, to.index, function);
                        }
                        _ => unreachable!(),
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

                            memory.set_boolean(destination.is_register, destination.index, boolean);
                        }
                        AddressKind::BYTE_MEMORY => {
                            let byte = value as u8;

                            memory.set_byte(destination.is_register, destination.index, byte);
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

                            memory.set_character(destination.is_register, destination.index, value);
                        }
                        AddressKind::FLOAT_CONSTANT => {
                            let value = call.chunk.float_constants[constant_index];

                            memory.set_float(destination.is_register, destination.index, value);
                        }
                        AddressKind::INTEGER_CONSTANT => {
                            let value = call.chunk.integer_constants[constant_index];

                            memory.set_integer(destination.is_register, destination.index, value);
                        }
                        AddressKind::STRING_CONSTANT => {
                            let value = call.chunk.string_constants[constant_index].clone();

                            memory.set_string(destination.is_register, destination.index, value);
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

                    memory.set_function(destination.is_register, destination.index, function);

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

                    memory.set_list(destination.is_register, destination.index, abstract_list);

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

                    match (left.kind.r#type(), right.kind.r#type()) {
                        (TypeKind::Byte, TypeKind::Byte) => {
                            let left_value = memory.get_byte(left.is_register(), left.index);
                            let right_value = memory.get_byte(right.is_register(), right.index);
                            let sum = left_value + right_value;

                            memory.set_byte(destination.is_register, destination.index, sum);
                        }
                        (TypeKind::Character, TypeKind::Character) => {
                            let left_value = if left.is_constant() {
                                call.chunk.character_constants[left_index]
                            } else {
                                memory.get_character(left.is_register(), left.index)
                            };
                            let right_value = if right.is_constant() {
                                call.chunk.character_constants[right.index as usize]
                            } else {
                                memory.get_character(right.is_register(), right.index)
                            };
                            let mut sum = DustString::new();

                            sum.push(left_value);
                            sum.push(right_value);

                            memory.set_string(destination.is_register, destination.index, sum);
                        }
                        (TypeKind::Float, TypeKind::Float) => {
                            let left_value = if left.is_constant() {
                                call.chunk.float_constants[left_index]
                            } else {
                                memory.get_float(left.is_register(), left.index)
                            };
                            let right_value = if right.is_constant() {
                                call.chunk.float_constants[right.index as usize]
                            } else {
                                memory.get_float(right.is_register(), right.index)
                            };
                            let sum = left_value + right_value;

                            memory.set_float(destination.is_register, destination.index, sum);
                        }
                        (TypeKind::Integer, TypeKind::Integer) => {
                            let left_value = if left.is_constant() {
                                call.chunk.integer_constants[left_index]
                            } else {
                                memory.get_integer(left.is_register(), left.index)
                            };
                            let right_value = if right.is_constant() {
                                call.chunk.integer_constants[right.index as usize]
                            } else {
                                memory.get_integer(right.is_register(), right.index)
                            };
                            let sum = left_value + right_value;

                            memory.set_integer(destination.is_register, destination.index, sum);
                        }
                        (TypeKind::String, TypeKind::String) => {
                            let left_value = if left.is_constant() {
                                &call.chunk.string_constants[left_index]
                            } else {
                                memory.get_string(left.is_register(), left.index)
                            };
                            let right_value = if right.is_constant() {
                                &call.chunk.string_constants[right.index as usize]
                            } else {
                                memory.get_string(right.is_register(), right.index)
                            };
                            let mut sum = DustString::new();

                            sum.push_str(left_value);
                            sum.push_str(right_value);

                            memory.set_string(destination.is_register, destination.index, sum);
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

                    match left.kind.r#type() {
                        TypeKind::Byte => {
                            let left_value = memory.get_byte(left.is_register(), left.index);
                            let right_value = memory.get_byte(right.is_register(), right.index);
                            let difference = left_value - right_value;

                            memory.set_byte(destination.is_register, destination.index, difference);
                        }
                        TypeKind::Float => {
                            let left_value = if left.is_constant() {
                                call.chunk.float_constants[left_index]
                            } else {
                                memory.get_float(left.is_register(), left.index)
                            };
                            let right_value = if right.is_constant() {
                                call.chunk.float_constants[right.index as usize]
                            } else {
                                memory.get_float(right.is_register(), right.index)
                            };
                            let difference = left_value - right_value;

                            memory.set_float(
                                destination.is_register,
                                destination.index,
                                difference,
                            );
                        }
                        TypeKind::Integer => {
                            let left_value = if left.is_constant() {
                                call.chunk.integer_constants[left_index]
                            } else {
                                memory.get_integer(left.is_register(), left.index)
                            };
                            let right_value = if right.is_constant() {
                                call.chunk.integer_constants[right.index as usize]
                            } else {
                                memory.get_integer(right.is_register(), right.index)
                            };
                            let difference = left_value - right_value;

                            memory.set_integer(
                                destination.is_register,
                                destination.index,
                                difference,
                            );
                        }
                        _ => unreachable!(),
                    }
                }
                Operation::MULTIPLY => {
                    let Multiply {
                        destination,
                        left,
                        right,
                    } = Multiply::from(&instruction);
                    let left_index = left.index as usize;

                    match left.kind.r#type() {
                        TypeKind::Byte => {
                            let left_value = memory.get_byte(left.is_register(), left.index);
                            let right_value = memory.get_byte(right.is_register(), right.index);
                            let product = left_value * right_value;

                            memory.set_byte(destination.is_register, destination.index, product);
                        }
                        TypeKind::Float => {
                            let left_value = if left.is_constant() {
                                call.chunk.float_constants[left_index]
                            } else {
                                memory.get_float(left.is_register(), left.index)
                            };
                            let right_value = if right.is_constant() {
                                call.chunk.float_constants[right.index as usize]
                            } else {
                                memory.get_float(right.is_register(), right.index)
                            };
                            let product = left_value * right_value;

                            memory.set_float(destination.is_register, destination.index, product);
                        }
                        TypeKind::Integer => {
                            let left_value = if left.is_constant() {
                                call.chunk.integer_constants[left_index]
                            } else {
                                memory.get_integer(left.is_register(), left.index)
                            };
                            let right_value = if right.is_constant() {
                                call.chunk.integer_constants[right.index as usize]
                            } else {
                                memory.get_integer(right.is_register(), right.index)
                            };
                            let product = left_value * right_value;

                            memory.set_integer(destination.is_register, destination.index, product);
                        }
                        _ => unreachable!(),
                    }
                }
                Operation::DIVIDE => {
                    let Divide {
                        destination,
                        left,
                        right,
                    } = Divide::from(&instruction);
                    let left_index = left.index as usize;

                    match left.kind.r#type() {
                        TypeKind::Byte => {
                            let left_value = memory.get_byte(left.is_register(), left.index);
                            let right_value = memory.get_byte(right.is_register(), right.index);
                            let quotient = left_value / right_value;

                            memory.set_byte(destination.is_register, destination.index, quotient);
                        }
                        TypeKind::Float => {
                            let left_value = if left.is_constant() {
                                call.chunk.float_constants[left_index]
                            } else {
                                memory.get_float(left.is_register(), left.index)
                            };
                            let right_value = if right.is_constant() {
                                call.chunk.float_constants[right.index as usize]
                            } else {
                                memory.get_float(right.is_register(), right.index)
                            };
                            let quotient = left_value / right_value;

                            memory.set_float(destination.is_register, destination.index, quotient);
                        }
                        TypeKind::Integer => {
                            let left_value = if left.is_constant() {
                                call.chunk.integer_constants[left_index]
                            } else {
                                memory.get_integer(left.is_register(), left.index)
                            };
                            let right_value = if right.is_constant() {
                                call.chunk.integer_constants[right.index as usize]
                            } else {
                                memory.get_integer(right.is_register(), right.index)
                            };
                            let quotient = left_value / right_value;

                            memory.set_integer(
                                destination.is_register,
                                destination.index,
                                quotient,
                            );
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

                    match left.kind.r#type() {
                        TypeKind::Byte => {
                            let left_value = memory.get_byte(left.is_register(), left.index);
                            let right_value = memory.get_byte(right.is_register(), right.index);
                            let remainder = left_value % right_value;

                            memory.set_byte(destination.is_register, destination.index, remainder);
                        }
                        TypeKind::Float => {
                            let left_value = if left.is_constant() {
                                call.chunk.float_constants[left_index]
                            } else {
                                memory.get_float(left.is_register(), left.index)
                            };
                            let right_value = if right.is_constant() {
                                call.chunk.float_constants[right.index as usize]
                            } else {
                                memory.get_float(right.is_register(), right.index)
                            };
                            let remainder = left_value % right_value;

                            memory.set_float(destination.is_register, destination.index, remainder);
                        }
                        TypeKind::Integer => {
                            let left_value = if left.is_constant() {
                                call.chunk.integer_constants[left_index]
                            } else {
                                memory.get_integer(left.is_register(), left.index)
                            };
                            let right_value = if right.is_constant() {
                                call.chunk.integer_constants[right.index as usize]
                            } else {
                                memory.get_integer(right.is_register(), right.index)
                            };
                            let remainder = left_value % right_value;

                            memory.set_integer(
                                destination.is_register,
                                destination.index,
                                remainder,
                            );
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

                    let is_equal = match left.kind.r#type() {
                        TypeKind::Boolean => {
                            let left_value = memory.get_boolean(left.is_register(), left.index);
                            let right_value = memory.get_boolean(right.is_register(), right.index);

                            left_value == right_value
                        }
                        TypeKind::Byte => {
                            let left_value = memory.get_byte(left.is_register(), left.index);
                            let right_value = memory.get_byte(right.is_register(), right.index);

                            left_value == right_value
                        }
                        TypeKind::Character => {
                            let left_value = if left.is_constant() {
                                call.chunk.character_constants[left.index as usize]
                            } else {
                                memory.get_character(left.is_register(), left.index)
                            };
                            let right_value = if right.is_constant() {
                                call.chunk.character_constants[right.index as usize]
                            } else {
                                memory.get_character(right.is_register(), right.index)
                            };

                            left_value == right_value
                        }
                        TypeKind::Float => {
                            let left_value = if left.is_constant() {
                                call.chunk.float_constants[left.index as usize]
                            } else {
                                memory.get_float(left.is_register(), left.index)
                            };
                            let right_value = if right.is_constant() {
                                call.chunk.float_constants[right.index as usize]
                            } else {
                                memory.get_float(right.is_register(), right.index)
                            };

                            left_value == right_value
                        }
                        TypeKind::Integer => {
                            let left_value = if left.is_constant() {
                                call.chunk.integer_constants[left.index as usize]
                            } else {
                                memory.get_integer(left.is_register(), left.index)
                            };
                            let right_value = if right.is_constant() {
                                call.chunk.integer_constants[right.index as usize]
                            } else {
                                memory.get_integer(right.is_register(), right.index)
                            };

                            left_value == right_value
                        }
                        TypeKind::String => {
                            let left_value = if left.is_constant() {
                                &call.chunk.string_constants[left.index as usize]
                            } else {
                                memory.get_string(left.is_register(), left.index)
                            };
                            let right_value = if right.is_constant() {
                                &call.chunk.string_constants[right.index as usize]
                            } else {
                                memory.get_string(right.is_register(), right.index)
                            };

                            left_value == right_value
                        }
                        TypeKind::List => {
                            let left_value = memory.get_list(left.is_register(), left.index);
                            let right_value = memory.get_list(right.is_register(), right.index);

                            left_value == right_value
                        }
                        TypeKind::Function => {
                            let left_value = if left.is_constant() {
                                &call.chunk.prototypes[left.index as usize]
                            } else {
                                memory.get_function(left.is_register(), left.index)
                            };
                            let right_value = if right.is_constant() {
                                &call.chunk.prototypes[right.index as usize]
                            } else {
                                memory.get_function(right.is_register(), right.index)
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

                    #[expect(clippy::bool_comparison)]
                    let is_less_than = match left.kind.r#type() {
                        TypeKind::Boolean => {
                            let left_value = memory.get_boolean(left.is_register(), left.index);
                            let right_value = memory.get_boolean(right.is_register(), right.index);

                            left_value < right_value
                        }
                        TypeKind::Byte => {
                            let left_value = memory.get_byte(left.is_register(), left.index);
                            let right_value = memory.get_byte(right.is_register(), right.index);

                            left_value < right_value
                        }
                        TypeKind::Character => {
                            let left_value = if left.is_constant() {
                                call.chunk.character_constants[left.index as usize]
                            } else {
                                memory.get_character(left.is_register(), left.index)
                            };
                            let right_value = if right.is_constant() {
                                call.chunk.character_constants[right.index as usize]
                            } else {
                                memory.get_character(right.is_register(), right.index)
                            };

                            left_value < right_value
                        }
                        TypeKind::Float => {
                            let left_value = if left.is_constant() {
                                call.chunk.float_constants[left.index as usize]
                            } else {
                                memory.get_float(left.is_register(), left.index)
                            };
                            let right_value = if right.is_constant() {
                                call.chunk.float_constants[right.index as usize]
                            } else {
                                memory.get_float(right.is_register(), right.index)
                            };

                            left_value < right_value
                        }
                        TypeKind::Integer => {
                            let left_value = if left.is_constant() {
                                call.chunk.integer_constants[left.index as usize]
                            } else {
                                memory.get_integer(left.is_register(), left.index)
                            };
                            let right_value = if right.is_constant() {
                                call.chunk.integer_constants[right.index as usize]
                            } else {
                                memory.get_integer(right.is_register(), right.index)
                            };

                            left_value < right_value
                        }
                        TypeKind::String => {
                            let left_value = if left.is_constant() {
                                &call.chunk.string_constants[left.index as usize]
                            } else {
                                memory.get_string(left.is_register(), left.index)
                            };
                            let right_value = if right.is_constant() {
                                &call.chunk.string_constants[right.index as usize]
                            } else {
                                memory.get_string(right.is_register(), right.index)
                            };

                            left_value < right_value
                        }
                        TypeKind::List => {
                            let left_value = memory.get_list(left.is_register(), left.index);
                            let right_value = memory.get_list(right.is_register(), right.index);

                            left_value < right_value
                        }
                        TypeKind::Function => {
                            let left_value = if left.is_constant() {
                                &call.chunk.prototypes[left.index as usize]
                            } else {
                                memory.get_function(left.is_register(), left.index)
                            };
                            let right_value = if right.is_constant() {
                                &call.chunk.prototypes[right.index as usize]
                            } else {
                                memory.get_function(right.is_register(), right.index)
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

                    let is_less_than_or_equal = match left.kind.r#type() {
                        TypeKind::Boolean => {
                            let left_value = memory.get_boolean(left.is_register(), left.index);
                            let right_value = memory.get_boolean(right.is_register(), right.index);

                            left_value <= right_value
                        }
                        TypeKind::Byte => {
                            let left_value = memory.get_byte(left.is_register(), left.index);
                            let right_value = memory.get_byte(right.is_register(), right.index);

                            left_value <= right_value
                        }
                        TypeKind::Character => {
                            let left_value = if left.is_constant() {
                                call.chunk.character_constants[left.index as usize]
                            } else {
                                memory.get_character(left.is_register(), left.index)
                            };
                            let right_value = if right.is_constant() {
                                call.chunk.character_constants[right.index as usize]
                            } else {
                                memory.get_character(right.is_register(), right.index)
                            };

                            left_value <= right_value
                        }
                        TypeKind::Float => {
                            let left_value = if left.is_constant() {
                                call.chunk.float_constants[left.index as usize]
                            } else {
                                memory.get_float(left.is_register(), left.index)
                            };
                            let right_value = if right.is_constant() {
                                call.chunk.float_constants[right.index as usize]
                            } else {
                                memory.get_float(right.is_register(), right.index)
                            };

                            left_value <= right_value
                        }
                        TypeKind::Integer => {
                            let left_value = if left.is_constant() {
                                call.chunk.integer_constants[left.index as usize]
                            } else {
                                memory.get_integer(left.is_register(), left.index)
                            };
                            let right_value = if right.is_constant() {
                                call.chunk.integer_constants[right.index as usize]
                            } else {
                                memory.get_integer(right.is_register(), right.index)
                            };

                            left_value <= right_value
                        }
                        TypeKind::String => {
                            let left_value = if left.is_constant() {
                                &call.chunk.string_constants[left.index as usize]
                            } else {
                                memory.get_string(left.is_register(), left.index)
                            };
                            let right_value = if right.is_constant() {
                                &call.chunk.string_constants[right.index as usize]
                            } else {
                                memory.get_string(right.is_register(), right.index)
                            };

                            left_value <= right_value
                        }
                        TypeKind::List => {
                            let left_value = memory.get_list(left.is_register(), left.index);
                            let right_value = memory.get_list(right.is_register(), right.index);

                            left_value <= right_value
                        }
                        TypeKind::Function => {
                            let left_value = if left.is_constant() {
                                &call.chunk.prototypes[left.index as usize]
                            } else {
                                memory.get_function(left.is_register(), left.index)
                            };
                            let right_value = if right.is_constant() {
                                &call.chunk.prototypes[right.index as usize]
                            } else {
                                memory.get_function(right.is_register(), right.index)
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

                    let is_true = memory.get_boolean(operand.is_register(), operand.index);

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
                        AddressKind::FUNCTION_REGISTER | AddressKind::FUNCTION_MEMORY => {
                            let function = memory.get_function(
                                function_address.is_register(),
                                function_address.index,
                            );

                            Arc::clone(function)
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
                        match argument.kind.r#type() {
                            TypeKind::Boolean => {
                                let boolean =
                                    memory.get_boolean(argument.is_register(), argument.index);

                                new_memory.set_boolean(
                                    parameter.is_register(),
                                    parameter.index,
                                    boolean,
                                );
                            }
                            TypeKind::Byte => {
                                let byte = memory.get_byte(argument.is_register(), argument.index);

                                new_memory.set_byte(parameter.is_register(), parameter.index, byte);
                            }
                            TypeKind::Float => {
                                let float = if argument.is_constant() {
                                    call.chunk.float_constants[argument.index as usize]
                                } else {
                                    memory.get_float(argument.is_register(), argument.index)
                                };

                                new_memory.set_float(
                                    parameter.is_register(),
                                    parameter.index,
                                    float,
                                );
                            }
                            TypeKind::Integer => {
                                let integer = if argument.is_constant() {
                                    call.chunk.integer_constants[argument.index as usize]
                                } else {
                                    memory.get_integer(argument.is_register(), argument.index)
                                };

                                new_memory.set_integer(
                                    parameter.is_register(),
                                    parameter.index,
                                    integer,
                                );
                            }
                            TypeKind::String => {
                                let string = if argument.is_constant() {
                                    call.chunk.string_constants[argument.index as usize].clone()
                                } else {
                                    memory
                                        .get_string(argument.is_register(), argument.index)
                                        .clone()
                                };

                                new_memory.set_string(
                                    parameter.is_register(),
                                    parameter.index,
                                    string,
                                );
                            }
                            TypeKind::List => {
                                let abstract_list = memory
                                    .get_list(argument.is_register(), argument.index)
                                    .clone();

                                new_memory.set_list(
                                    parameter.is_register(),
                                    parameter.index,
                                    abstract_list,
                                );
                            }
                            TypeKind::Function => {
                                let function = match argument.kind {
                                    AddressKind::FUNCTION_REGISTER
                                    | AddressKind::FUNCTION_MEMORY => Arc::clone(
                                        memory.get_function(argument.is_register(), argument.index),
                                    ),
                                    AddressKind::FUNCTION_PROTOTYPE => {
                                        Arc::clone(&call.chunk.prototypes[argument.index as usize])
                                    }
                                    AddressKind::FUNCTION_SELF => Arc::clone(&call.chunk),
                                    _ => unreachable!(),
                                };

                                new_memory.set_function(
                                    parameter.is_register(),
                                    parameter.index,
                                    function,
                                );
                            }
                            _ => todo!(),
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

                    let (new_call, new_memory) = match return_value_address.kind.r#type() {
                        TypeKind::None => {
                            if call_stack.is_empty() {
                                return None;
                            }

                            (call_stack.pop().unwrap(), memory_stack.pop().unwrap())
                        }
                        TypeKind::Boolean => {
                            let boolean = memory.get_boolean(
                                return_value_address.is_register(),
                                return_value_address.index,
                            );

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::Boolean(boolean));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            new_memory.set_boolean(
                                call.return_address.is_register(),
                                call.return_address.index,
                                boolean,
                            );

                            (new_call, new_memory)
                        }
                        TypeKind::Byte => {
                            let byte = memory.get_byte(
                                return_value_address.is_register(),
                                return_value_address.index,
                            );

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::Byte(byte));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            new_memory.set_byte(
                                call.return_address.is_register(),
                                call.return_address.index,
                                byte,
                            );

                            (new_call, new_memory)
                        }
                        TypeKind::Character => {
                            let character = if return_value_address.is_constant() {
                                call.chunk.character_constants[return_value_address.index as usize]
                            } else {
                                memory.get_character(
                                    return_value_address.is_register(),
                                    return_value_address.index,
                                )
                            };

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::Character(character));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            new_memory.set_character(
                                call.return_address.is_register(),
                                call.return_address.index,
                                character,
                            );

                            (new_call, new_memory)
                        }
                        TypeKind::Float => {
                            let float = if return_value_address.is_constant() {
                                call.chunk.float_constants[return_value_address.index as usize]
                            } else {
                                memory.get_float(
                                    return_value_address.is_register(),
                                    return_value_address.index,
                                )
                            };

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::Float(float));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            new_memory.set_float(
                                call.return_address.is_register(),
                                call.return_address.index,
                                float,
                            );

                            (new_call, new_memory)
                        }
                        TypeKind::Integer => {
                            let integer = if return_value_address.is_constant() {
                                call.chunk.integer_constants[return_value_address.index as usize]
                            } else {
                                memory.get_integer(
                                    return_value_address.is_register(),
                                    return_value_address.index,
                                )
                            };

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::Integer(integer));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            new_memory.set_integer(
                                call.return_address.is_register(),
                                call.return_address.index,
                                integer,
                            );

                            (new_call, new_memory)
                        }
                        TypeKind::String => {
                            let string = if return_value_address.is_constant() {
                                call.chunk.string_constants[return_value_address.index as usize]
                                    .clone()
                            } else {
                                memory
                                    .get_string(
                                        return_value_address.is_register(),
                                        return_value_address.index,
                                    )
                                    .clone()
                            };

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::String(string));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            new_memory.set_string(
                                call.return_address.is_register(),
                                call.return_address.index,
                                string,
                            );

                            (new_call, new_memory)
                        }
                        TypeKind::List => {
                            let list = memory.get_list(
                                return_value_address.is_register(),
                                return_value_address.index,
                            );

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::List(
                                        memory.make_list_concrete(list),
                                    ));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            new_memory.set_list(
                                call.return_address.is_register(),
                                call.return_address.index,
                                list.clone(),
                            );

                            (new_call, new_memory)
                        }
                        TypeKind::Function => {
                            let function = match return_value_address.kind {
                                AddressKind::FUNCTION_REGISTER | AddressKind::FUNCTION_MEMORY => {
                                    Arc::clone(memory.get_function(
                                        return_value_address.is_register(),
                                        return_value_address.index,
                                    ))
                                }
                                AddressKind::FUNCTION_PROTOTYPE => Arc::clone(
                                    &call.chunk.prototypes[return_value_address.index as usize],
                                ),
                                AddressKind::FUNCTION_SELF => Arc::clone(&call.chunk),
                                _ => unreachable!(),
                            };

                            if call_stack.is_empty() {
                                if should_return_value {
                                    return Some(ConcreteValue::Function(function));
                                } else {
                                    return None;
                                }
                            }

                            let new_call = call_stack.pop().unwrap();
                            let mut new_memory = memory_stack.pop().unwrap();

                            new_memory.set_function(
                                call.return_address.is_register(),
                                call.return_address.index,
                                function,
                            );

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

use std::{
    mem::replace,
    sync::{Arc, RwLock},
    thread::{Builder as ThreadBuilder, JoinHandle},
};

use tracing::{Level, info, span, warn};

use crate::{
    Address, Chunk, DustString, Operation, Value,
    instruction::{
        Add, Call, CallNative, Divide, Equal, Jump, Less, LessEqual, List, Load, MemoryKind,
        Modulo, Multiply, Negate, OperandType, Return, Subtract, Test,
    },
    value::List as ListValue,
    vm::Object,
};

use super::{CallFrame, Cell, Memory, Register, RuntimeError};

pub struct Thread<C> {
    pub handle: JoinHandle<Result<Option<Value<C>>, RuntimeError>>,
}

impl<C: 'static + Chunk + Send + Sync> Thread<C> {
    pub fn new(
        chunk: C,
        cells: Arc<RwLock<Vec<Cell<C>>>>,
        threads: Arc<RwLock<Vec<Thread<C>>>>,
    ) -> Self {
        let name = chunk
            .name()
            .as_ref()
            .map(|name| name.to_string())
            .unwrap_or_else(|| "anonymous".to_string());
        let runner = ThreadRunner {
            chunk,
            threads,
            cells,
        };
        let handle = ThreadBuilder::new()
            .name(name)
            .spawn(|| runner.run())
            .expect("Failed to spawn thread");

        Thread { handle }
    }
}

#[derive(Clone)]
struct ThreadRunner<C> {
    chunk: C,
    threads: Arc<RwLock<Vec<Thread<C>>>>,
    cells: Arc<RwLock<Vec<Cell<C>>>>,
}

impl<C: Chunk> ThreadRunner<C> {
    fn run(self) -> Result<Option<Value<C>>, RuntimeError> {
        let span = span!(Level::INFO, "Thread");
        let _enter = span.enter();

        info!(
            "Starting thread {}",
            self.chunk
                .name()
                .as_ref()
                .map(|name| name.as_ref())
                .unwrap_or_default()
        );

        let mut call_stack = Vec::<CallFrame<C>>::with_capacity(0);
        let mut memory = Memory::<C>::new();

        memory.allocate_registers(self.chunk.register_count());

        let mut call = CallFrame::new(
            Arc::new(self.chunk),
            Address::default(),
            OperandType::NONE,
            0,
        );
        let instructions = call.chunk.instructions().clone();
        let mut highest_ip = 0;

        loop {
            let ip = call.ip;
            call.ip += 1;

            if ip > highest_ip {
                highest_ip = ip;
            }

            assert!(ip < call.chunk.instructions().len(), "IP out of bounds");

            let instruction = instructions[ip];
            let operation = instruction.operation();

            info!("IP = {ip} Run {operation}");

            match operation {
                // NO_OP
                Operation::NO_OP => {
                    warn!("Running NO_OP instruction");
                }

                // LOAD
                Operation::LOAD => {
                    let Load {
                        destination,
                        operand,
                        r#type,
                        jump_next,
                    } = Load::from(instruction);

                    let new_register =
                        get_register_from_address!(operand, r#type, memory, call, operation);
                    let old_register =
                        read_register_mut!(destination.index, memory, call, operation);

                    *old_register = new_register;

                    if jump_next {
                        call.ip += 1;
                    }
                }

                // LIST
                Operation::LIST => {
                    let List {
                        destination,
                        start,
                        end,
                        r#type,
                    } = List::from(instruction);

                    let length = end.index - start.index;
                    let object = match r#type {
                        OperandType::LIST_BOOLEAN => {
                            let mut booleans = Vec::with_capacity(length);

                            for register_index in start.index..=end.index {
                                let boolean =
                                    read_register!(register_index, memory, call, operation)
                                        .as_boolean();

                                booleans.push(boolean);
                            }

                            Object::ValueList(ListValue::<C>::Boolean(booleans))
                        }
                        OperandType::LIST_BYTE => {
                            let mut bytes = Vec::with_capacity(length);

                            for register_index in start.index..=end.index {
                                let byte = read_register!(register_index, memory, call, operation)
                                    .as_byte();

                                bytes.push(byte);
                            }

                            Object::ValueList(ListValue::<C>::Byte(bytes))
                        }
                        OperandType::LIST_CHARACTER => {
                            let mut characters = Vec::with_capacity(length);

                            for register_index in start.index..=end.index {
                                let character =
                                    read_register!(register_index, memory, call, operation)
                                        .as_character();

                                characters.push(character);
                            }

                            Object::ValueList(ListValue::<C>::Character(characters))
                        }
                        OperandType::LIST_FLOAT => {
                            let mut floats = Vec::with_capacity(length);

                            for register_index in start.index..=end.index {
                                let float = read_register!(register_index, memory, call, operation)
                                    .as_float();

                                floats.push(float);
                            }

                            Object::ValueList(ListValue::<C>::Float(floats))
                        }
                        OperandType::LIST_INTEGER => {
                            let mut integers = Vec::with_capacity(length);

                            for register_index in start.index..=end.index {
                                let integer =
                                    read_register!(register_index, memory, call, operation)
                                        .as_integer();

                                integers.push(integer);
                            }

                            Object::ValueList(ListValue::<C>::Integer(integers))
                        }
                        OperandType::LIST_STRING => {
                            let mut string_registers = Vec::with_capacity(length);

                            for register_index in start.index..=end.index {
                                let register =
                                    read_register!(register_index, memory, call, operation);

                                string_registers.push(register);
                            }

                            Object::RegisterList(string_registers)
                        }
                        OperandType::LIST_LIST => {
                            let mut list_registers = Vec::with_capacity(length);

                            for register_index in start.index..=end.index {
                                let register =
                                    read_register!(register_index, memory, call, operation);

                                list_registers.push(register);
                            }

                            Object::RegisterList(list_registers)
                        }
                        OperandType::LIST_FUNCTION => {
                            let mut function_registers = Vec::with_capacity(length);

                            for register_index in start.index..=end.index {
                                let register =
                                    read_register!(register_index, memory, call, operation);

                                function_registers.push(register);
                            }

                            Object::RegisterList(function_registers)
                        }
                        _ => return Err(RuntimeError(operation)),
                    };

                    let object_register = memory.store_object(object);
                    let destination_register =
                        read_register_mut!(destination.index, memory, call, operation);

                    *destination_register = object_register;
                }

                // ADD
                Operation::ADD => {
                    let Add {
                        destination,
                        left,
                        right,
                        r#type,
                    } = Add::from(instruction);

                    let sum_register = match r#type {
                        OperandType::INTEGER => {
                            let left_integer = get_integer!(left, memory, call, operation);
                            let right_integer = get_integer!(right, memory, call, operation);
                            let integer_sum = left_integer.saturating_add(right_integer);

                            Register::integer(integer_sum)
                        }
                        OperandType::STRING => {
                            let left_string = match left.memory {
                                MemoryKind::REGISTER => {
                                    let register =
                                        read_register!(left.index, memory, call, operation);
                                    let object =
                                        read_object!(register.as_index(), memory, operation);

                                    if let Object::String(string) = object {
                                        string
                                    } else {
                                        return Err(RuntimeError(operation));
                                    }
                                }
                                MemoryKind::CONSTANT => read_constant!(left.index, call, operation)
                                    .as_string()
                                    .ok_or(RuntimeError(operation))?,
                                MemoryKind::CELL => todo!(),
                                _ => return Err(RuntimeError(operation)),
                            };
                            let right_string = match right.memory {
                                MemoryKind::REGISTER => {
                                    let register =
                                        read_register!(right.index, memory, call, operation);
                                    let object =
                                        read_object!(register.as_index(), memory, operation);

                                    if let Object::String(string) = object {
                                        string
                                    } else {
                                        return Err(RuntimeError(operation));
                                    }
                                }
                                MemoryKind::CONSTANT => {
                                    read_constant!(right.index, call, operation)
                                        .as_string()
                                        .ok_or(RuntimeError(operation))?
                                }
                                MemoryKind::CELL => todo!(),
                                _ => return Err(RuntimeError(operation)),
                            };
                            let concatenated_string =
                                DustString::from(format!("{left_string}{right_string}"));
                            let string_object = Object::String(concatenated_string);

                            memory.store_object(string_object)
                        }
                        _ => todo!(),
                    };

                    let destination_register =
                        read_register_mut!(destination.index, memory, call, operation);

                    *destination_register = sum_register;
                }

                // SUBTRACT
                Operation::SUBTRACT => {
                    let Subtract {
                        destination,
                        left,
                        right,
                        r#type,
                    } = Subtract::from(instruction);
                }

                // MULTIPLY
                Operation::MULTIPLY => {
                    let Multiply {
                        destination,
                        left,
                        right,
                        r#type,
                    } = Multiply::from(instruction);
                }

                // DIVIDE
                Operation::DIVIDE => {
                    let Divide {
                        destination,
                        left,
                        right,
                        r#type,
                    } = Divide::from(instruction);
                }

                // MODULO
                Operation::MODULO => {
                    let Modulo {
                        destination,
                        left,
                        right,
                        r#type,
                    } = Modulo::from(instruction);
                }

                // EQUAL
                Operation::EQUAL => {
                    let Equal {
                        comparator,
                        left,
                        right,
                        r#type,
                    } = Equal::from(instruction);

                    let is_equal = match r#type {
                        OperandType::INTEGER => {
                            let left_integer = get_integer!(left, memory, call, operation);
                            let right_integer = get_integer!(right, memory, call, operation);

                            left_integer == right_integer
                        }
                        _ => return Err(RuntimeError(operation)),
                    };

                    if is_equal == comparator {
                        call.ip += 1;
                    }
                }

                // LESS
                Operation::LESS => {
                    let Less {
                        comparator,
                        left,
                        right,
                        r#type,
                    } = Less::from(instruction);

                    let is_less = match r#type {
                        OperandType::INTEGER => {
                            let left_integer = get_integer!(left, memory, call, operation);
                            let right_integer = get_integer!(right, memory, call, operation);

                            left_integer < right_integer
                        }
                        _ => return Err(RuntimeError(operation)),
                    };

                    if is_less == comparator {
                        call.ip += 1;
                    }
                }

                // LESS_EQUAL
                Operation::LESS_EQUAL => {
                    let LessEqual {
                        comparator,
                        left,
                        right,
                        r#type,
                    } = LessEqual::from(instruction);
                }

                // TEST
                Operation::TEST => {
                    let Test {
                        comparator,
                        operand,
                    } = Test::from(instruction);
                }

                // NEGATE
                Operation::NEGATE => {
                    let Negate {
                        destination,
                        operand,
                        r#type,
                    } = Negate::from(instruction);
                }

                // CALL
                Operation::CALL => {
                    let Call {
                        destination,
                        function,
                        argument_count,
                        return_type,
                    } = Call::from(instruction);
                    let object = get_value_from_address_by_cloning_objects!(
                        function,
                        OperandType::FUNCTION,
                        memory,
                        call,
                        operation
                    );
                    let function = object.as_function().ok_or(RuntimeError(operation))?.clone();
                    let argument_types = function
                        .r#type()
                        .value_parameters
                        .iter()
                        .map(|parameter| parameter.as_operand_type())
                        .collect::<Vec<_>>();

                    memory.allocate_registers(function.register_count());

                    let new_call = CallFrame::new(
                        function,
                        destination,
                        return_type,
                        call.skipped_registers + call.chunk.register_count(),
                    );
                    let old_call = replace(&mut call, new_call);

                    let first_argument_index = destination.index - argument_count;

                    for (destination_index, (argument_index, r#type)) in
                        (0..).zip((first_argument_index..destination.index).zip(argument_types))
                    {
                        let argument_address = Address::register(argument_index);
                        let argument_register = get_register_from_address!(
                            argument_address,
                            r#type,
                            memory,
                            old_call,
                            operation
                        );
                        let destination_register =
                            read_register_mut!(destination_index, memory, call, operation);

                        *destination_register = argument_register;
                    }

                    call_stack.push(old_call);
                }

                // CALL_NATIVE
                Operation::CALL_NATIVE => {
                    let CallNative {
                        destination,
                        function,
                        argument_count,
                    } = CallNative::<C>::from(instruction);
                }

                // JUMP
                Operation::JUMP => {
                    let Jump {
                        offset,
                        is_positive,
                    } = Jump::from(instruction);

                    if is_positive {
                        call.ip += offset;
                    } else {
                        call.ip -= offset + 1;
                    }
                }

                // RETURN
                Operation::RETURN => {
                    let Return {
                        should_return_value,
                        return_value_address,
                        r#type,
                    } = Return::from(instruction);

                    if call_stack.is_empty() {
                        if should_return_value {
                            let return_value = get_value_from_address_by_replacing_objects!(
                                return_value_address,
                                r#type,
                                memory,
                                call,
                                operation
                            );

                            return Ok(Some(return_value));
                        } else {
                            return Ok(None);
                        }
                    }

                    let resume_call = call_stack.pop().unwrap();

                    if should_return_value {
                        let return_value = get_register_from_address!(
                            return_value_address,
                            r#type,
                            memory,
                            call,
                            operation
                        );
                        let destination_register = read_register_mut!(
                            call.return_address.index,
                            memory,
                            resume_call,
                            operation
                        );

                        *destination_register = return_value;
                    }

                    call = resume_call;
                }
                _ => todo!("Handle operation: {operation}"),
            }
        }
    }
}

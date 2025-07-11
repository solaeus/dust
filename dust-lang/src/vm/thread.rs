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

        loop {
            let ip = call.ip;
            call.ip += 1;

            assert!(ip < call.chunk.instructions().len(), "IP out of bounds");

            let instruction = call.chunk.instructions()[ip];
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
                            let mut strings = Vec::with_capacity(length);

                            for register_index in start.index..=end.index {
                                let object_index =
                                    read_register!(register_index, memory, call, operation)
                                        .as_index();
                                let object =
                                    replace(&mut memory.objects[object_index], Object::Empty);

                                if let Object::String(string) = object {
                                    strings.push(string);
                                } else {
                                    return Err(RuntimeError(operation));
                                }
                            }

                            Object::ValueList(ListValue::<C>::String(strings))
                        }
                        OperandType::LIST_LIST => {
                            let mut lists = Vec::with_capacity(length);

                            for register_index in start.index..=end.index {
                                let object_index =
                                    read_register!(register_index, memory, call, operation)
                                        .as_index();
                                let object =
                                    replace(&mut memory.objects[object_index], Object::Empty);

                                if let Object::ValueList(list) = object {
                                    lists.push(list);
                                } else {
                                    return Err(RuntimeError(operation));
                                }
                            }

                            Object::ValueList(ListValue::<C>::List(lists))
                        }
                        OperandType::LIST_FUNCTION => {
                            let mut functions = Vec::with_capacity(length);

                            for register_index in start.index..=end.index {
                                let object_index =
                                    read_register!(register_index, memory, call, operation)
                                        .as_index();
                                let object =
                                    replace(&mut memory.objects[object_index], Object::Empty);

                                if let Object::Function(function) = object {
                                    functions.push(function);
                                } else {
                                    return Err(RuntimeError(operation));
                                }
                            }

                            Object::ValueList(ListValue::<C>::Function(functions))
                        }
                        _ => todo!(),
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
                        OperandType::BYTE => {
                            let left_byte = get_byte!(left, memory, call, operation);
                            let right_byte = get_byte!(right, memory, call, operation);
                            let byte_sum = left_byte.saturating_add(right_byte);

                            Register::byte(byte_sum)
                        }
                        OperandType::CHARACTER => {
                            let left_character = get_character!(left, memory, call, operation);
                            let right_character = get_character!(right, memory, call, operation);
                            let concatenated_string =
                                DustString::from(format!("{left_character}{right_character}",));
                            let string_object = Object::String(concatenated_string);

                            memory.store_object(string_object)
                        }
                        OperandType::FLOAT => {
                            let left_float = get_float!(left, memory, call, operation);
                            let right_float = get_float!(right, memory, call, operation);
                            let float_sum = left_float + right_float;

                            Register::float(float_sum)
                        }
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

                    let difference_register = match r#type {
                        OperandType::BYTE => {
                            let left_byte = get_byte!(left, memory, call, operation);
                            let right_byte = get_byte!(right, memory, call, operation);
                            let byte_difference = left_byte.saturating_sub(right_byte);

                            Register::byte(byte_difference)
                        }
                        OperandType::FLOAT => {
                            let left_float = get_float!(left, memory, call, operation);
                            let right_float = get_float!(right, memory, call, operation);
                            let float_difference = left_float - right_float;

                            Register::float(float_difference)
                        }
                        OperandType::INTEGER => {
                            let left_integer = get_integer!(left, memory, call, operation);
                            let right_integer = get_integer!(right, memory, call, operation);
                            let integer_difference = left_integer.saturating_sub(right_integer);

                            Register::integer(integer_difference)
                        }
                        _ => return Err(RuntimeError(operation)),
                    };
                    let destination_register =
                        read_register_mut!(destination.index, memory, call, operation);

                    *destination_register = difference_register;
                }

                // MULTIPLY
                Operation::MULTIPLY => {
                    let Multiply {
                        destination,
                        left,
                        right,
                        r#type,
                    } = Multiply::from(instruction);

                    let product_register = match r#type {
                        OperandType::BYTE => {
                            let left_byte = get_byte!(left, memory, call, operation);
                            let right_byte = get_byte!(right, memory, call, operation);
                            let byte_product = left_byte.saturating_mul(right_byte);

                            Register::byte(byte_product)
                        }
                        OperandType::FLOAT => {
                            let left_float = get_float!(left, memory, call, operation);
                            let right_float = get_float!(right, memory, call, operation);
                            let float_product = left_float * right_float;

                            Register::float(float_product)
                        }
                        OperandType::INTEGER => {
                            let left_integer = get_integer!(left, memory, call, operation);
                            let right_integer = get_integer!(right, memory, call, operation);
                            let integer_product = left_integer.saturating_mul(right_integer);

                            Register::integer(integer_product)
                        }
                        _ => return Err(RuntimeError(operation)),
                    };
                    let destination_register =
                        read_register_mut!(destination.index, memory, call, operation);

                    *destination_register = product_register;
                }

                // DIVIDE
                Operation::DIVIDE => {
                    let Divide {
                        destination,
                        left,
                        right,
                        r#type,
                    } = Divide::from(instruction);

                    let quotient_register = match r#type {
                        OperandType::BYTE => {
                            let left_byte = get_byte!(left, memory, call, operation);
                            let right_byte = get_byte!(right, memory, call, operation);

                            if right_byte == 0 {
                                return Err(RuntimeError(operation));
                            }

                            Register::byte(left_byte / right_byte)
                        }
                        OperandType::FLOAT => {
                            let left_float = get_float!(left, memory, call, operation);
                            let right_float = get_float!(right, memory, call, operation);

                            if right_float == 0.0 {
                                return Err(RuntimeError(operation));
                            }

                            Register::float(left_float / right_float)
                        }
                        OperandType::INTEGER => {
                            let left_integer = get_integer!(left, memory, call, operation);
                            let right_integer = get_integer!(right, memory, call, operation);

                            if right_integer == 0 {
                                return Err(RuntimeError(operation));
                            }

                            Register::integer(left_integer / right_integer)
                        }
                        _ => return Err(RuntimeError(operation)),
                    };
                    let destination_register =
                        read_register_mut!(destination.index, memory, call, operation);

                    *destination_register = quotient_register;
                }

                // MODULO
                Operation::MODULO => {
                    let Modulo {
                        destination,
                        left,
                        right,
                        r#type,
                    } = Modulo::from(instruction);

                    let remainder_register = match r#type {
                        OperandType::BYTE => {
                            let left_byte = get_byte!(left, memory, call, operation);
                            let right_byte = get_byte!(right, memory, call, operation);

                            if right_byte == 0 {
                                return Err(RuntimeError(operation));
                            }

                            Register::byte(left_byte % right_byte)
                        }
                        OperandType::FLOAT => {
                            let left_float = get_float!(left, memory, call, operation);
                            let right_float = get_float!(right, memory, call, operation);

                            if right_float == 0.0 {
                                return Err(RuntimeError(operation));
                            }

                            Register::float(left_float % right_float)
                        }
                        OperandType::INTEGER => {
                            let left_integer = get_integer!(left, memory, call, operation);
                            let right_integer = get_integer!(right, memory, call, operation);

                            if right_integer == 0 {
                                return Err(RuntimeError(operation));
                            }

                            Register::integer(left_integer % right_integer)
                        }
                        _ => return Err(RuntimeError(operation)),
                    };
                    let destination_register =
                        read_register_mut!(destination.index, memory, call, operation);

                    *destination_register = remainder_register;
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
                        OperandType::BOOLEAN => {
                            let left_boolean = get_boolean!(left, memory, call, operation);
                            let right_boolean = get_boolean!(right, memory, call, operation);

                            left_boolean == right_boolean
                        }
                        OperandType::BYTE => {
                            let left_byte = get_byte!(left, memory, call, operation);
                            let right_byte = get_byte!(right, memory, call, operation);

                            left_byte == right_byte
                        }
                        OperandType::CHARACTER => {
                            let left_character = get_character!(left, memory, call, operation);
                            let right_character = get_character!(right, memory, call, operation);

                            left_character == right_character
                        }
                        OperandType::FLOAT => {
                            let left_float = get_float!(left, memory, call, operation);
                            let right_float = get_float!(right, memory, call, operation);

                            left_float == right_float
                        }
                        OperandType::INTEGER => {
                            let left_integer = get_integer!(left, memory, call, operation);
                            let right_integer = get_integer!(right, memory, call, operation);

                            left_integer == right_integer
                        }
                        OperandType::STRING => {
                            let left_string = get_string!(left, memory, call, operation);
                            let right_string = get_string!(right, memory, call, operation);

                            left_string == right_string
                        }
                        OperandType::LIST => {
                            let left_list = get_list!(left, memory, call, operation);
                            let right_list = get_list!(right, memory, call, operation);

                            left_list == right_list
                        }
                        OperandType::FUNCTION => {
                            let left_function = get_function!(left, memory, call, operation);
                            let right_function = get_function!(right, memory, call, operation);

                            left_function == right_function
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
                        OperandType::BOOLEAN => {
                            let left_boolean = get_boolean!(left, memory, call, operation);
                            let right_boolean = get_boolean!(right, memory, call, operation);

                            left_boolean && !right_boolean
                        }
                        OperandType::BYTE => {
                            let left_byte = get_byte!(left, memory, call, operation);
                            let right_byte = get_byte!(right, memory, call, operation);

                            left_byte < right_byte
                        }
                        OperandType::CHARACTER => {
                            let left_character = get_character!(left, memory, call, operation);
                            let right_character = get_character!(right, memory, call, operation);

                            left_character < right_character
                        }
                        OperandType::FLOAT => {
                            let left_float = get_float!(left, memory, call, operation);
                            let right_float = get_float!(right, memory, call, operation);

                            left_float < right_float
                        }
                        OperandType::INTEGER => {
                            let left_integer = get_integer!(left, memory, call, operation);
                            let right_integer = get_integer!(right, memory, call, operation);

                            left_integer < right_integer
                        }
                        OperandType::STRING => {
                            let left_string = get_string!(left, memory, call, operation);
                            let right_string = get_string!(right, memory, call, operation);

                            left_string < right_string
                        }
                        OperandType::LIST => {
                            let left_list = get_list!(left, memory, call, operation);
                            let right_list = get_list!(right, memory, call, operation);

                            left_list < right_list
                        }
                        OperandType::FUNCTION => {
                            let left_function = get_function!(left, memory, call, operation);
                            let right_function = get_function!(right, memory, call, operation);

                            left_function < right_function
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

                    let is_less_or_equal = match r#type {
                        OperandType::BOOLEAN => {
                            let left_boolean = get_boolean!(left, memory, call, operation);
                            let right_boolean = get_boolean!(right, memory, call, operation);

                            left_boolean <= right_boolean
                        }
                        OperandType::BYTE => {
                            let left_byte = get_byte!(left, memory, call, operation);
                            let right_byte = get_byte!(right, memory, call, operation);

                            left_byte <= right_byte
                        }
                        OperandType::CHARACTER => {
                            let left_character = get_character!(left, memory, call, operation);
                            let right_character = get_character!(right, memory, call, operation);

                            left_character <= right_character
                        }
                        OperandType::FLOAT => {
                            let left_float = get_float!(left, memory, call, operation);
                            let right_float = get_float!(right, memory, call, operation);

                            left_float <= right_float
                        }
                        OperandType::INTEGER => {
                            let left_integer = get_integer!(left, memory, call, operation);
                            let right_integer = get_integer!(right, memory, call, operation);

                            left_integer <= right_integer
                        }
                        OperandType::STRING => {
                            let left_string = get_string!(left, memory, call, operation);
                            let right_string = get_string!(right, memory, call, operation);

                            left_string <= right_string
                        }
                        OperandType::LIST => {
                            let left_list = get_list!(left, memory, call, operation);
                            let right_list = get_list!(right, memory, call, operation);

                            left_list <= right_list
                        }
                        OperandType::FUNCTION => {
                            let left_function = get_function!(left, memory, call, operation);
                            let right_function = get_function!(right, memory, call, operation);

                            left_function <= right_function
                        }
                        _ => return Err(RuntimeError(operation)),
                    };

                    if is_less_or_equal == comparator {
                        call.ip += 1;
                    }
                }

                // TEST
                Operation::TEST => {
                    let Test {
                        comparator,
                        operand,
                    } = Test::from(instruction);

                    let is_true = get_boolean!(operand, memory, call, operation);

                    if is_true == comparator {
                        call.ip += 1;
                    }
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
                    let prototype = Arc::clone(get_function!(function, memory, call, operation));
                    let argument_types = prototype
                        .r#type()
                        .value_parameters
                        .iter()
                        .map(|parameter| parameter.as_operand_type())
                        .collect::<Vec<_>>();

                    memory.allocate_registers(prototype.register_count());

                    let new_call = CallFrame::new(
                        prototype,
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
                                operation,
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

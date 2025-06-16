use std::{
    mem::replace,
    sync::{Arc, RwLock},
    thread::{Builder, JoinHandle},
};

use tracing::{Level, error, info, span, warn};

use crate::{
    Address, Chunk, DustString, FullChunk, Operation, Value,
    instruction::{
        Add, Call, CallNative, Close, Divide, Equal, Jump, Less, LessEqual, List, Load, MemoryKind,
        Modulo, Multiply, Negate, OperandType, Return, Subtract, Test,
    },
    value::List as ListValue,
};

use super::{CallFrame, Cell, CellValue, HeapSlot, Memory, macros::*};

pub struct Thread<C> {
    pub handle: JoinHandle<Option<Value<C>>>,
}

impl<C: 'static + Chunk + Send + Sync> Thread<C> {
    pub fn new(
        chunk: Arc<C>,
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
        let handle = Builder::new()
            .name(name)
            .spawn(|| runner.run())
            .expect("Failed to spawn thread");

        Thread { handle }
    }
}

#[derive(Clone)]
struct ThreadRunner<C> {
    chunk: Arc<C>,
    threads: Arc<RwLock<Vec<Thread<C>>>>,
    cells: Arc<RwLock<Vec<Cell<C>>>>,
}

impl<C: Chunk> ThreadRunner<C> {
    fn run(mut self) -> Option<Value<C>> {
        let span = span!(Level::INFO, "Thread");
        let _enter = span.enter();

        info!(
            "Starting thread {}",
            self.chunk
                .name()
                .as_ref()
                .map(|name| name.as_str())
                .unwrap_or_default()
        );

        let mut call_stack = Vec::<CallFrame<C>>::new();
        let mut memory_stack = Vec::<Memory<C>>::new();

        let mut call = CallFrame::new(Arc::clone(&self.chunk), Address::default());
        let mut memory = Memory::new(&*call.chunk);

        loop {
            let instructions = &call.chunk.instructions();
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
                Operation::LOAD => {
                    let Load {
                        destination,
                        operand,
                        r#type,
                        jump_next,
                    } = Load::from(&instruction);

                    match r#type {
                        OperandType::BOOLEAN => {
                            let boolean = get_boolean!(operand, memory, call.chunk, self.cells);

                            set_boolean!(destination, memory, self.cells, boolean);
                        }
                        OperandType::BYTE => {
                            let byte = get_byte!(operand, memory, call.chunk, self.cells);

                            set_byte!(destination, memory, self.cells, byte);
                        }
                        OperandType::CHARACTER => {
                            let character = get_character!(operand, memory, call.chunk, self.cells);

                            set_character!(destination, memory, self.cells, character);
                        }
                        OperandType::FLOAT => {
                            let float = get_float!(operand, memory, call.chunk, self.cells);

                            set_float!(destination, memory, self.cells, float);
                        }
                        OperandType::INTEGER => {
                            let integer = get_integer!(operand, memory, call.chunk, self.cells);

                            set_integer!(destination, memory, self.cells, integer);
                        }
                        OperandType::STRING => {
                            let string = get_string!(operand, memory, call.chunk, self.cells);

                            set_string!(destination, memory, self.cells, string);
                        }
                        OperandType::LIST_BOOLEAN
                        | OperandType::LIST_BYTE
                        | OperandType::LIST_CHARACTER
                        | OperandType::LIST_FLOAT
                        | OperandType::LIST_INTEGER
                        | OperandType::LIST_STRING
                        | OperandType::LIST_LIST
                        | OperandType::LIST_FUNCTION => {
                            let list = get_list!(operand, memory, call.chunk, self.cells);

                            set_list!(destination, memory, self.cells, list);
                        }
                        OperandType::FUNCTION => {
                            let function = get_function!(operand, memory, &call.chunk, self.cells);

                            set_function!(destination, memory, self.cells, function);
                        }
                        _ => unreachable!(),
                    }

                    if jump_next {
                        call.ip += 1;
                    }
                }
                Operation::LIST => {
                    let List {
                        destination,
                        start,
                        end,
                        r#type,
                    } = List::from(&instruction);

                    let list = match r#type.list_item_type() {
                        Some(OperandType::BOOLEAN) => {
                            let mut booleans = Vec::new();

                            for index in start.index..=end.index {
                                let heap_slot = take_heap_slot!(index, memory, booleans);

                                match heap_slot {
                                    HeapSlot::Closed => continue,
                                    HeapSlot::Open(value) => booleans.push(value),
                                }
                            }

                            ListValue::Boolean(booleans)
                        }
                        Some(OperandType::BYTE) => {
                            let mut bytes = Vec::new();

                            for index in start.index..=end.index {
                                let heap_slot = take_heap_slot!(index, memory, bytes);

                                match heap_slot {
                                    HeapSlot::Closed => continue,
                                    HeapSlot::Open(value) => bytes.push(value),
                                }
                            }

                            ListValue::Byte(bytes)
                        }
                        Some(OperandType::CHARACTER) => {
                            let mut characters = Vec::new();

                            for index in start.index..=end.index {
                                let heap_slot = take_heap_slot!(index, memory, characters);

                                match heap_slot {
                                    HeapSlot::Closed => continue,
                                    HeapSlot::Open(value) => characters.push(value),
                                }
                            }

                            ListValue::Character(characters)
                        }
                        Some(OperandType::FLOAT) => {
                            let mut floats = Vec::new();

                            for index in start.index..=end.index {
                                let heap_slot = take_heap_slot!(index, memory, floats);

                                match heap_slot {
                                    HeapSlot::Closed => continue,
                                    HeapSlot::Open(value) => floats.push(value),
                                }
                            }

                            ListValue::Float(floats)
                        }
                        Some(OperandType::INTEGER) => {
                            let mut integers = Vec::new();

                            for index in start.index..=end.index {
                                let heap_slot = take_heap_slot!(index, memory, integers);

                                match heap_slot {
                                    HeapSlot::Closed => continue,
                                    HeapSlot::Open(value) => integers.push(value),
                                }
                            }

                            ListValue::Integer(integers)
                        }
                        Some(OperandType::STRING) => {
                            let mut strings = Vec::new();

                            for index in start.index..=end.index {
                                let heap_slot = take_heap_slot!(index, memory, strings);

                                match heap_slot {
                                    HeapSlot::Closed => continue,
                                    HeapSlot::Open(value) => strings.push(value),
                                }
                            }

                            ListValue::String(strings)
                        }
                        Some(OperandType::FUNCTION) => {
                            let mut functions = Vec::new();

                            for index in start.index..=end.index {
                                let heap_slot = take_heap_slot!(index, memory, functions);

                                match heap_slot {
                                    HeapSlot::Closed => continue,
                                    HeapSlot::Open(value) => functions.push(value),
                                }
                            }

                            ListValue::Function(functions)
                        }
                        Some(invalid) => invalid.invalid_panic(),
                        None => unreachable!(),
                    };

                    set_list!(destination, memory, self.cells, list);
                }
                Operation::CLOSE => {
                    let Close { from, to, r#type } = Close::from(&instruction);

                    match r#type {
                        OperandType::BOOLEAN => {
                            for index in from.index..=to.index {
                                close_heap_slot!(index, memory, booleans)
                            }
                        }
                        OperandType::BYTE => {
                            for index in from.index..=to.index {
                                close_heap_slot!(index, memory, bytes)
                            }
                        }
                        OperandType::CHARACTER => {
                            for index in from.index..=to.index {
                                close_heap_slot!(index, memory, characters)
                            }
                        }
                        OperandType::FLOAT => {
                            for index in from.index..=to.index {
                                close_heap_slot!(index, memory, floats)
                            }
                        }
                        OperandType::INTEGER => {
                            for index in from.index..=to.index {
                                close_heap_slot!(index, memory, integers)
                            }
                        }
                        OperandType::STRING => {
                            for index in from.index..=to.index {
                                close_heap_slot!(index, memory, strings)
                            }
                        }
                        OperandType::LIST_BOOLEAN
                        | OperandType::LIST_BYTE
                        | OperandType::LIST_CHARACTER
                        | OperandType::LIST_FLOAT
                        | OperandType::LIST_INTEGER
                        | OperandType::LIST_STRING
                        | OperandType::LIST_LIST
                        | OperandType::LIST_FUNCTION => {
                            for index in from.index..=to.index {
                                close_heap_slot!(index, memory, lists)
                            }
                        }
                        OperandType::FUNCTION => {
                            for index in from.index..=to.index {
                                close_heap_slot!(index, memory, functions)
                            }
                        }
                        invalid => invalid.invalid_panic(),
                    }
                }
                Operation::RETURN => {
                    let Return {
                        should_return_value,
                        return_value_address,
                        r#type,
                    } = Return::from(&instruction);

                    if call_stack.is_empty() {
                        if should_return_value {
                            let return_value = match r#type {
                                OperandType::BOOLEAN => {
                                    let value = get_boolean!(
                                        return_value_address,
                                        memory,
                                        call.chunk,
                                        self.cells
                                    );
                                    Value::Boolean(value)
                                }
                                OperandType::BYTE => {
                                    let value = get_byte!(
                                        return_value_address,
                                        memory,
                                        call.chunk,
                                        self.cells
                                    );
                                    Value::Byte(value)
                                }
                                OperandType::CHARACTER => {
                                    let value = get_character!(
                                        return_value_address,
                                        memory,
                                        call.chunk,
                                        self.cells
                                    );
                                    Value::Character(value)
                                }
                                OperandType::FLOAT => {
                                    let value = get_float!(
                                        return_value_address,
                                        memory,
                                        call.chunk,
                                        self.cells
                                    );
                                    Value::Float(value)
                                }
                                OperandType::INTEGER => {
                                    let value = get_integer!(
                                        return_value_address,
                                        memory,
                                        call.chunk,
                                        self.cells
                                    );
                                    Value::Integer(value)
                                }
                                OperandType::STRING => {
                                    let value = get_string!(
                                        return_value_address,
                                        memory,
                                        call.chunk,
                                        self.cells
                                    );
                                    Value::String(value)
                                }
                                OperandType::LIST_BOOLEAN
                                | OperandType::LIST_BYTE
                                | OperandType::LIST_CHARACTER
                                | OperandType::LIST_FLOAT
                                | OperandType::LIST_INTEGER
                                | OperandType::LIST_STRING
                                | OperandType::LIST_LIST
                                | OperandType::LIST_FUNCTION => {
                                    let value = get_list!(
                                        return_value_address,
                                        memory,
                                        call.chunk,
                                        self.cells
                                    );
                                    Value::List(value)
                                }
                                OperandType::FUNCTION => {
                                    let value = get_function!(
                                        return_value_address,
                                        memory,
                                        &call.chunk,
                                        self.cells
                                    );
                                    Value::Function(value)
                                }
                                invalid => invalid.invalid_panic(),
                            };

                            return Some(return_value);
                        } else {
                            return None;
                        }
                    }
                }
                Operation::ADD => {
                    let Add {
                        destination,
                        left,
                        right,
                        r#type,
                    } = Add::from(&instruction);

                    match r#type {
                        OperandType::BYTE => {
                            let left = get_byte!(left, memory, call.chunk, self.cells);
                            let right = get_byte!(right, memory, call.chunk, self.cells);
                            let sum = left.saturating_add(right);

                            set_byte!(destination, memory, self.cells, sum);
                        }
                        OperandType::CHARACTER => {
                            let left = get_character!(left, memory, call.chunk, self.cells);
                            let right = get_character!(right, memory, call.chunk, self.cells);
                            let mut sum = DustString::new();

                            sum.push(left);
                            sum.push(right);

                            set_string!(destination, memory, self.cells, sum);
                        }
                        OperandType::FLOAT => {
                            let left = get_float!(left, memory, call.chunk, self.cells);
                            let right = get_float!(right, memory, call.chunk, self.cells);
                            let sum = left + right;

                            set_float!(destination, memory, self.cells, sum);
                        }
                        OperandType::INTEGER => {
                            let left = get_integer!(left, memory, call.chunk, self.cells);
                            let right = get_integer!(right, memory, call.chunk, self.cells);
                            let sum = left.saturating_add(right);

                            set_integer!(destination, memory, self.cells, sum);
                        }
                        invalid => invalid.invalid_panic(),
                    }
                }
                _ => panic!("Handle operation: {operation}"),
            }
        }
    }
}

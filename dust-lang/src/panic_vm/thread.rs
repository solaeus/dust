use std::{
    mem::replace,
    sync::{Arc, RwLock},
    thread::{Builder, JoinHandle},
};

use tracing::{Level, info, span, warn};

use crate::{
    Address, Chunk, DustString, Operation, Value,
    instruction::{
        Add, Call, CallNative, Close, Divide, Equal, Jump, Less, LessEqual, List, Load, MemoryKind,
        Modulo, Multiply, Negate, OperandType, Return, Subtract, Test,
    },
    invalid_operand_type_panic,
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
    fn run(self) -> Option<Value<C>> {
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

        let mut call_stack = Vec::<CallFrame<C>>::new();
        let mut memory_stack = Vec::<Memory<C>>::new();

        let mut call = CallFrame::new(
            Arc::clone(&self.chunk),
            Address::default(),
            OperandType::NONE,
        );
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

                // LIST
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
                        Some(OperandType::LIST) => {
                            let mut lists = Vec::new();

                            for index in start.index..=end.index {
                                let heap_slot = take_heap_slot!(index, memory, lists);

                                match heap_slot {
                                    HeapSlot::Closed => continue,
                                    HeapSlot::Open(value) => lists.push(value),
                                }
                            }

                            ListValue::List(lists)
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
                        Some(invalid) => invalid_operand_type_panic!(invalid, operation),
                        None => unreachable!(),
                    };

                    set_list!(destination, memory, self.cells, list);
                }

                // CLOSE
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
                        invalid => invalid_operand_type_panic!(invalid, operation),
                    }
                }

                // ADD
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

                            sum.inner.push(left);
                            sum.inner.push(right);

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
                        OperandType::STRING => {
                            let left = get_string!(left, memory, call.chunk, self.cells);
                            let right = get_string!(right, memory, call.chunk, self.cells);
                            let mut sum = DustString::new();

                            sum.inner.push_str(&left);
                            sum.inner.push_str(&right);

                            set_string!(destination, memory, self.cells, sum);
                        }
                        OperandType::CHARACTER_STRING => {
                            let left = get_character!(left, memory, call.chunk, self.cells);
                            let right = get_string!(right, memory, call.chunk, self.cells);
                            let mut sum = DustString::new();

                            sum.inner.push(left);
                            sum.inner.push_str(&right);

                            set_string!(destination, memory, self.cells, sum);
                        }
                        OperandType::STRING_CHARACTER => {
                            let left = get_string!(left, memory, call.chunk, self.cells);
                            let right = get_character!(right, memory, call.chunk, self.cells);
                            let mut sum = DustString::new();

                            sum.inner.push_str(&left);
                            sum.inner.push(right);

                            set_string!(destination, memory, self.cells, sum);
                        }
                        invalid => invalid_operand_type_panic!(invalid, operation),
                    }
                }

                // SUBTRACT
                Operation::SUBTRACT => {
                    let Subtract {
                        destination,
                        left,
                        right,
                        r#type,
                    } = Subtract::from(&instruction);

                    match r#type {
                        OperandType::BYTE => {
                            let left = get_byte!(left, memory, call.chunk, self.cells);
                            let right = get_byte!(right, memory, call.chunk, self.cells);
                            let difference = left.saturating_sub(right);

                            set_byte!(destination, memory, self.cells, difference);
                        }
                        OperandType::FLOAT => {
                            let left = get_float!(left, memory, call.chunk, self.cells);
                            let right = get_float!(right, memory, call.chunk, self.cells);
                            let difference = left - right;

                            set_float!(destination, memory, self.cells, difference);
                        }
                        OperandType::INTEGER => {
                            let left = get_integer!(left, memory, call.chunk, self.cells);
                            let right = get_integer!(right, memory, call.chunk, self.cells);
                            let difference = left.saturating_sub(right);

                            set_integer!(destination, memory, self.cells, difference);
                        }
                        invalid => invalid_operand_type_panic!(invalid, operation),
                    }
                }

                // MULTIPLY
                Operation::MULTIPLY => {
                    let Multiply {
                        destination,
                        left,
                        right,
                        r#type,
                    } = Multiply::from(&instruction);

                    match r#type {
                        OperandType::BYTE => {
                            let left = get_byte!(left, memory, call.chunk, self.cells);
                            let right = get_byte!(right, memory, call.chunk, self.cells);
                            let product = left.saturating_mul(right);

                            set_byte!(destination, memory, self.cells, product);
                        }
                        OperandType::FLOAT => {
                            let left = get_float!(left, memory, call.chunk, self.cells);
                            let right = get_float!(right, memory, call.chunk, self.cells);
                            let product = left * right;

                            set_float!(destination, memory, self.cells, product);
                        }
                        OperandType::INTEGER => {
                            let left = get_integer!(left, memory, call.chunk, self.cells);
                            let right = get_integer!(right, memory, call.chunk, self.cells);
                            let product = left.saturating_mul(right);

                            set_integer!(destination, memory, self.cells, product);
                        }
                        invalid => invalid_operand_type_panic!(invalid, operation),
                    }
                }

                // DIVIDE
                Operation::DIVIDE => {
                    let Divide {
                        destination,
                        left,
                        right,
                        r#type,
                    } = Divide::from(&instruction);

                    match r#type {
                        OperandType::BYTE => {
                            let left = get_byte!(left, memory, call.chunk, self.cells);
                            let right = get_byte!(right, memory, call.chunk, self.cells);
                            let quotient = left.saturating_div(right);

                            set_byte!(destination, memory, self.cells, quotient);
                        }
                        OperandType::FLOAT => {
                            let left = get_float!(left, memory, call.chunk, self.cells);
                            let right = get_float!(right, memory, call.chunk, self.cells);
                            let quotient = left / right;

                            set_float!(destination, memory, self.cells, quotient);
                        }
                        OperandType::INTEGER => {
                            let left = get_integer!(left, memory, call.chunk, self.cells);
                            let right = get_integer!(right, memory, call.chunk, self.cells);
                            let quotient = left.saturating_div(right);

                            set_integer!(destination, memory, self.cells, quotient);
                        }
                        invalid => invalid_operand_type_panic!(invalid, operation),
                    }
                }

                // MODULO
                Operation::MODULO => {
                    let Modulo {
                        destination,
                        left,
                        right,
                        r#type,
                    } = Modulo::from(&instruction);

                    match r#type {
                        OperandType::BYTE => {
                            let left = get_byte!(left, memory, call.chunk, self.cells);
                            let right = get_byte!(right, memory, call.chunk, self.cells);
                            let remainder = left % right;

                            set_byte!(destination, memory, self.cells, remainder);
                        }
                        OperandType::FLOAT => {
                            let left = get_float!(left, memory, call.chunk, self.cells);
                            let right = get_float!(right, memory, call.chunk, self.cells);
                            let remainder = left % right;

                            set_float!(destination, memory, self.cells, remainder);
                        }
                        OperandType::INTEGER => {
                            let left = get_integer!(left, memory, call.chunk, self.cells);
                            let right = get_integer!(right, memory, call.chunk, self.cells);
                            let remainder = left % right;

                            set_integer!(destination, memory, self.cells, remainder);
                        }
                        invalid => invalid_operand_type_panic!(invalid, operation),
                    }
                }

                // EQUAL
                Operation::EQUAL => {
                    let Equal {
                        comparator,
                        left,
                        right,
                        r#type,
                    } = Equal::from(&instruction);

                    let is_equal = match r#type {
                        OperandType::BOOLEAN => {
                            let left = get_boolean!(left, memory, call.chunk, self.cells);
                            let right = get_boolean!(right, memory, call.chunk, self.cells);

                            left == right
                        }
                        OperandType::BYTE => {
                            let left = get_byte!(left, memory, call.chunk, self.cells);
                            let right = get_byte!(right, memory, call.chunk, self.cells);

                            left == right
                        }
                        OperandType::CHARACTER => {
                            let left = get_character!(left, memory, call.chunk, self.cells);
                            let right = get_character!(right, memory, call.chunk, self.cells);

                            left == right
                        }
                        OperandType::FLOAT => {
                            let left = get_float!(left, memory, call.chunk, self.cells);
                            let right = get_float!(right, memory, call.chunk, self.cells);

                            left == right
                        }
                        OperandType::INTEGER => {
                            let left = get_integer!(left, memory, call.chunk, self.cells);
                            let right = get_integer!(right, memory, call.chunk, self.cells);

                            left == right
                        }
                        OperandType::STRING => {
                            let left = get_string!(left, memory, call.chunk, self.cells);
                            let right = get_string!(right, memory, call.chunk, self.cells);

                            left == right
                        }
                        OperandType::LIST_BOOLEAN
                        | OperandType::LIST_BYTE
                        | OperandType::LIST_CHARACTER
                        | OperandType::LIST_FLOAT
                        | OperandType::LIST_INTEGER
                        | OperandType::LIST_STRING
                        | OperandType::LIST_LIST
                        | OperandType::LIST_FUNCTION => {
                            let left = get_list!(left, memory, call.chunk, self.cells);
                            let right = get_list!(right, memory, call.chunk, self.cells);

                            left == right
                        }
                        OperandType::FUNCTION => {
                            let left = get_function!(left, memory, &call.chunk, self.cells);
                            let right = get_function!(right, memory, &call.chunk, self.cells);

                            Arc::ptr_eq(&left, &right)
                        }
                        invalid => invalid_operand_type_panic!(invalid, operation),
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
                    } = Less::from(&instruction);

                    let is_less = match r#type {
                        OperandType::BOOLEAN => {
                            let left = get_boolean!(left, memory, call.chunk, self.cells);
                            let right = get_boolean!(right, memory, call.chunk, self.cells);

                            !left && right
                        }
                        OperandType::BYTE => {
                            let left = get_byte!(left, memory, call.chunk, self.cells);
                            let right = get_byte!(right, memory, call.chunk, self.cells);

                            left < right
                        }
                        OperandType::CHARACTER => {
                            let left = get_character!(left, memory, call.chunk, self.cells);
                            let right = get_character!(right, memory, call.chunk, self.cells);

                            left < right
                        }
                        OperandType::FLOAT => {
                            let left = get_float!(left, memory, call.chunk, self.cells);
                            let right = get_float!(right, memory, call.chunk, self.cells);

                            left < right
                        }
                        OperandType::INTEGER => {
                            let left = get_integer!(left, memory, call.chunk, self.cells);
                            let right = get_integer!(right, memory, call.chunk, self.cells);

                            left < right
                        }
                        OperandType::STRING => {
                            let left = get_string!(left, memory, call.chunk, self.cells);
                            let right = get_string!(right, memory, call.chunk, self.cells);

                            left < right
                        }
                        OperandType::LIST_BOOLEAN
                        | OperandType::LIST_BYTE
                        | OperandType::LIST_CHARACTER
                        | OperandType::LIST_FLOAT
                        | OperandType::LIST_INTEGER
                        | OperandType::LIST_STRING
                        | OperandType::LIST_LIST
                        | OperandType::LIST_FUNCTION => {
                            let left = get_list!(left, memory, call.chunk, self.cells);
                            let right = get_list!(right, memory, call.chunk, self.cells);

                            left < right
                        }
                        OperandType::FUNCTION => {
                            let left = get_function!(left, memory, &call.chunk, self.cells);
                            let right = get_function!(right, memory, &call.chunk, self.cells);

                            Arc::ptr_eq(&left, &right)
                        }
                        invalid => invalid_operand_type_panic!(invalid, operation),
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
                    } = LessEqual::from(&instruction);

                    let is_less_equal = match r#type {
                        OperandType::BOOLEAN => {
                            let left = get_boolean!(left, memory, call.chunk, self.cells);
                            let right = get_boolean!(right, memory, call.chunk, self.cells);

                            left <= right
                        }
                        OperandType::BYTE => {
                            let left = get_byte!(left, memory, call.chunk, self.cells);
                            let right = get_byte!(right, memory, call.chunk, self.cells);

                            left <= right
                        }
                        OperandType::CHARACTER => {
                            let left = get_character!(left, memory, call.chunk, self.cells);
                            let right = get_character!(right, memory, call.chunk, self.cells);

                            left <= right
                        }
                        OperandType::FLOAT => {
                            let left = get_float!(left, memory, call.chunk, self.cells);
                            let right = get_float!(right, memory, call.chunk, self.cells);

                            left <= right
                        }
                        OperandType::INTEGER => {
                            let left = get_integer!(left, memory, call.chunk, self.cells);
                            let right = get_integer!(right, memory, call.chunk, self.cells);

                            left <= right
                        }
                        OperandType::STRING => {
                            let left = get_string!(left, memory, call.chunk, self.cells);
                            let right = get_string!(right, memory, call.chunk, self.cells);

                            left <= right
                        }
                        OperandType::LIST_BOOLEAN
                        | OperandType::LIST_BYTE
                        | OperandType::LIST_CHARACTER
                        | OperandType::LIST_FLOAT
                        | OperandType::LIST_INTEGER
                        | OperandType::LIST_STRING
                        | OperandType::LIST_LIST
                        | OperandType::LIST_FUNCTION => {
                            let left = get_list!(left, memory, call.chunk, self.cells);
                            let right = get_list!(right, memory, call.chunk, self.cells);

                            left <= right
                        }
                        OperandType::FUNCTION => {
                            let left = get_function!(left, memory, &call.chunk, self.cells);
                            let right = get_function!(right, memory, &call.chunk, self.cells);

                            Arc::ptr_eq(&left, &right)
                        }
                        invalid => invalid_operand_type_panic!(invalid, operation),
                    };

                    if is_less_equal == comparator {
                        call.ip += 1;
                    }
                }

                // TEST
                Operation::TEST => {
                    let Test {
                        comparator,
                        operand,
                    } = Test::from(&instruction);

                    let boolean = get_boolean!(operand, memory, call.chunk, self.cells);

                    if boolean == comparator {
                        call.ip += 1;
                    }
                }

                // NEGATE
                Operation::NEGATE => {
                    let Negate {
                        destination,
                        operand,
                        r#type,
                    } = Negate::from(&instruction);

                    match r#type {
                        OperandType::BOOLEAN => {
                            let boolean = get_boolean!(operand, memory, call.chunk, self.cells);

                            set_boolean!(destination, memory, self.cells, !boolean);
                        }
                        OperandType::BYTE => {
                            let byte = get_byte!(operand, memory, call.chunk, self.cells);

                            set_byte!(destination, memory, self.cells, !byte);
                        }
                        OperandType::FLOAT => {
                            let float = get_float!(operand, memory, call.chunk, self.cells);

                            set_float!(destination, memory, self.cells, -float);
                        }
                        OperandType::INTEGER => {
                            let integer = get_integer!(operand, memory, call.chunk, self.cells);

                            set_integer!(destination, memory, self.cells, -integer);
                        }
                        invalid => invalid_operand_type_panic!(invalid, operation),
                    }
                }

                // CALL
                Operation::CALL => {
                    let Call {
                        destination,
                        function,
                        argument_list_index,
                        return_type,
                    } = Call::from(&instruction);

                    let chunk = Arc::clone(&call.chunk);
                    let arguments_list = chunk.arguments();
                    let index = argument_list_index as usize;

                    assert!(
                        index < arguments_list.len(),
                        "Argument list index out of bounds"
                    );

                    let arguments = &arguments_list[index];
                    let function = get_function!(function, memory, &call.chunk, self.cells);

                    let mut new_memory = Memory::new(&*function);
                    let new_call = CallFrame::new(function, destination, return_type);

                    for ((argument_address, r#type), parameter_address) in
                        arguments.iter().zip(new_call.chunk.parameters().iter())
                    {
                        match *r#type {
                            OperandType::BOOLEAN => {
                                let boolean =
                                    get_boolean!(argument_address, memory, call.chunk, self.cells);

                                set_boolean!(parameter_address, new_memory, self.cells, boolean);
                            }
                            OperandType::BYTE => {
                                let byte =
                                    get_byte!(argument_address, memory, call.chunk, self.cells);

                                set_byte!(parameter_address, new_memory, self.cells, byte);
                            }
                            OperandType::CHARACTER => {
                                let character = get_character!(
                                    argument_address,
                                    memory,
                                    call.chunk,
                                    self.cells
                                );

                                set_character!(
                                    parameter_address,
                                    new_memory,
                                    self.cells,
                                    character
                                );
                            }
                            OperandType::FLOAT => {
                                let float =
                                    get_float!(argument_address, memory, call.chunk, self.cells);

                                set_float!(parameter_address, new_memory, self.cells, float);
                            }
                            OperandType::INTEGER => {
                                let integer =
                                    get_integer!(argument_address, memory, call.chunk, self.cells);

                                set_integer!(parameter_address, new_memory, self.cells, integer);
                            }
                            OperandType::STRING => {
                                let string =
                                    get_string!(argument_address, memory, call.chunk, self.cells);

                                set_string!(parameter_address, new_memory, self.cells, string);
                            }
                            OperandType::LIST_BOOLEAN
                            | OperandType::LIST_BYTE
                            | OperandType::LIST_CHARACTER
                            | OperandType::LIST_FLOAT
                            | OperandType::LIST_INTEGER
                            | OperandType::LIST_STRING
                            | OperandType::LIST_LIST
                            | OperandType::LIST_FUNCTION => {
                                let list =
                                    get_list!(argument_address, memory, call.chunk, self.cells);

                                set_list!(parameter_address, new_memory, self.cells, list);
                            }
                            OperandType::FUNCTION => {
                                let function = get_function!(
                                    argument_address,
                                    memory,
                                    &call.chunk,
                                    self.cells
                                );

                                set_function!(parameter_address, new_memory, self.cells, function);
                            }
                            invalid => invalid_operand_type_panic!(invalid, operation),
                        }
                    }

                    call_stack.push(replace(&mut call, new_call));
                    memory_stack.push(replace(&mut memory, new_memory));
                }

                // CALL_NATIVE
                Operation::CALL_NATIVE => {
                    let CallNative {
                        destination,
                        function,
                        argument_list_index,
                    } = CallNative::<C>::from(&instruction);

                    let chunk = Arc::clone(&call.chunk);
                    let arguments_list = chunk.arguments();
                    let index = argument_list_index as usize;

                    assert!(
                        index < arguments_list.len(),
                        "Argument list index out of bounds"
                    );

                    let arguments = &arguments_list[index];

                    function.call(
                        destination,
                        arguments,
                        &mut call,
                        &mut memory,
                        &self.cells,
                        &self.threads,
                    );
                }

                // JUMP
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

                // RETURN
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
                                    let value = take_or_decode_boolean!(
                                        return_value_address,
                                        memory,
                                        call.chunk,
                                        self.cells
                                    );
                                    Value::Boolean(value)
                                }
                                OperandType::BYTE => {
                                    let value = take_or_decode_byte!(
                                        return_value_address,
                                        memory,
                                        call.chunk,
                                        self.cells
                                    );
                                    Value::Byte(value)
                                }
                                OperandType::CHARACTER => {
                                    let value = take_or_get_character!(
                                        return_value_address,
                                        memory,
                                        call.chunk,
                                        self.cells
                                    );
                                    Value::Character(value)
                                }
                                OperandType::FLOAT => {
                                    let value = take_or_get_float!(
                                        return_value_address,
                                        memory,
                                        call.chunk,
                                        self.cells
                                    );
                                    Value::Float(value)
                                }
                                OperandType::INTEGER => {
                                    let value = take_or_get_integer!(
                                        return_value_address,
                                        memory,
                                        call.chunk,
                                        self.cells
                                    );
                                    Value::Integer(value)
                                }
                                OperandType::STRING => {
                                    let value = take_or_get_string!(
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
                                    let value = take_or_get_list!(
                                        return_value_address,
                                        memory,
                                        call.chunk,
                                        self.cells
                                    );
                                    Value::List(value)
                                }
                                OperandType::FUNCTION => {
                                    let value = take_or_get_function!(
                                        return_value_address,
                                        memory,
                                        &call.chunk,
                                        self.cells
                                    );
                                    Value::Function(value)
                                }
                                invalid => invalid_operand_type_panic!(invalid, operation),
                            };

                            return Some(return_value);
                        } else {
                            return None;
                        }
                    }

                    let mut old_memory = replace(
                        &mut memory,
                        memory_stack.pop().expect("Memory stack underflow"),
                    );
                    let old_call =
                        replace(&mut call, call_stack.pop().expect("Call stack underflow"));

                    if should_return_value {
                        match r#type {
                            OperandType::BOOLEAN => {
                                let boolean = take_or_decode_boolean!(
                                    return_value_address,
                                    old_memory,
                                    old_call.chunk,
                                    self.cells
                                );

                                set_boolean!(old_call.return_address, memory, self.cells, boolean);
                            }
                            OperandType::BYTE => {
                                let byte = take_or_decode_byte!(
                                    return_value_address,
                                    old_memory,
                                    old_call.chunk,
                                    self.cells
                                );

                                set_byte!(old_call.return_address, memory, self.cells, byte);
                            }
                            OperandType::CHARACTER => {
                                let character = take_or_get_character!(
                                    return_value_address,
                                    old_memory,
                                    old_call.chunk,
                                    self.cells
                                );

                                set_character!(
                                    old_call.return_address,
                                    memory,
                                    self.cells,
                                    character
                                );
                            }
                            OperandType::FLOAT => {
                                let float = take_or_get_float!(
                                    return_value_address,
                                    old_memory,
                                    old_call.chunk,
                                    self.cells
                                );

                                set_float!(old_call.return_address, memory, self.cells, float);
                            }
                            OperandType::INTEGER => {
                                let integer = take_or_get_integer!(
                                    return_value_address,
                                    old_memory,
                                    old_call.chunk,
                                    self.cells
                                );

                                set_integer!(old_call.return_address, memory, self.cells, integer);
                            }
                            OperandType::STRING => {
                                let string = take_or_get_string!(
                                    return_value_address,
                                    old_memory,
                                    old_call.chunk,
                                    self.cells
                                );

                                set_string!(old_call.return_address, memory, self.cells, string);
                            }
                            OperandType::LIST_BOOLEAN
                            | OperandType::LIST_BYTE
                            | OperandType::LIST_CHARACTER
                            | OperandType::LIST_FLOAT
                            | OperandType::LIST_INTEGER
                            | OperandType::LIST_STRING
                            | OperandType::LIST_LIST
                            | OperandType::LIST_FUNCTION => {
                                let list = take_or_get_list!(
                                    return_value_address,
                                    old_memory,
                                    old_call.chunk,
                                    self.cells
                                );

                                set_list!(old_call.return_address, memory, self.cells, list);
                            }
                            OperandType::FUNCTION => {
                                let function = take_or_get_function!(
                                    return_value_address,
                                    old_memory,
                                    &old_call.chunk,
                                    self.cells
                                );

                                set_function!(
                                    old_call.return_address,
                                    memory,
                                    self.cells,
                                    function
                                );
                            }
                            invalid => invalid_operand_type_panic!(invalid, operation),
                        }
                    }
                }
                _ => todo!("Handle operation: {operation}"),
            }
        }
    }
}

use std::{
    sync::{Arc, RwLock},
    thread::{Builder, JoinHandle},
};

use tracing::{Level, info, span, warn};

use crate::{
    Address, Chunk, Operation, Value,
    instruction::{
        Add, Call, CallNative, Divide, Equal, Jump, Less, LessEqual, List, Load, MemoryKind,
        Modulo, Multiply, Negate, OperandType, Return, Subtract, Test,
    },
    invalid_operand_type_panic,
    panic_vm::memory::Register,
};

use super::{CallFrame, Cell, Memory};

pub struct Thread<C> {
    pub handle: JoinHandle<Option<Value<C>>>,
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
        let handle = Builder::new()
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

        let mut call_stack = Vec::<CallFrame<C>>::with_capacity(0);
        let mut memory = Memory::<C>::new(&self.chunk);

        memory.create_registers(self.chunk.register_count());

        let mut call = CallFrame::new(&self.chunk, Address::default(), OperandType::NONE, 0);

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
                    } = Load::from(&instruction);

                    let new_register = get_register!(operand, r#type, memory, call);

                    memory.registers[destination.index + call.skipped_registers] = new_register;

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
                }

                // ADD
                Operation::ADD => {
                    let Add {
                        destination,
                        left,
                        right,
                        r#type,
                    } = Add::from(&instruction);

                    let sum_register = match r#type {
                        OperandType::INTEGER => {
                            let left_integer = match left.memory {
                                MemoryKind::REGISTER => memory.registers
                                    [left.index + call.skipped_registers]
                                    .as_integer(),
                                MemoryKind::CONSTANT => call.chunk.constants()[left.index]
                                    .as_integer()
                                    .expect("Expected integer constant"),
                                MemoryKind::CELL => todo!(),
                                _ => unreachable!(),
                            };
                            let right_integer = match right.memory {
                                MemoryKind::REGISTER => memory.registers
                                    [right.index + call.skipped_registers]
                                    .as_integer(),
                                MemoryKind::CONSTANT => call.chunk.constants()[right.index]
                                    .as_integer()
                                    .expect("Expected integer constant"),
                                MemoryKind::CELL => todo!(),
                                _ => unreachable!(),
                            };
                            let integer_sum = left_integer.saturating_add(right_integer);

                            Register::integer(integer_sum)
                        }
                        _ => todo!(),
                    };

                    memory.registers[destination.index + call.skipped_registers] = sum_register;
                }

                // SUBTRACT
                Operation::SUBTRACT => {
                    let Subtract {
                        destination,
                        left,
                        right,
                        r#type,
                    } = Subtract::from(&instruction);
                }

                // MULTIPLY
                Operation::MULTIPLY => {
                    let Multiply {
                        destination,
                        left,
                        right,
                        r#type,
                    } = Multiply::from(&instruction);
                }

                // DIVIDE
                Operation::DIVIDE => {
                    let Divide {
                        destination,
                        left,
                        right,
                        r#type,
                    } = Divide::from(&instruction);
                }

                // MODULO
                Operation::MODULO => {
                    let Modulo {
                        destination,
                        left,
                        right,
                        r#type,
                    } = Modulo::from(&instruction);
                }

                // EQUAL
                Operation::EQUAL => {
                    let Equal {
                        comparator,
                        left,
                        right,
                        r#type,
                    } = Equal::from(&instruction);
                }

                // LESS
                Operation::LESS => {
                    let Less {
                        comparator,
                        left,
                        right,
                        r#type,
                    } = Less::from(&instruction);

                    let left_register = get_register!(left, r#type, memory, call);
                    let right_register = get_register!(right, r#type, memory, call);
                    let is_less = match r#type {
                        OperandType::INTEGER => {
                            left_register.as_integer() < right_register.as_integer()
                        }
                        OperandType::FLOAT => left_register.as_float() < right_register.as_float(),
                        _ => invalid_operand_type_panic!(r#type, "LESS"),
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
                }

                // TEST
                Operation::TEST => {
                    let Test {
                        comparator,
                        operand,
                    } = Test::from(&instruction);
                }

                // NEGATE
                Operation::NEGATE => {
                    let Negate {
                        destination,
                        operand,
                        r#type,
                    } = Negate::from(&instruction);
                }

                // CALL
                Operation::CALL => {
                    let Call {
                        destination,
                        function,
                        argument_count,
                        return_type,
                    } = Call::from(&instruction);
                }

                // CALL_NATIVE
                Operation::CALL_NATIVE => {
                    let CallNative {
                        destination,
                        function,
                        argument_count,
                    } = CallNative::<C>::from(&instruction);
                }

                // JUMP
                Operation::JUMP => {
                    let Jump {
                        offset,
                        is_positive,
                    } = Jump::from(&instruction);

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
                    } = Return::from(&instruction);

                    if call_stack.is_empty() {
                        if should_return_value {
                            let return_value =
                                get_value!(return_value_address, r#type, memory, call);

                            return Some(return_value);
                        } else {
                            return None;
                        }
                    }
                }
                _ => todo!("Handle operation: {operation}"),
            }
        }
    }
}

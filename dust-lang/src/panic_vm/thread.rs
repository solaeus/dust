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
    panic_vm::memory::Register,
    value::List as ListValue,
};

use super::{CallFrame, Cell, CellValue, Memory, macros::*};

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
        let mut memory = Memory::<C>::new(&self.chunk);

        let mut call = CallFrame::new(
            Arc::clone(&self.chunk),
            Address::default(),
            OperandType::NONE,
            memory.get_registers(self.chunk.register_count()),
        );
        let constants = self.chunk.constants();
        let instructions = self.chunk.instructions();

        loop {
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
                        OperandType::INTEGER => {
                            let integer = match operand.memory {
                                MemoryKind::CONSTANT => {
                                    copy_constant!(operand.index, constants, Integer)
                                }
                                _ => todo!(),
                            };

                            match destination.memory {
                                MemoryKind::REGISTER => {
                                    call.registers[destination.index] = Register::integer(integer);
                                }
                                _ => todo!(),
                            }
                        }
                        _ => todo!(),
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
                }

                // CLOSE
                Operation::CLOSE => {
                    let Close { from, to, r#type } = Close::from(&instruction);
                }

                // ADD
                Operation::ADD => {
                    let Add {
                        destination,
                        left,
                        right,
                        r#type,
                    } = Add::from(&instruction);
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
                        argument_list_index,
                        return_type,
                    } = Call::from(&instruction);
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
                            todo!()
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

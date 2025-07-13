use std::{
    mem::replace,
    sync::{Arc, RwLock},
    thread::{Builder as ThreadBuilder, JoinHandle},
};

use smallvec::{SmallVec, smallvec};
use tracing::{Level, error, info, span, trace, warn};

use crate::{
    Address, Chunk, DustString, Instruction, Operation, StrippedChunk, Value,
    instruction::{
        Add, Call, CallNative, Divide, Equal, Jump, Less, LessEqual, List, Load, MemoryKind,
        Modulo, Multiply, Negate, OperandType, Return, Subtract, Test,
    },
    value::List as ListValue,
    vm::Register,
};

use super::{CallFrame, Cell, Object, RuntimeError};

pub struct Thread {
    pub handle: JoinHandle<Result<Option<Value<StrippedChunk>>, RuntimeError>>,
}

impl Thread {
    pub fn new(
        chunk: StrippedChunk,
        cells: Arc<RwLock<Vec<Cell>>>,
        threads: Arc<RwLock<Vec<Thread>>>,
    ) -> Self {
        let name = chunk
            .name()
            .as_ref()
            .map(|name| name.to_string())
            .unwrap_or_else(|| "anonymous".to_string());
        let handle = ThreadBuilder::new()
            .name(name)
            .spawn(|| {
                let runner = ThreadRunner {
                    decoded_instructions: SmallVec::with_capacity(chunk.instructions.len()),
                    register_stack: smallvec![Register::default(); chunk.register_count],
                    register_start: 0,
                    object_pool: SmallVec::new(),
                    call_stack: SmallVec::new(),
                    active_call: CallFrame::new(
                        Arc::new(chunk),
                        Address::default(),
                        OperandType::NONE,
                    ),
                    threads,
                    cells,
                    return_value: None,
                };

                runner.run()
            })
            .expect("Failed to spawn thread");

        Thread { handle }
    }
}

#[derive(Clone)]
struct ThreadRunner {
    decoded_instructions: SmallVec<[DecodedInstruction; 32]>,

    call_stack: SmallVec<[CallFrame; 8]>,
    active_call: CallFrame,

    register_stack: SmallVec<[Register; 32]>,
    register_start: usize,

    object_pool: SmallVec<[Object; 8]>,

    threads: Arc<RwLock<Vec<Thread>>>,
    cells: Arc<RwLock<Vec<Cell>>>,

    return_value: Option<Option<Value<StrippedChunk>>>,
}

impl ThreadRunner {
    fn run(mut self) -> Result<Option<Value>, RuntimeError> {
        let span = span!(Level::INFO, "Thread");
        let _enter = span.enter();

        info!(
            "Starting thread {}",
            self.active_call
                .chunk
                .name()
                .as_ref()
                .map(|name| name.as_ref())
                .unwrap_or_default()
        );

        for instruction in &self.active_call.chunk.instructions {
            let instruction_runner = DecodedInstruction::new(*instruction)?;

            self.decoded_instructions.push(instruction_runner);
        }

        self.next()?;

        Ok(self.return_value.take().flatten())
    }

    fn next(&mut self) -> Result<(), RuntimeError> {
        info!("VM thread IP = {}", self.active_call.ip);

        let decoded_instruction = self.decoded_instructions[self.active_call.ip];
        self.active_call.ip += 1;

        (decoded_instruction.logic)(
            self,
            decoded_instruction.destination_index,
            decoded_instruction.left_index,
            decoded_instruction.right_index,
        )
    }
}

type InstructionLogic = fn(
    runner: &mut ThreadRunner,
    destination_index: usize,
    left_index: usize,
    right_index: usize,
) -> Result<(), RuntimeError>;

#[derive(Copy, Clone)]
struct DecodedInstruction {
    logic: InstructionLogic,
    destination_index: usize,
    left_index: usize,
    right_index: usize,
}

impl Default for DecodedInstruction {
    fn default() -> Self {
        DecodedInstruction {
            logic: no_op,
            destination_index: 0,
            left_index: 0,
            right_index: 0,
        }
    }
}

impl DecodedInstruction {
    fn new(instruction: Instruction) -> Result<Self, RuntimeError> {
        match instruction.operation() {
            Operation::NO_OP => Ok(DecodedInstruction {
                logic: no_op,
                destination_index: 0,
                left_index: 0,
                right_index: 0,
            }),
            Operation::LOAD => {
                let Load {
                    destination,
                    operand,
                    jump_next,
                    r#type,
                } = Load::from(instruction);

                let logic = match operand.memory {
                    MemoryKind::REGISTER => load_from_register,
                    MemoryKind::CONSTANT => match r#type {
                        OperandType::INTEGER => load_constant_integer,
                        _ => return Err(RuntimeError::InvalidOperandType),
                    },
                    _ => return Err(RuntimeError::InvalidMemoryKind),
                };

                Ok(DecodedInstruction {
                    logic,
                    destination_index: destination.index,
                    left_index: operand.index,
                    right_index: jump_next,
                })
            }
            Operation::ADD => {
                let Add {
                    destination,
                    left,
                    right,
                    r#type,
                } = Add::from(instruction);

                let logic = match r#type {
                    OperandType::INTEGER => match (left.memory, right.memory) {
                        (MemoryKind::REGISTER, MemoryKind::REGISTER) => {
                            register_integer_plus_register_integer
                        }
                        (MemoryKind::REGISTER, MemoryKind::CONSTANT) => {
                            register_integer_plus_constant_integer
                        }
                        _ => return Err(RuntimeError::InvalidMemoryKind),
                    },
                    _ => return Err(RuntimeError::InvalidOperandType),
                };

                Ok(DecodedInstruction {
                    logic,
                    destination_index: destination.index,
                    left_index: left.index,
                    right_index: right.index,
                })
            }
            Operation::LESS => {
                let Less {
                    comparator,
                    left,
                    right,
                    r#type,
                } = Less::from(instruction);

                let logic = match r#type {
                    OperandType::INTEGER => match (left.memory, right.memory) {
                        (MemoryKind::REGISTER, MemoryKind::REGISTER) => {
                            register_integer_less_than_register_integer
                        }
                        (MemoryKind::REGISTER, MemoryKind::CONSTANT) => {
                            register_integer_less_than_constant_integer
                        }
                        _ => return Err(RuntimeError::InvalidMemoryKind),
                    },
                    _ => return Err(RuntimeError::InvalidOperandType),
                };

                Ok(DecodedInstruction {
                    logic,
                    destination_index: comparator,
                    left_index: left.index,
                    right_index: right.index,
                })
            }
            Operation::JUMP => {
                let Jump {
                    offset,
                    is_positive,
                } = Jump::from(instruction);

                Ok(DecodedInstruction {
                    logic: jump,
                    destination_index: 0,
                    left_index: offset,
                    right_index: is_positive,
                })
            }
            Operation::RETURN => {
                let Return {
                    should_return_value,
                    return_value_address,
                    r#type,
                } = Return::from(instruction);

                if should_return_value != 0 {
                    match r#type {
                        OperandType::INTEGER => match return_value_address.memory {
                            MemoryKind::REGISTER => Ok(DecodedInstruction {
                                logic: return_register_integer,
                                destination_index: 0,
                                left_index: return_value_address.index,
                                right_index: 0,
                            }),
                            _ => Err(RuntimeError::InvalidMemoryKind),
                        },
                        _ => Err(RuntimeError::InvalidOperandType),
                    }
                } else {
                    Ok(DecodedInstruction {
                        logic: return_nothing,
                        destination_index: 0,
                        left_index: 0,
                        right_index: 0,
                    })
                }
            }
            _ => Err(RuntimeError::InvalidOperation),
        }
    }
}

fn no_op(thread: &mut ThreadRunner, _: usize, _: usize, _: usize) -> Result<(), RuntimeError> {
    error!("Running NO_OP instruction");

    thread.next()
}

fn load_from_register(
    thread: &mut ThreadRunner,
    destination_index: usize,
    operand_index: usize,
    jump_next: usize,
) -> Result<(), RuntimeError> {
    trace!(
        "{}",
        Instruction::load(
            Address::register(destination_index),
            Address::register(operand_index),
            OperandType::NONE, // The type is not used when stringifying the instruction
            jump_next,
        )
    );

    let new_register = read_register!(operand_index, thread.register_stack);
    let old_register = read_register_mut!(destination_index, thread.register_stack);

    *old_register = new_register;

    if jump_next != 0 {
        thread.active_call.ip += 1;
    }

    thread.next()
}

fn load_constant_integer(
    thread: &mut ThreadRunner,
    destination_index: usize,
    operand_index: usize,
    jump_next: usize,
) -> Result<(), RuntimeError> {
    trace!(
        "{}",
        Instruction::load(
            Address::register(destination_index),
            Address::constant(operand_index),
            OperandType::INTEGER,
            jump_next,
        )
    );

    let integer = thread.active_call.chunk.constants[operand_index]
        .as_integer()
        .ok_or(RuntimeError::InvalidConstantType)?;
    let new_register = Register::integer(integer);
    let old_register = read_register_mut!(destination_index, thread.register_stack);

    *old_register = new_register;

    if jump_next != 0 {
        thread.active_call.ip += 1;
    }

    thread.next()
}

fn register_integer_plus_register_integer(
    thread: &mut ThreadRunner,
    destination_index: usize,
    left_index: usize,
    right_index: usize,
) -> Result<(), RuntimeError> {
    trace!(
        "{}",
        Instruction::add(
            Address::register(destination_index),
            Address::register(left_index),
            Address::register(right_index),
            OperandType::INTEGER,
        )
    );

    let left_integer = read_register!(left_index, thread.register_stack).as_integer();
    let right_integer = read_register!(right_index, thread.register_stack).as_integer();
    let sum = left_integer + right_integer;

    let new_register = Register::integer(sum);
    let old_register = read_register_mut!(destination_index, thread.register_stack);

    *old_register = new_register;

    thread.next()
}

fn register_integer_plus_constant_integer(
    thread: &mut ThreadRunner,
    destination_index: usize,
    left_index: usize,
    right_index: usize,
) -> Result<(), RuntimeError> {
    trace!(
        "{}",
        Instruction::add(
            Address::register(destination_index),
            Address::register(left_index),
            Address::constant(right_index),
            OperandType::INTEGER,
        )
    );

    let left_integer = read_register!(left_index, thread.register_stack).as_integer();
    let right_integer = thread.active_call.chunk.constants[right_index]
        .as_integer()
        .ok_or(RuntimeError::InvalidConstantType)?;
    let sum = left_integer + right_integer;

    let new_register = Register::integer(sum);
    let old_register = read_register_mut!(destination_index, thread.register_stack);

    *old_register = new_register;

    thread.next()
}

fn register_integer_less_than_register_integer(
    thread: &mut ThreadRunner,
    comparator: usize,
    left_index: usize,
    right_index: usize,
) -> Result<(), RuntimeError> {
    trace!(
        "{}",
        Instruction::less(
            comparator,
            Address::register(left_index),
            Address::register(right_index),
            OperandType::INTEGER,
        )
    );

    let left_integer = read_register!(left_index, thread.register_stack).as_integer();
    let right_integer = read_register!(right_index, thread.register_stack).as_integer();
    let is_less = left_integer < right_integer;

    if is_less == (comparator != 0) {
        thread.active_call.ip += 1;
    }

    thread.next()
}

fn register_integer_less_than_constant_integer(
    thread: &mut ThreadRunner,
    comparator: usize,
    left_index: usize,
    right_index: usize,
) -> Result<(), RuntimeError> {
    trace!(
        "{}",
        Instruction::less(
            comparator,
            Address::register(left_index),
            Address::constant(right_index),
            OperandType::INTEGER,
        )
    );

    let left_integer = read_register!(left_index, thread.register_stack).as_integer();
    let right_integer = read_constant!(right_index, thread.active_call.chunk.constants)
        .as_integer()
        .ok_or(RuntimeError::InvalidConstantType)?;
    let is_less = left_integer < right_integer;

    if is_less == (comparator != 0) {
        thread.active_call.ip += 1;
    }

    thread.next()
}

fn jump(
    thread: &mut ThreadRunner,
    _: usize,
    offset: usize,
    is_positive: usize,
) -> Result<(), RuntimeError> {
    trace!("{}", Instruction::jump(offset, is_positive));

    if is_positive != 0 {
        thread.active_call.ip += offset;
    } else {
        thread.active_call.ip -= offset + 1;
    }

    thread.next()
}

fn return_nothing(
    thread: &mut ThreadRunner,
    _: usize,
    _: usize,
    _: usize,
) -> Result<(), RuntimeError> {
    trace!(
        "{}",
        Instruction::r#return(false as usize, Address::default(), OperandType::NONE)
    );

    if thread.call_stack.is_empty() {
        thread.return_value = Some(None);
    }

    Ok(())
}

fn return_register_integer(
    thread: &mut ThreadRunner,
    _: usize,
    return_value_index: usize,
    _: usize,
) -> Result<(), RuntimeError> {
    trace!(
        "{}",
        Instruction::r#return(
            false as usize,
            Address::register(return_value_index),
            OperandType::INTEGER
        )
    );

    let return_value_integer =
        read_register!(return_value_index, thread.register_stack).as_integer();

    if thread.call_stack.is_empty() {
        thread.return_value = Some(Some(Value::Integer(return_value_integer)));
    } else {
        thread.active_call = thread
            .call_stack
            .pop()
            .ok_or(RuntimeError::CallStackUnderflow)?;
    }

    Ok(())
}

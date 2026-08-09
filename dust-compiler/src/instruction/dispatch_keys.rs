use crate::instruction::{InstructionBuilder, MemoryKind, OperandType, Operation};

pub const MOVE_BOOLEAN_ENCODED: u64 = InstructionBuilder::new(Operation::MOVE)
    .operand_type(OperandType::BOOLEAN)
    .b_memory(MemoryKind::ENCODED)
    .build()
    .dispatch_key();

pub const MOVE_BOOLEAN_REGISTER: u64 = InstructionBuilder::new(Operation::MOVE)
    .operand_type(OperandType::BOOLEAN)
    .b_memory(MemoryKind::REGISTER)
    .build()
    .dispatch_key();

pub const MOVE_I32_REGISTER: u64 = InstructionBuilder::new(Operation::MOVE)
    .operand_type(OperandType::I_32)
    .b_memory(MemoryKind::REGISTER)
    .build()
    .dispatch_key();

pub const MOVE_I32_ENCODED: u64 = InstructionBuilder::new(Operation::MOVE)
    .operand_type(OperandType::I_32)
    .b_memory(MemoryKind::ENCODED)
    .build()
    .dispatch_key();

pub const GET_BOOLEAN_REGISTER: u64 = InstructionBuilder::new(Operation::GET)
    .operand_type(OperandType::BOOLEAN)
    .c_memory(MemoryKind::REGISTER)
    .build()
    .dispatch_key();

pub const SET_BOOLEAN_REGISTER_ENCODED: u64 = InstructionBuilder::new(Operation::SET)
    .operand_type(OperandType::BOOLEAN)
    .b_memory(MemoryKind::REGISTER)
    .c_memory(MemoryKind::ENCODED)
    .build()
    .dispatch_key();

pub const LESS_I32_REGISTER_ENCODED: u64 = InstructionBuilder::new(Operation::LESS)
    .operand_type(OperandType::I_32)
    .b_memory(MemoryKind::REGISTER)
    .c_memory(MemoryKind::ENCODED)
    .build()
    .dispatch_key();

pub const TEST_REGISTER: u64 = InstructionBuilder::new(Operation::TEST)
    .b_memory(MemoryKind::REGISTER)
    .build()
    .dispatch_key();

pub const ADD_I32_REGISER_REGISTER: u64 = InstructionBuilder::new(Operation::ADD)
    .operand_type(OperandType::I_32)
    .b_memory(MemoryKind::REGISTER)
    .c_memory(MemoryKind::REGISTER)
    .build()
    .dispatch_key();

pub const ADD_I32_REGISTER_ENCODED: u64 = InstructionBuilder::new(Operation::ADD)
    .operand_type(OperandType::I_32)
    .b_memory(MemoryKind::REGISTER)
    .c_memory(MemoryKind::ENCODED)
    .build()
    .dispatch_key();

pub const MULTIPLY_I32_REGISTER_REGISTER: u64 = InstructionBuilder::new(Operation::MULTIPLY)
    .operand_type(OperandType::I_32)
    .b_memory(MemoryKind::REGISTER)
    .c_memory(MemoryKind::REGISTER)
    .build()
    .dispatch_key();

pub const NEGATE_BOOLEAN_REGISTER: u64 = InstructionBuilder::new(Operation::NEGATE)
    .operand_type(OperandType::BOOLEAN)
    .b_memory(MemoryKind::REGISTER)
    .build()
    .dispatch_key();

pub const JUMP_POSTIVE: u64 = InstructionBuilder::new(Operation::JUMP)
    .b_memory(MemoryKind(true as u8))
    .build()
    .dispatch_key();

pub const JUMP_NEGATIVE: u64 = InstructionBuilder::new(Operation::JUMP)
    .b_memory(MemoryKind(false as u8))
    .build()
    .dispatch_key();

pub const RETURN: u64 = InstructionBuilder::new(Operation::RETURN)
    .build()
    .dispatch_key();

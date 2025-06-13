use std::fmt::{self, Display, Formatter};

use crate::r#type::TypeKind;

use super::{Address, Instruction, InstructionFields, OperandType, Operation};

pub struct Call {
    pub destination: Address,
    pub function: Address,
    pub argument_list_index: u16,
    pub return_type: OperandType,
}

impl From<&Instruction> for Call {
    fn from(instruction: &Instruction) -> Self {
        let destination = instruction.destination();
        let function_register = instruction.b_address();
        let argument_list_index = instruction.c_field();
        let return_type = instruction.operand_type();

        Call {
            destination,
            function: function_register,
            argument_list_index,
            return_type,
        }
    }
}

impl From<Call> for Instruction {
    fn from(call: Call) -> Self {
        let operation = Operation::CALL;
        let Address {
            index: a_field,
            memory: a_memory_kind,
        } = call.destination;
        let Address {
            index: b_field,
            memory: b_memory_kind,
        } = call.function;
        let c_field = call.argument_list_index;
        let operand_type = call.return_type;

        InstructionFields {
            operation,
            a_field,
            a_memory_kind,
            b_field,
            b_memory_kind,
            c_field,
            operand_type,
            ..Default::default()
        }
        .build()
    }
}

impl Display for Call {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Call {
            destination,
            function,
            argument_list_index,
            return_type,
        } = self;
        let (function_type, destination_type) = match *return_type {
            OperandType::FUNCTION_RETURNS_NONE => (TypeKind::Function, TypeKind::None),
            OperandType::SELF_RETURNS_NONE => (TypeKind::Function, TypeKind::None),
            OperandType::FUNCTION_RETURNS_BOOLEAN => (TypeKind::Function, TypeKind::Boolean),
            OperandType::SELF_RETURNS_BOOLEAN => (TypeKind::Function, TypeKind::Boolean),
            OperandType::FUNCTION_RETURNS_BYTE => (TypeKind::Function, TypeKind::Byte),
            OperandType::SELF_RETURNS_BYTE => (TypeKind::Function, TypeKind::Byte),
            OperandType::FUNCTION_RETURNS_CHARACTER => (TypeKind::Function, TypeKind::Character),
            OperandType::SELF_RETURNS_CHARACTER => (TypeKind::Function, TypeKind::Character),
            OperandType::FUNCTION_RETURNS_FLOAT => (TypeKind::Function, TypeKind::Float),
            OperandType::SELF_RETURNS_FLOAT => (TypeKind::Function, TypeKind::Float),
            OperandType::FUNCTION_RETURNS_INTEGER => (TypeKind::Function, TypeKind::Integer),
            OperandType::SELF_RETURNS_INTEGER => (TypeKind::Function, TypeKind::Integer),
            OperandType::FUNCTION_RETURNS_STRING => (TypeKind::Function, TypeKind::String),
            OperandType::SELF_RETURNS_STRING => (TypeKind::Function, TypeKind::String),
            OperandType::FUNCTION_RETURNS_LIST => (TypeKind::Function, TypeKind::List),
            OperandType::SELF_RETURNS_LIST => (TypeKind::Function, TypeKind::List),
            OperandType::FUNCTION_RETURNS_FUNCTION => (TypeKind::Function, TypeKind::Function),
            OperandType::SELF_RETURNS_FUNCTION => (TypeKind::FunctionSelf, TypeKind::Function),
            OperandType::FUNCTION_RETURNS_SELF => (TypeKind::Function, TypeKind::FunctionSelf),
            OperandType::SELF_RETURNS_SELF => (TypeKind::FunctionSelf, TypeKind::FunctionSelf),
            _ => return write!(f, "INVALID_CALL_INSTRUCTION"),
        };

        if destination_type != TypeKind::None {
            destination.display(f, destination_type)?;
            write!(f, " = ")?;
        }

        function.display(f, function_type)?;
        write!(f, "(ARGS_{argument_list_index})")
    }
}

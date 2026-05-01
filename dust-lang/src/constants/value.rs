use std::fmt::{self, Display, Formatter};

use crate::{
    compiler::{
        error::CompileError,
        resolver::{PrototypeId, types::TypeId},
    },
    instruction::OperandType,
    syntax::{
        components::{ComparisonExpression, LogicExpression, MathExpression},
        node::SyntaxKind,
        reader::SyntaxReader,
    },
};

#[derive(Clone, Copy, Debug)]
pub enum ConstantValue {
    Boolean(bool),
    Character(char),
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    U128(u128),
    I128(i128),
    F32(f32),
    F64(f64),
    Function {
        prototype_id: PrototypeId,
        type_id: TypeId,
    },
}

impl ConstantValue {
    pub fn type_id(self) -> TypeId {
        match self {
            ConstantValue::Boolean(_) => TypeId::BOOLEAN,
            ConstantValue::Character(_) => TypeId::CHARACTER,
            ConstantValue::U8(_) => TypeId::U_8,
            ConstantValue::I8(_) => TypeId::I_8,
            ConstantValue::U16(_) => TypeId::U_16,
            ConstantValue::I16(_) => TypeId::I_16,
            ConstantValue::U32(_) => TypeId::U_32,
            ConstantValue::I32(_) => TypeId::I_32,
            ConstantValue::U64(_) => TypeId::U_64,
            ConstantValue::I64(_) => TypeId::I_64,
            ConstantValue::U128(_) => TypeId::U_128,
            ConstantValue::I128(_) => TypeId::I_128,
            ConstantValue::F32(_) => TypeId::F_32,
            ConstantValue::F64(_) => TypeId::F_64,
            ConstantValue::Function { type_id, .. } => type_id,
        }
    }

    pub fn operand_type(self) -> OperandType {
        match self {
            ConstantValue::Boolean(_) => OperandType::BOOLEAN,
            ConstantValue::Character(_) => OperandType::CHARACTER,
            ConstantValue::U8(_) => OperandType::U_8,
            ConstantValue::I8(_) => OperandType::I_8,
            ConstantValue::U16(_) => OperandType::U_16,
            ConstantValue::I16(_) => OperandType::I_16,
            ConstantValue::U32(_) => OperandType::U_32,
            ConstantValue::I32(_) => OperandType::I_32,
            ConstantValue::U64(_) => OperandType::U_64,
            ConstantValue::I64(_) => OperandType::I_64,
            ConstantValue::U128(_) => OperandType::U_128,
            ConstantValue::I128(_) => OperandType::I_128,
            ConstantValue::F32(_) => OperandType::F_32,
            ConstantValue::F64(_) => OperandType::F_64,
            ConstantValue::Function { .. } => OperandType::FUNCTION,
        }
    }

    pub fn encoded_u16(self) -> Option<u16> {
        fn encode<I: TryInto<u16>>(integer: I) -> Option<u16> {
            integer.try_into().ok()
        }

        match self {
            ConstantValue::Boolean(boolean) => Some(boolean as u16),
            ConstantValue::I8(integer) => Some(integer as u16),
            ConstantValue::I16(integer) => Some(integer as u16),
            ConstantValue::I32(integer) => encode(integer),
            ConstantValue::I64(integer) => encode(integer),
            ConstantValue::I128(integer) => encode(integer),
            ConstantValue::U8(integer) => Some(integer as u16),
            ConstantValue::U16(integer) => Some(integer),
            ConstantValue::U32(integer) => encode(integer),
            ConstantValue::U64(integer) => encode(integer),
            ConstantValue::U128(integer) => encode(integer),
            ConstantValue::Character(_) | ConstantValue::F32(_) | ConstantValue::F64(_) => None,
            ConstantValue::Function { prototype_id, .. } => Some(prototype_id.inner()),
        }
    }

    pub fn add(self, other: Self, syntax: &SyntaxReader) -> Result<Self, CompileError> {
        match (self, other) {
            (ConstantValue::U8(left), ConstantValue::U8(right)) => left
                .checked_add(right)
                .map(ConstantValue::U8)
                .ok_or_else(|| self.create_overflow_error(other, syntax)),
            (ConstantValue::U16(left), ConstantValue::U16(right)) => left
                .checked_add(right)
                .map(ConstantValue::U16)
                .ok_or_else(|| self.create_overflow_error(other, syntax)),
            (ConstantValue::I16(left), ConstantValue::I16(right)) => left
                .checked_add(right)
                .map(ConstantValue::I16)
                .ok_or_else(|| self.create_overflow_error(other, syntax)),
            (ConstantValue::U32(left), ConstantValue::U32(right)) => left
                .checked_add(right)
                .map(ConstantValue::U32)
                .ok_or_else(|| self.create_overflow_error(other, syntax)),
            (ConstantValue::I32(left), ConstantValue::I32(right)) => left
                .checked_add(right)
                .map(ConstantValue::I32)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::U64(left), ConstantValue::U64(right)) => left
                .checked_add(right)
                .map(ConstantValue::U64)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::I64(left), ConstantValue::I64(right)) => left
                .checked_add(right)
                .map(ConstantValue::I64)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::U128(left), ConstantValue::U128(right)) => left
                .checked_add(right)
                .map(ConstantValue::U128)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::I128(left), ConstantValue::I128(right)) => left
                .checked_add(right)
                .map(ConstantValue::I128)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::F32(left), ConstantValue::F32(right)) => {
                Ok(ConstantValue::F32(left + right))
            }
            (ConstantValue::F64(left), ConstantValue::F64(right)) => {
                Ok(ConstantValue::F64(left + right))
            }
            _ => Err(self.create_type_conflict_error(other, syntax)),
        }
    }

    pub fn subtract(self, other: Self, syntax: &SyntaxReader) -> Result<Self, CompileError> {
        match (self, other) {
            (ConstantValue::U8(left), ConstantValue::U8(right)) => left
                .checked_sub(right)
                .map(ConstantValue::U8)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::I8(left), ConstantValue::I8(right)) => left
                .checked_sub(right)
                .map(ConstantValue::I8)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::U16(left), ConstantValue::U16(right)) => left
                .checked_sub(right)
                .map(ConstantValue::U16)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::I16(left), ConstantValue::I16(right)) => left
                .checked_sub(right)
                .map(ConstantValue::I16)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::U32(left), ConstantValue::U32(right)) => left
                .checked_sub(right)
                .map(ConstantValue::U32)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::I32(left), ConstantValue::I32(right)) => left
                .checked_sub(right)
                .map(ConstantValue::I32)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::U64(left), ConstantValue::U64(right)) => left
                .checked_sub(right)
                .map(ConstantValue::U64)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::I64(left), ConstantValue::I64(right)) => left
                .checked_sub(right)
                .map(ConstantValue::I64)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::U128(left), ConstantValue::U128(right)) => left
                .checked_sub(right)
                .map(ConstantValue::U128)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::I128(left), ConstantValue::I128(right)) => left
                .checked_sub(right)
                .map(ConstantValue::I128)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::F32(left), ConstantValue::F32(right)) => {
                Ok(ConstantValue::F32(left - right))
            }
            (ConstantValue::F64(left), ConstantValue::F64(right)) => {
                Ok(ConstantValue::F64(left - right))
            }
            _ => Err(self.create_type_conflict_error(other, syntax)),
        }
    }

    pub fn multiply(self, other: Self, syntax: &SyntaxReader) -> Result<Self, CompileError> {
        match (self, other) {
            (ConstantValue::U8(left), ConstantValue::U8(right)) => left
                .checked_mul(right)
                .map(ConstantValue::U8)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::I8(left), ConstantValue::I8(right)) => left
                .checked_mul(right)
                .map(ConstantValue::I8)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::U16(left), ConstantValue::U16(right)) => left
                .checked_mul(right)
                .map(ConstantValue::U16)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::I16(left), ConstantValue::I16(right)) => left
                .checked_mul(right)
                .map(ConstantValue::I16)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::U32(left), ConstantValue::U32(right)) => left
                .checked_mul(right)
                .map(ConstantValue::U32)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::I32(left), ConstantValue::I32(right)) => left
                .checked_mul(right)
                .map(ConstantValue::I32)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::U64(left), ConstantValue::U64(right)) => left
                .checked_mul(right)
                .map(ConstantValue::U64)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::I64(left), ConstantValue::I64(right)) => left
                .checked_mul(right)
                .map(ConstantValue::I64)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::U128(left), ConstantValue::U128(right)) => left
                .checked_mul(right)
                .map(ConstantValue::U128)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::I128(left), ConstantValue::I128(right)) => left
                .checked_mul(right)
                .map(ConstantValue::I128)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::F32(left), ConstantValue::F32(right)) => {
                Ok(ConstantValue::F32(left * right))
            }
            (ConstantValue::F64(left), ConstantValue::F64(right)) => {
                Ok(ConstantValue::F64(left * right))
            }
            _ => Err(self.create_type_conflict_error(other, syntax)),
        }
    }

    pub fn divide(self, other: Self, syntax: &SyntaxReader) -> Result<Self, CompileError> {
        match (self, other) {
            (ConstantValue::U8(left), ConstantValue::U8(right)) => left
                .checked_div(right)
                .map(ConstantValue::U8)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::I8(left), ConstantValue::I8(right)) => left
                .checked_div(right)
                .map(ConstantValue::I8)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::U16(left), ConstantValue::U16(right)) => left
                .checked_div(right)
                .map(ConstantValue::U16)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::I16(left), ConstantValue::I16(right)) => left
                .checked_div(right)
                .map(ConstantValue::I16)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::U32(left), ConstantValue::U32(right)) => left
                .checked_div(right)
                .map(ConstantValue::U32)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::I32(left), ConstantValue::I32(right)) => left
                .checked_div(right)
                .map(ConstantValue::I32)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::U64(left), ConstantValue::U64(right)) => left
                .checked_div(right)
                .map(ConstantValue::U64)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::I64(left), ConstantValue::I64(right)) => left
                .checked_div(right)
                .map(ConstantValue::I64)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::U128(left), ConstantValue::U128(right)) => left
                .checked_div(right)
                .map(ConstantValue::U128)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::I128(left), ConstantValue::I128(right)) => left
                .checked_div(right)
                .map(ConstantValue::I128)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::F32(left), ConstantValue::F32(right)) => {
                Ok(ConstantValue::F32(left / right))
            }
            (ConstantValue::F64(left), ConstantValue::F64(right)) => {
                Ok(ConstantValue::F64(left / right))
            }
            _ => Err(self.create_type_conflict_error(other, syntax)),
        }
    }

    pub fn modulo(self, other: Self, syntax: &SyntaxReader) -> Result<Self, CompileError> {
        match (self, other) {
            (ConstantValue::U8(left), ConstantValue::U8(right)) => left
                .checked_rem(right)
                .map(ConstantValue::U8)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::I8(left), ConstantValue::I8(right)) => left
                .checked_rem(right)
                .map(ConstantValue::I8)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::U16(left), ConstantValue::U16(right)) => left
                .checked_rem(right)
                .map(ConstantValue::U16)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::I16(left), ConstantValue::I16(right)) => left
                .checked_rem(right)
                .map(ConstantValue::I16)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::U32(left), ConstantValue::U32(right)) => left
                .checked_rem(right)
                .map(ConstantValue::U32)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::I32(left), ConstantValue::I32(right)) => left
                .checked_rem(right)
                .map(ConstantValue::I32)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::U64(left), ConstantValue::U64(right)) => left
                .checked_rem(right)
                .map(ConstantValue::U64)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::I64(left), ConstantValue::I64(right)) => left
                .checked_rem(right)
                .map(ConstantValue::I64)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::U128(left), ConstantValue::U128(right)) => left
                .checked_rem(right)
                .map(ConstantValue::U128)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::I128(left), ConstantValue::I128(right)) => left
                .checked_rem(right)
                .map(ConstantValue::I128)
                .ok_or_else(|| self.create_type_conflict_error(other, syntax)),
            (ConstantValue::F32(left), ConstantValue::F32(right)) => {
                Ok(ConstantValue::F32(left % right))
            }
            (ConstantValue::F64(left), ConstantValue::F64(right)) => {
                Ok(ConstantValue::F64(left % right))
            }
            _ => Err(self.create_type_conflict_error(other, syntax)),
        }
    }

    pub fn exponentiate(self, other: Self, syntax: &SyntaxReader) -> Result<Self, CompileError> {
        macro_rules! create_exponent_error {
            ($syntax: expr) => {{
                match $syntax.as_component() {
                    Ok(MathExpression { left, right }) => CompileError::InvalidConstantExponent {
                        base_value: self,
                        base_span: left.node.span,
                        exponent_value: other,
                        exponent_span: right.node.span,
                        operator: $syntax.node.kind,
                        source_id: $syntax.source_id(),
                    },
                    Err(error) => CompileError::Syntax(error),
                }
            }};
        }

        match (self, other) {
            (ConstantValue::U8(left), ConstantValue::U8(right)) => left
                .checked_pow(right as u32)
                .map(ConstantValue::U8)
                .ok_or_else(|| self.create_overflow_error(other, syntax)),
            (ConstantValue::U8(left), ConstantValue::U32(right)) => left
                .checked_pow(right)
                .map(ConstantValue::U8)
                .ok_or_else(|| self.create_overflow_error(other, syntax)),
            (ConstantValue::I8(left), ConstantValue::I8(right)) => left
                .checked_pow(right as u32)
                .map(ConstantValue::I8)
                .ok_or_else(|| self.create_overflow_error(other, syntax)),
            (ConstantValue::I8(left), ConstantValue::U32(right)) => left
                .checked_pow(right)
                .map(ConstantValue::I8)
                .ok_or_else(|| self.create_overflow_error(other, syntax)),
            (ConstantValue::U16(left), ConstantValue::U16(right)) => left
                .checked_pow(right as u32)
                .map(ConstantValue::U16)
                .ok_or_else(|| self.create_overflow_error(other, syntax)),
            (ConstantValue::U16(left), ConstantValue::U32(right)) => left
                .checked_pow(right)
                .map(ConstantValue::U16)
                .ok_or_else(|| self.create_overflow_error(other, syntax)),
            (ConstantValue::I16(left), ConstantValue::I16(right)) => left
                .checked_pow(right as u32)
                .map(ConstantValue::I16)
                .ok_or_else(|| self.create_overflow_error(other, syntax)),
            (ConstantValue::I16(left), ConstantValue::U32(right)) => left
                .checked_pow(right)
                .map(ConstantValue::I16)
                .ok_or_else(|| self.create_overflow_error(other, syntax)),
            (ConstantValue::U32(left), ConstantValue::U32(right)) => left
                .checked_pow(right)
                .map(ConstantValue::U32)
                .ok_or_else(|| self.create_overflow_error(other, syntax)),
            (ConstantValue::I32(left), ConstantValue::I32(right)) => left
                .checked_pow(right as u32)
                .map(ConstantValue::I32)
                .ok_or_else(|| self.create_overflow_error(other, syntax)),
            (ConstantValue::I32(left), ConstantValue::U32(right)) => left
                .checked_pow(right)
                .map(ConstantValue::I32)
                .ok_or_else(|| self.create_overflow_error(other, syntax)),
            (ConstantValue::U64(left), ConstantValue::U64(right)) => {
                let right: u32 = right
                    .try_into()
                    .map_err(|_| create_exponent_error!(syntax))?;

                left.checked_pow(right)
                    .map(ConstantValue::U64)
                    .ok_or_else(|| self.create_overflow_error(other, syntax))
            }
            (ConstantValue::I64(left), ConstantValue::U32(right)) => left
                .checked_pow(right)
                .map(ConstantValue::I64)
                .ok_or_else(|| self.create_overflow_error(other, syntax)),
            (ConstantValue::I64(left), ConstantValue::I64(right)) => {
                let right: u32 = right
                    .try_into()
                    .map_err(|_| create_exponent_error!(syntax))?;

                left.checked_pow(right as u32)
                    .map(ConstantValue::I64)
                    .ok_or_else(|| self.create_overflow_error(other, syntax))
            }
            (ConstantValue::U64(left), ConstantValue::U32(right)) => left
                .checked_pow(right)
                .map(ConstantValue::U64)
                .ok_or_else(|| self.create_overflow_error(other, syntax)),
            (ConstantValue::U128(left), ConstantValue::U128(right)) => {
                let right: u32 = right
                    .try_into()
                    .map_err(|_| create_exponent_error!(syntax))?;

                left.checked_pow(right as u32)
                    .map(ConstantValue::U128)
                    .ok_or_else(|| self.create_overflow_error(other, syntax))
            }
            (ConstantValue::I128(left), ConstantValue::U32(right)) => left
                .checked_pow(right)
                .map(ConstantValue::I128)
                .ok_or_else(|| self.create_overflow_error(other, syntax)),
            (ConstantValue::I128(left), ConstantValue::I128(right)) => {
                let right: u32 = right
                    .try_into()
                    .map_err(|_| create_exponent_error!(syntax))?;

                left.checked_pow(right as u32)
                    .map(ConstantValue::I128)
                    .ok_or_else(|| self.create_overflow_error(other, syntax))
            }
            (ConstantValue::U128(left), ConstantValue::U32(right)) => left
                .checked_pow(right)
                .map(ConstantValue::U128)
                .ok_or_else(|| self.create_overflow_error(other, syntax)),
            (ConstantValue::F32(left), ConstantValue::F32(right)) => {
                Ok(ConstantValue::F32(left.powf(right)))
            }
            (ConstantValue::F64(left), ConstantValue::F64(right)) => {
                Ok(ConstantValue::F64(left.powf(right)))
            }
            _ => Err(self.create_type_conflict_error(other, syntax)),
        }
    }

    pub fn equal(self, other: Self, syntax: &SyntaxReader) -> Result<Self, CompileError> {
        self.equal_inner(other, syntax).map(ConstantValue::Boolean)
    }

    fn equal_inner(self, other: Self, syntax: &SyntaxReader) -> Result<bool, CompileError> {
        match (self, other) {
            (ConstantValue::Boolean(left), ConstantValue::Boolean(right)) => Ok(left == right),
            (ConstantValue::Character(left), ConstantValue::Character(right)) => Ok(left == right),
            (ConstantValue::U8(left), ConstantValue::U8(right)) => Ok(left == right),
            (ConstantValue::I8(left), ConstantValue::I8(right)) => Ok(left == right),
            (ConstantValue::U16(left), ConstantValue::U16(right)) => Ok(left == right),
            (ConstantValue::I16(left), ConstantValue::I16(right)) => Ok(left == right),
            (ConstantValue::U32(left), ConstantValue::U32(right)) => Ok(left == right),
            (ConstantValue::I32(left), ConstantValue::I32(right)) => Ok(left == right),
            (ConstantValue::U64(left), ConstantValue::U64(right)) => Ok(left == right),
            (ConstantValue::I64(left), ConstantValue::I64(right)) => Ok(left == right),
            (ConstantValue::U128(left), ConstantValue::U128(right)) => Ok(left == right),
            (ConstantValue::I128(left), ConstantValue::I128(right)) => Ok(left == right),
            (ConstantValue::F32(left), ConstantValue::F32(right)) => Ok(left == right),
            (ConstantValue::F64(left), ConstantValue::F64(right)) => Ok(left == right),
            _ => Err(self.create_type_conflict_error(other, syntax)),
        }
    }

    pub fn not_equal(self, other: Self, syntax: &SyntaxReader) -> Result<Self, CompileError> {
        self.equal_inner(other, syntax)
            .map(|equal| ConstantValue::Boolean(!equal))
    }

    pub fn less_than(self, other: Self, syntax: &SyntaxReader) -> Result<Self, CompileError> {
        self.less_than_inner(other, syntax)
            .map(ConstantValue::Boolean)
    }

    fn less_than_inner(self, other: Self, syntax: &SyntaxReader) -> Result<bool, CompileError> {
        match (self, other) {
            (ConstantValue::Character(left), ConstantValue::Character(right)) => Ok(left < right),
            (ConstantValue::U8(left), ConstantValue::U8(right)) => Ok(left < right),
            (ConstantValue::I8(left), ConstantValue::I8(right)) => Ok(left < right),
            (ConstantValue::U16(left), ConstantValue::U16(right)) => Ok(left < right),
            (ConstantValue::I16(left), ConstantValue::I16(right)) => Ok(left < right),
            (ConstantValue::U32(left), ConstantValue::U32(right)) => Ok(left < right),
            (ConstantValue::I32(left), ConstantValue::I32(right)) => Ok(left < right),
            (ConstantValue::U64(left), ConstantValue::U64(right)) => Ok(left < right),
            (ConstantValue::I64(left), ConstantValue::I64(right)) => Ok(left < right),
            (ConstantValue::U128(left), ConstantValue::U128(right)) => Ok(left < right),
            (ConstantValue::I128(left), ConstantValue::I128(right)) => Ok(left < right),
            (ConstantValue::F32(left), ConstantValue::F32(right)) => Ok(left < right),
            (ConstantValue::F64(left), ConstantValue::F64(right)) => Ok(left < right),
            _ => Err(self.create_type_conflict_error(other, syntax)),
        }
    }

    pub fn greater_than(self, other: Self, syntax: &SyntaxReader) -> Result<Self, CompileError> {
        self.less_than_or_equal_inner(other, syntax)
            .map(|less_than_or_equal| ConstantValue::Boolean(!less_than_or_equal))
    }

    pub fn less_than_or_equal(
        self,
        other: Self,
        syntax: &SyntaxReader,
    ) -> Result<Self, CompileError> {
        self.less_than_or_equal_inner(other, syntax)
            .map(ConstantValue::Boolean)
    }

    fn less_than_or_equal_inner(
        self,
        other: Self,
        syntax: &SyntaxReader,
    ) -> Result<bool, CompileError> {
        match (self, other) {
            (ConstantValue::Character(left), ConstantValue::Character(right)) => Ok(left <= right),
            (ConstantValue::U8(left), ConstantValue::U8(right)) => Ok(left <= right),
            (ConstantValue::I8(left), ConstantValue::I8(right)) => Ok(left <= right),
            (ConstantValue::U16(left), ConstantValue::U16(right)) => Ok(left <= right),
            (ConstantValue::I16(left), ConstantValue::I16(right)) => Ok(left <= right),
            (ConstantValue::U32(left), ConstantValue::U32(right)) => Ok(left <= right),
            (ConstantValue::I32(left), ConstantValue::I32(right)) => Ok(left <= right),
            (ConstantValue::U64(left), ConstantValue::U64(right)) => Ok(left <= right),
            (ConstantValue::I64(left), ConstantValue::I64(right)) => Ok(left <= right),
            (ConstantValue::U128(left), ConstantValue::U128(right)) => Ok(left <= right),
            (ConstantValue::I128(left), ConstantValue::I128(right)) => Ok(left <= right),
            (ConstantValue::F32(left), ConstantValue::F32(right)) => Ok(left <= right),
            (ConstantValue::F64(left), ConstantValue::F64(right)) => Ok(left <= right),
            _ => Err(self.create_type_conflict_error(other, syntax)),
        }
    }

    pub fn greater_than_or_equal(
        self,
        other: Self,
        syntax: &SyntaxReader,
    ) -> Result<Self, CompileError> {
        self.less_than_inner(other, syntax)
            .map(|less_than| ConstantValue::Boolean(!less_than))
    }

    pub fn and(self, other: Self, syntax: &SyntaxReader) -> Result<Self, CompileError> {
        match (self, other) {
            (ConstantValue::Boolean(left), ConstantValue::Boolean(right)) => {
                Ok(ConstantValue::Boolean(left && right))
            }
            _ => Err(self.create_type_conflict_error(other, syntax)),
        }
    }

    pub fn or(self, other: Self, syntax: &SyntaxReader) -> Result<Self, CompileError> {
        match (self, other) {
            (ConstantValue::Boolean(left), ConstantValue::Boolean(right)) => {
                Ok(ConstantValue::Boolean(left || right))
            }
            _ => Err(self.create_type_conflict_error(other, syntax)),
        }
    }

    pub fn negate(self, syntax: &SyntaxReader) -> Result<Self, CompileError> {
        let overflow_error = || CompileError::ConstantUnaryOverflow {
            value: self,
            operand_span: syntax.node.span,
            operator: syntax.node.kind,
            source_id: syntax.source_id(),
        };

        match self {
            ConstantValue::Boolean(boolean) => Ok(ConstantValue::Boolean(!boolean)),
            ConstantValue::I8(integer) => integer
                .checked_neg()
                .map(ConstantValue::I8)
                .ok_or_else(overflow_error),
            ConstantValue::I16(integer) => integer
                .checked_neg()
                .map(ConstantValue::I16)
                .ok_or_else(overflow_error),
            ConstantValue::I32(integer) => integer
                .checked_neg()
                .map(ConstantValue::I32)
                .ok_or_else(overflow_error),
            ConstantValue::I64(integer) => integer
                .checked_neg()
                .map(ConstantValue::I64)
                .ok_or_else(overflow_error),
            ConstantValue::I128(integer) => integer
                .checked_neg()
                .map(ConstantValue::I128)
                .ok_or_else(overflow_error),
            ConstantValue::F32(float) => Ok(ConstantValue::F32(-float)),
            ConstantValue::F64(float) => Ok(ConstantValue::F64(-float)),
            _ => Err(CompileError::CannotApplyOperator {
                operator: syntax.node.kind,
                type_id: self.type_id(),
                operand_position: syntax.position(),
            }),
        }
    }

    fn create_overflow_error(self, other: Self, syntax: &SyntaxReader) -> CompileError {
        match syntax.as_component() {
            Ok(MathExpression { left, right }) => CompileError::ConstantOverflow {
                left_value: self,
                left_span: left.node.span,
                right_value: other,
                right_span: right.node.span,
                operator: syntax.node.kind,
                source_id: syntax.source_id(),
            },
            Err(error) => CompileError::Syntax(error),
        }
    }

    fn create_type_conflict_error(self, other: Self, syntax: &SyntaxReader) -> CompileError {
        let (left, right) = match syntax.node.kind {
            SyntaxKind::AdditionExpression
            | SyntaxKind::SubtractionExpression
            | SyntaxKind::MultiplicationExpression
            | SyntaxKind::DivisionExpression
            | SyntaxKind::ModuloExpression
            | SyntaxKind::ExponentExpression => {
                let MathExpression { left, right } = match syntax.as_component() {
                    Ok(math_expression) => math_expression,
                    Err(error) => return CompileError::Syntax(error),
                };

                (left, right)
            }
            SyntaxKind::EqualExpression
            | SyntaxKind::NotEqualExpression
            | SyntaxKind::LessThanExpression
            | SyntaxKind::GreaterThanExpression
            | SyntaxKind::LessThanOrEqualExpression
            | SyntaxKind::GreaterThanOrEqualExpression => {
                let ComparisonExpression { left, right } = match syntax.as_component() {
                    Ok(comparison_expression) => comparison_expression,
                    Err(error) => return CompileError::Syntax(error),
                };

                (left, right)
            }
            SyntaxKind::AndExpression | SyntaxKind::OrExpression => {
                let LogicExpression { left, right } = match syntax.as_component() {
                    Ok(logical_expression) => logical_expression,
                    Err(error) => return CompileError::Syntax(error),
                };

                (left, right)
            }
            _ => {
                return CompileError::UnexpectedSyntax {
                    expected: &[
                        SyntaxKind::AdditionExpression,
                        SyntaxKind::SubtractionExpression,
                        SyntaxKind::MultiplicationExpression,
                        SyntaxKind::DivisionExpression,
                        SyntaxKind::ModuloExpression,
                        SyntaxKind::ExponentExpression,
                        SyntaxKind::EqualExpression,
                        SyntaxKind::NotEqualExpression,
                        SyntaxKind::LessThanExpression,
                        SyntaxKind::GreaterThanExpression,
                        SyntaxKind::LessThanOrEqualExpression,
                        SyntaxKind::GreaterThanOrEqualExpression,
                        SyntaxKind::AndExpression,
                        SyntaxKind::OrExpression,
                    ],
                    found: syntax.node.kind,
                };
            }
        };

        CompileError::TypeConflict {
            expected_type: self.type_id(),
            expected_position: Some(left.position()),
            found_type: other.type_id(),
            found_position: right.position(),
        }
    }
}

impl Display for ConstantValue {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ConstantValue::Boolean(boolean) => write!(f, "{boolean}"),
            ConstantValue::Character(character) => write!(f, "{character}"),
            ConstantValue::U8(integer) => write!(f, "{integer}"),
            ConstantValue::I8(integer) => write!(f, "{integer}"),
            ConstantValue::U16(integer) => write!(f, "{integer}"),
            ConstantValue::I16(integer) => write!(f, "{integer}"),
            ConstantValue::U32(integer) => write!(f, "{integer}"),
            ConstantValue::I32(integer) => write!(f, "{integer}"),
            ConstantValue::U64(integer) => write!(f, "{integer}"),
            ConstantValue::I64(integer) => write!(f, "{integer}"),
            ConstantValue::U128(integer) => write!(f, "{integer}"),
            ConstantValue::I128(integer) => write!(f, "{integer}"),
            ConstantValue::F32(float) => write!(f, "{float}"),
            ConstantValue::F64(float) => write!(f, "{float}"),
            ConstantValue::Function { prototype_id, .. } => write!(f, "proto_{prototype_id}"),
        }
    }
}

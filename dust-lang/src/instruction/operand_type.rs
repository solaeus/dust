use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct OperandType(pub u8);

impl OperandType {
    // Operand fields are meaningless
    pub const NONE: OperandType = OperandType(0);

    // One or two operands of the same type
    pub const BOOLEAN: OperandType = OperandType(1);
    pub const BYTE: OperandType = OperandType(2);
    pub const CHARACTER: OperandType = OperandType(3);
    pub const FLOAT: OperandType = OperandType(4);
    pub const INTEGER: OperandType = OperandType(5);
    pub const STRING: OperandType = OperandType(6);
    pub const LIST: OperandType = OperandType(7);
    pub const FUNCTION: OperandType = OperandType(8);
    pub const FUNCTION_SELF: OperandType = OperandType(9);

    // Two operands of different types
    pub const CHARACTER_STRING: OperandType = OperandType(10);
    pub const STRING_CHARACTER: OperandType = OperandType(11);

    // Function return types
    pub const FUNCTION_RETURNS_NONE: OperandType = OperandType(12);
    pub const SELF_RETURNS_NONE: OperandType = OperandType(13);
    pub const FUNCTION_RETURNS_BOOLEAN: OperandType = OperandType(14);
    pub const SELF_RETURNS_BOOLEAN: OperandType = OperandType(15);
    pub const FUNCTION_RETURNS_BYTE: OperandType = OperandType(16);
    pub const SELF_RETURNS_BYTE: OperandType = OperandType(17);
    pub const FUNCTION_RETURNS_CHARACTER: OperandType = OperandType(18);
    pub const SELF_RETURNS_CHARACTER: OperandType = OperandType(19);
    pub const FUNCTION_RETURNS_FLOAT: OperandType = OperandType(20);
    pub const SELF_RETURNS_FLOAT: OperandType = OperandType(21);
    pub const FUNCTION_RETURNS_INTEGER: OperandType = OperandType(22);
    pub const SELF_RETURNS_INTEGER: OperandType = OperandType(23);
    pub const FUNCTION_RETURNS_STRING: OperandType = OperandType(24);
    pub const SELF_RETURNS_STRING: OperandType = OperandType(25);
    pub const FUNCTION_RETURNS_LIST: OperandType = OperandType(26);
    pub const SELF_RETURNS_LIST: OperandType = OperandType(27);
    pub const FUNCTION_RETURNS_FUNCTION: OperandType = OperandType(28);
    pub const SELF_RETURNS_FUNCTION: OperandType = OperandType(29);
    pub const FUNCTION_RETURNS_SELF: OperandType = OperandType(30);
    pub const SELF_RETURNS_SELF: OperandType = OperandType(31);
}

pub mod cases {
    pub const BOOLEAN: &str = "true";
    pub const BYTE: &str = "0x2A";
    pub const CHARACTER: &str = "'q'";
    pub const FLOAT: &str = "42.0";
    pub const INTEGER: &str = "42";
    pub const STRING: &str = "\"foobar\"";

    pub const CONSTANT_BYTE_ADDITION: &str = "0x28 + 0x02";
    pub const CONSTANT_FLOAT_ADDITION: &str = "40.0 + 2.0";
    pub const CONSTANT_INTEGER_ADDITION: &str = "40 + 2";

    pub const CONSTANT_BYTE_SUBTRACTION: &str = "0x2C - 0x02";
    pub const CONSTANT_FLOAT_SUBTRACTION: &str = "44.0 - 2.0";
    pub const CONSTANT_INTEGER_SUBTRACTION: &str = "44 - 2";

    pub const CONSTANT_BYTE_MULTIPLICATION: &str = "0x0E * 0x03";
    pub const CONSTANT_FLOAT_MULTIPLICATION: &str = "14.0 * 3.0";
    pub const CONSTANT_INTEGER_MULTIPLICATION: &str = "14 * 3";

    pub const CONSTANT_BYTE_DIVISION: &str = "0x54 / 0x02";
    pub const CONSTANT_FLOAT_DIVISION: &str = "84.0 / 2.0";
    pub const CONSTANT_INTEGER_DIVISION: &str = "84 / 2";

    pub const CONSTANT_STRING_CONCATENATION: &str = "\"foo\" + \"bar\"";
    pub const CONSTANT_CHARACTER_CONCATENATION: &str = "'q' + 'q'";
    pub const CONSTANT_STRING_CHARACTER_CONCATENATION: &str = "\"foo\" + 'q'";
    pub const CONSTANT_CHARACTER_STRING_CONCATENATION: &str = "'q' + \"foo\"";

    pub const LOCAL_DECLARATION: &str = "let x: int = 42;";
    pub const LOCAL_MUT_DECLARATION: &str = "let mut x: int = 42;";
    pub const LOCAL_EVALUATION: &str = "let x: int = 42; x";
}

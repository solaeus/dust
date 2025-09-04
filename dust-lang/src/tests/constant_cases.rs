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

pub const CONSTANT_BOOLEAN_AND: &str = "true && false";
pub const CONSTANT_BOOLEAN_OR: &str = "true || false";
pub const CONSTANT_BOOLEAN_NOT: &str = "!true";

pub const CONSTANT_BOOLEAN_GREATER_THAN: &str = "true > false";
pub const CONSTANT_BOOLEAN_LESS_THAN: &str = "false < true";
pub const CONSTANT_BOOLEAN_GREATER_THAN_OR_EQUAL: &str = "true >= true";
pub const CONSTANT_BOOLEAN_LESS_THAN_OR_EQUAL: &str = "true <= true";
pub const CONSTANT_BOOLEAN_EQUAL: &str = "true == true";
pub const CONSTANT_BOOLEAN_NOT_EQUAL: &str = "true != false";

pub const CONSTANT_BYTE_GREATER_THAN: &str = "0x2B > 0x2A";
pub const CONSTANT_BYTE_LESS_THAN: &str = "0x29 < 0x2A";
pub const CONSTANT_BYTE_GREATER_THAN_OR_EQUAL: &str = "0x2A >= 0x2A";
pub const CONSTANT_BYTE_LESS_THAN_OR_EQUAL: &str = "0x2A <= 0x2A";
pub const CONSTANT_BYTE_EQUAL: &str = "0x2A == 0x2A";
pub const CONSTANT_BYTE_NOT_EQUAL: &str = "0x2A != 0x2B";

pub const CONSTANT_CHARACTER_GREATER_THAN: &str = "'{' > 'z'";
pub const CONSTANT_CHARACTER_LESS_THAN: &str = "'y' < 'z'";
pub const CONSTANT_CHARACTER_GREATER_THAN_OR_EQUAL: &str = "'z' >= 'z'";
pub const CONSTANT_CHARACTER_LESS_THAN_OR_EQUAL: &str = "'z' <= 'z'";
pub const CONSTANT_CHARACTER_EQUAL: &str = "'z' == 'z'";
pub const CONSTANT_CHARACTER_NOT_EQUAL: &str = "'z' != '{'";

pub const CONSTANT_FLOAT_GREATER_THAN: &str = "43.0 > 42.0";
pub const CONSTANT_FLOAT_LESS_THAN: &str = "41.0 < 42.0";
pub const CONSTANT_FLOAT_GREATER_THAN_OR_EQUAL: &str = "42.0 >= 42.0";
pub const CONSTANT_FLOAT_LESS_THAN_OR_EQUAL: &str = "42.0 <= 42.0";
pub const CONSTANT_FLOAT_EQUAL: &str = "42.0 == 42.0";
pub const CONSTANT_FLOAT_NOT_EQUAL: &str = "42.0 != 43.0";

pub const CONSTANT_INTEGER_GREATER_THAN: &str = "43 > 42";
pub const CONSTANT_INTEGER_LESS_THAN: &str = "41 < 42";
pub const CONSTANT_INTEGER_GREATER_THAN_OR_EQUAL: &str = "42 >= 42";
pub const CONSTANT_INTEGER_LESS_THAN_OR_EQUAL: &str = "42 <= 42";
pub const CONSTANT_INTEGER_EQUAL: &str = "42 == 42";
pub const CONSTANT_INTEGER_NOT_EQUAL: &str = "42 != 43";

pub const CONSTANT_STRING_GREATER_THAN: &str = "\"bar\" > \"foo\"";
pub const CONSTANT_STRING_LESS_THAN: &str = "\"foo\" < \"bar\"";
pub const CONSTANT_STRING_GREATER_THAN_OR_EQUAL: &str = "\"foo\" >= \"foo\"";
pub const CONSTANT_STRING_LESS_THAN_OR_EQUAL: &str = "\"foo\" <= \"foo\"";
pub const CONSTANT_STRING_EQUAL: &str = "\"foo\" == \"foo\"";
pub const CONSTANT_STRING_NOT_EQUAL: &str = "\"foo\" != \"bar\"";

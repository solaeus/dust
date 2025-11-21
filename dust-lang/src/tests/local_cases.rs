pub const LOCAL_BOOLEAN: &str = r#"
let x: bool = true;
x
"#;
pub const LOCAL_BYTE: &str = r#"
let x: byte = 0x2A;
x
"#;
pub const LOCAL_CHARACTER: &str = r#"
let x: char = 'q';
x
"#;
pub const LOCAL_FLOAT: &str = r#"
let x: float = 42.0;
x
"#;
pub const LOCAL_INTEGER: &str = r#"
let x: int = 42;
x
"#;
pub const LOCAL_STRING: &str = r#"
let x: str = "foobar";
x
"#;

pub const LOCAL_BYTE_ADDITION: &str = r#"
let a: byte = 0x28;
let b: byte = 0x02;
a + b
"#;
pub const LOCAL_FLOAT_ADDITION: &str = r#"
let a: float = 40.0;
let b: float = 2.0;
a + b
"#;
pub const LOCAL_INTEGER_ADDITION: &str = r#"
let a: int = 40;
let b: int = 2;
a + b
"#;

pub const LOCAL_MUT_BYTE_ADDITION: &str = r#"
let mut a: byte = 0x28;
a += 0x02;
a
"#;
pub const LOCAL_MUT_FLOAT_ADDITION: &str = r#"
let mut a: float = 40.0;
a += 2.0;
a
"#;
pub const LOCAL_MUT_INTEGER_ADDITION: &str = r#"
let mut a: int = 40;
a += 2;
a
"#;

pub const LOCAL_BYTE_SUBTRACTION: &str = r#"
let a: byte = 0x2C;
let b: byte = 0x02;
a - b
"#;
pub const LOCAL_FLOAT_SUBTRACTION: &str = r#"
let a: float = 44.0;
let b: float = 2.0;
a - b
"#;
pub const LOCAL_INTEGER_SUBTRACTION: &str = r#"
let a: int = 44;
let b: int = 2;
a - b
"#;

pub const LOCAL_MUT_BYTE_SUBTRACTION: &str = r#"
let mut a: byte = 0x2C;
a -= 0x02;
a
"#;
pub const LOCAL_MUT_FLOAT_SUBTRACTION: &str = r#"
let mut a: float = 44.0;
a -= 2.0;
a
"#;
pub const LOCAL_MUT_INTEGER_SUBTRACTION: &str = r#"
let mut a: int = 44;
a -= 2;
a
"#;

pub const LOCAL_BYTE_MULTIPLICATION: &str = r#"
let a: byte = 0x0E;
let b: byte = 0x03;
a * b
"#;
pub const LOCAL_FLOAT_MULTIPLICATION: &str = r#"
let a: float = 14.0;
let b: float = 3.0;
a * b
"#;
pub const LOCAL_INTEGER_MULTIPLICATION: &str = r#"
let a: int = 14;
let b: int = 3;
a * b
"#;

pub const LOCAL_MUT_BYTE_MULTIPLICATION: &str = r#"
let mut a: byte = 0x0E;
a *= 0x03;
a
"#;
pub const LOCAL_MUT_FLOAT_MULTIPLICATION: &str = r#"
let mut a: float = 14.0;
a *= 3.0;
a
"#;
pub const LOCAL_MUT_INTEGER_MULTIPLICATION: &str = r#"
let mut a: int = 14;
a *= 3;
a
"#;

pub const LOCAL_BYTE_DIVISION: &str = r#"
let a: byte = 0x54;
let b: byte = 0x02;
a / b
"#;
pub const LOCAL_FLOAT_DIVISION: &str = r#"
let a: float = 84.0;
let b: float = 2.0;
a / b
"#;
pub const LOCAL_INTEGER_DIVISION: &str = r#"
let a: int = 84;
let b: int = 2;
a / b
"#;

pub const LOCAL_MUT_BYTE_DIVISION: &str = r#"
let mut a: byte = 0x54;
a /= 0x02;
a
"#;
pub const LOCAL_MUT_FLOAT_DIVISION: &str = r#"
let mut a: float = 84.0;
a /= 2.0;
a
"#;
pub const LOCAL_MUT_INTEGER_DIVISION: &str = r#"
let mut a: int = 84;
a /= 2;
a
"#;

pub const LOCAL_BYTE_MODULO: &str = r#"
let a: byte = 0x54;
let b: byte = 0x05;
a % b
"#;
pub const LOCAL_FLOAT_MODULO: &str = r#"
let a: float = 84.0;
let b: float = 5.0;
a % b
"#;
pub const LOCAL_INTEGER_MODULO: &str = r#"
let a: int = 84;
let b: int = 5;
a % b
"#;

pub const LOCAL_MUT_BYTE_MODULO: &str = r#"
let mut a: byte = 0x54;
a %= 0x05;
a
"#;
pub const LOCAL_MUT_FLOAT_MODULO: &str = r#"
let mut a: float = 84.0;
a %= 5.0;
a
"#;
pub const LOCAL_MUT_INTEGER_MODULO: &str = r#"
let mut a: int = 84;
a %= 5;
a
"#;

pub const LOCAL_BYTE_EXPONENT: &str = r#"
let a: byte = 0x02;
let b: byte = 0x03;
a ^ b
"#;
pub const LOCAL_FLOAT_EXPONENT: &str = r#"
let a: float = 2.0;
let b: float = 3.0;
a ^ b
"#;
pub const LOCAL_INTEGER_EXPONENT: &str = r#"
let a: int = 2;
let b: int = 3;
a ^ b
"#;

pub const LOCAL_MUT_BYTE_EXPONENT: &str = r#"
let mut a: byte = 0x02;
a ^= 0x03;
a
"#;
pub const LOCAL_MUT_FLOAT_EXPONENT: &str = r#"
let mut a: float = 2.0;
a ^= 3.0;
a
"#;
pub const LOCAL_MUT_INTEGER_EXPONENT: &str = r#"
let mut a: int = 2;
a ^= 3;
a
"#;

pub const LOCAL_STRING_CONCATENATION: &str = r#"
let a: str = "foo";
let b: str = "bar";
a + b
"#;
pub const LOCAL_CHARACTER_CONCATENATION: &str = r#"
let a: char = 'q';
let b: char = 'q';
a + b
"#;
pub const LOCAL_STRING_CHARACTER_CONCATENATION: &str = r#"
let a: str = "foo";
let b: char = 'q';
a + b
"#;
pub const LOCAL_CHARACTER_STRING_CONCATENATION: &str = r#"
let a: char = 'q';
let b: str = "foo";
a + b
"#;

pub const LOCAL_MUT_STRING_CONCATENATION: &str = r#"
let mut a: str = "foo";
a += "bar";
a
"#;
pub const LOCAL_MUT_STRING_CHARACTER_CONCATENATION: &str = r#"
let mut a: str = "foo";
a += 'q';
a
"#;

pub const LOCAL_BOOLEAN_AND: &str = r#"
let a: bool = true;
let b: bool = false;
a && b
"#;
pub const LOCAL_BOOLEAN_OR: &str = r#"
let a: bool = true;
let b: bool = false;
a || b
"#;
pub const LOCAL_BOOLEAN_NOT: &str = r#"
let a: bool = true;
!a
"#;

pub const LOCAL_BOOLEAN_GREATER_THAN: &str = r#"
let a: bool = true;
let b: bool = false;
a > b
"#;
pub const LOCAL_BOOLEAN_LESS_THAN: &str = r#"
let a: bool = false;
let b: bool = true;
a < b
"#;
pub const LOCAL_BOOLEAN_GREATER_THAN_OR_EQUAL: &str = r#"
let a: bool = true;
let b: bool = true;
a >= b
"#;
pub const LOCAL_BOOLEAN_LESS_THAN_OR_EQUAL: &str = r#"
let a: bool = true;
let b: bool = true;
a <= b
"#;
pub const LOCAL_BOOLEAN_EQUAL: &str = r#"
let a: bool = true;
let b: bool = true;
a == b
"#;
pub const LOCAL_BOOLEAN_NOT_EQUAL: &str = r#"
let a: bool = true;
let b: bool = false;
a != b
"#;

pub const LOCAL_BYTE_GREATER_THAN: &str = r#"
let a: byte = 0x2B;
let b: byte = 0x2A;
a > b
"#;
pub const LOCAL_BYTE_LESS_THAN: &str = r#"
let a: byte = 0x29;
let b: byte = 0x2A;
a < b
"#;
pub const LOCAL_BYTE_GREATER_THAN_OR_EQUAL: &str = r#"
let a: byte = 0x2A;
let b: byte = 0x2A;
a >= b
"#;
pub const LOCAL_BYTE_LESS_THAN_OR_EQUAL: &str = r#"
let a: byte = 0x2A;
let b: byte = 0x2A;
a <= b
"#;
pub const LOCAL_BYTE_EQUAL: &str = r#"
let a: byte = 0x2A;
let b: byte = 0x2A;
a == b
"#;
pub const LOCAL_BYTE_NOT_EQUAL: &str = r#"
let a: byte = 0x2A;
let b: byte = 0x2B;
a != b
"#;

pub const LOCAL_CHARACTER_GREATER_THAN: &str = r#"
let a: char = '{';
let b: char = 'z';
a > b
"#;
pub const LOCAL_CHARACTER_LESS_THAN: &str = r#"
let a: char = 'y';
let b: char = 'z';
a < b
"#;
pub const LOCAL_CHARACTER_GREATER_THAN_OR_EQUAL: &str = r#"
let a: char = 'z';
let b: char = 'z';
a >= b
"#;
pub const LOCAL_CHARACTER_LESS_THAN_OR_EQUAL: &str = r#"
let a: char = 'z';
let b: char = 'z';
a <= b
"#;
pub const LOCAL_CHARACTER_EQUAL: &str = r#"
let a: char = 'z';
let b: char = 'z';
a == b
"#;
pub const LOCAL_CHARACTER_NOT_EQUAL: &str = r#"
let a: char = 'z';
let b: char = '{';
a != b
"#;

pub const LOCAL_FLOAT_GREATER_THAN: &str = r#"
let a: float = 43.0;
let b: float = 42.0;
a > b
"#;
pub const LOCAL_FLOAT_LESS_THAN: &str = r#"
let a: float = 41.0;
let b: float = 42.0;
a < b
"#;
pub const LOCAL_FLOAT_GREATER_THAN_OR_EQUAL: &str = r#"
let a: float = 42.0;
let b: float = 42.0;
a >= b
"#;
pub const LOCAL_FLOAT_LESS_THAN_OR_EQUAL: &str = r#"
let a: float = 42.0;
let b: float = 42.0;
a <= b
"#;
pub const LOCAL_FLOAT_EQUAL: &str = r#"
let a: float = 42.0;
let b: float = 42.0;
a == b
"#;
pub const LOCAL_FLOAT_NOT_EQUAL: &str = r#"
let a: float = 42.0;
let b: float = 43.0;
a != b
"#;

pub const LOCAL_INTEGER_GREATER_THAN: &str = r#"
let a: int = 43;
let b: int = 42;
a > b
"#;
pub const LOCAL_INTEGER_LESS_THAN: &str = r#"
let a: int = 41;
let b: int = 42;
a < b
"#;
pub const LOCAL_INTEGER_GREATER_THAN_OR_EQUAL: &str = r#"
let a: int = 42;
let b: int = 42;
a >= b
"#;
pub const LOCAL_INTEGER_LESS_THAN_OR_EQUAL: &str = r#"
let a: int = 42;
let b: int = 42;
a <= b
"#;
pub const LOCAL_INTEGER_EQUAL: &str = r#"
let a: int = 42;
let b: int = 42;
a == b
"#;
pub const LOCAL_INTEGER_NOT_EQUAL: &str = r#"
let a: int = 42;
let b: int = 43;
a != b
"#;

pub const LOCAL_STRING_GREATER_THAN: &str = r#"
let a: str = "bar";
let b: str = "foo";
a > b
"#;
pub const LOCAL_STRING_LESS_THAN: &str = r#"
let a: str = "foo";
let b: str = "bar";
a < b
"#;
pub const LOCAL_STRING_GREATER_THAN_OR_EQUAL: &str = r#"
let a: str = "foo";
let b: str = "foo";
a >= b
"#;
pub const LOCAL_STRING_LESS_THAN_OR_EQUAL: &str = r#"
let a: str = "foo";
let b: str = "foo";
a <= b
"#;
pub const LOCAL_STRING_EQUAL: &str = r#"
let a: str = "foo";
let b: str = "foo";
a == b
"#;
pub const LOCAL_STRING_NOT_EQUAL: &str = r#"
let a: str = "foo";
let b: str = "bar";
a != b
"#;

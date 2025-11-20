pub const LIST_BOOLEAN: &str = "[true, false, true]";
pub const LIST_BYTE: &str = "[0x2A, 0x2B, 0x2C]";
pub const LIST_CHARACTER: &str = "['a', 'b', 'c']";
pub const LIST_FLOAT: &str = "[1.0, 2.0, 3.0]";
pub const LIST_INTEGER: &str = "[1, 2, 3]";
pub const LIST_STRING: &str = r#"["foo", "bar", "baz"]"#;

pub const LIST_EQUAL: &str = "[true, false] == [true, false]";
pub const LIST_NOT_EQUAL: &str = "[0x2A, 0x2B] != [0x2B, 0x2A]";
pub const LIST_GREATER_THAN: &str = "['b', 'a'] > ['a', 'b']";
pub const LIST_LESS_THAN: &str = "[1.0, 2.0] < [2.0, 1.0]";
pub const LIST_GREATER_THAN_OR_EQUAL: &str = "[1, 2] >= [1, 2]";
pub const LIST_LESS_THAN_OR_EQUAL: &str = r#"["foo", "bar"] <= ["foo", "bar"]"#;

pub const LIST_INDEX_BOOLEAN: &str = r#"
let x: [bool] = [true, false, true];
x[0]
"#;
pub const LIST_INDEX_BYTE: &str = r#"
let x: [byte] = [0x2A, 0x2B, 0x2C];
x[1]
"#;
pub const LIST_INDEX_CHARACTER: &str = r#"
let x: [char] = ['a', 'b', 'c'];
x[2]
"#;
pub const LIST_INDEX_FLOAT: &str = r#"
let x: [float] = [1.0, 2.0, 3.0];
x[1]
"#;
pub const LIST_INDEX_INTEGER: &str = r#"
let x: [int] = [1, 2, 3];
x[0]
"#;
pub const LIST_INDEX_STRING: &str = r#"
let x: [str] = ["foo", "bar", "baz"];
x[2]
"#;

pub const LOCAL_LIST_BOOLEAN: &str = r#"
let x: [bool] = [true, false, true];
x
"#;

pub const LOCAL_LIST_EQUAL: &str = r#"
let a: [bool] = [true, false];
let b: [bool] = [true, false];
a == b
"#;
pub const LOCAL_LIST_NOT_EQUAL: &str = r#"
let a: [byte] = [0x2A, 0x2B];
let b: [byte] = [0x2B, 0x2A];
a != b
"#;
pub const LOCAL_LIST_GREATER_THAN: &str = r#"
let a: [char] = ['b', 'a'];
let b: [char] = ['a', 'b'];
a > b
"#;
pub const LOCAL_LIST_LESS_THAN: &str = r#"
let a: [float] = [1.0, 2.0];
let b: [float] = [2.0, 1.0];
a < b
"#;
pub const LOCAL_LIST_GREATER_THAN_OR_EQUAL: &str = r#"
let a: [int] = [1, 2];
let b: [int] = [1, 2];
a >= b
"#;
pub const LOCAL_LIST_LESS_THAN_OR_EQUAL: &str = r#"
let a: [str] = ["foo", "bar"];
let b: [str] = ["foo", "bar"];
a <= b
"#;

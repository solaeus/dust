use std::{
    fmt::{self, Display, Formatter},
    io::{self, stdin, stdout, Write},
    string::{self},
};

use serde::{Deserialize, Serialize};

use crate::{AnnotatedError, Instruction, Primitive, Span, Value, Vm, VmError};

// Assertio
const ASSERT: u8 = 0b0000_0000;
const ASSERT_EQ: u8 = 0b0000_0001;
const ASSERT_NE: u8 = 0b0000_0010;
const PANIC: u8 = 0b0000_0011;

// Type conversion
const PARSE: u8 = 0b0000_0100;
const TO_BYTE: u8 = 0b0000_0101;
const TO_FLOAT: u8 = 0b0000_0110;
const TO_INTEGER: u8 = 0b0000_0111;
const TO_STRING: u8 = 0b0000_1000;

// List and string
const ALL: u8 = 0b0000_1001;
const ANY: u8 = 0b0000_1010;
const APPEND: u8 = 0b0000_1011;
const CONTAINS: u8 = 0b0000_1100;
const DEDUP: u8 = 0b0000_1101;
const ENDS_WITH: u8 = 0b0000_1110;
const FIND: u8 = 0b0000_1111;
const GET: u8 = 0b0001_0000;
const INDEX_OF: u8 = 0b0001_0001;
const LENGTH: u8 = 0b0001_0010;
const PREPEND: u8 = 0b0001_0011;
const REPLACE: u8 = 0b0001_0100;
const SET: u8 = 0b0001_0101;
const STARTS_WITH: u8 = 0b0001_0110;
const SLICE: u8 = 0b0001_0111;
const SORT: u8 = 0b0001_1000;
const SPLIT: u8 = 0b0001_1001;

// List
const FLATTEN: u8 = 0b0001_1010;
const JOIN: u8 = 0b0001_1011;
const MAP: u8 = 0b0001_1100;
const REDUCE: u8 = 0b0001_1101;
const REMOVE: u8 = 0b0001_1110;
const REVERSE: u8 = 0b0001_1111;
const UNZIP: u8 = 0b0010_0000;
const ZIP: u8 = 0b0010_0001;

// String
const BYTES: u8 = 0b0010_0010;
const CHAR_AT: u8 = 0b0010_0011;
const CHAR_CODE_AT: u8 = 0b0010_0100;
const CHARS: u8 = 0b0010_0101;
const FORMAT: u8 = 0b0010_0110;
const REPEAT: u8 = 0b0010_0111;
const SPLIT_AT: u8 = 0b0010_1000;
const SPLIT_LINES: u8 = 0b0010_1001;
const SPLIT_WHITESPACE: u8 = 0b0010_1010;
const TO_LOWER_CASE: u8 = 0b0010_1011;
const TO_UPPER_CASE: u8 = 0b0010_1100;
const TRIM: u8 = 0b0010_1101;
const TRIM_END: u8 = 0b0010_1110;
const TRIM_START: u8 = 0b0010_1111;

// I/O
const READ_LINE: u8 = 0b0011_0000;
const WRITE_LINE: u8 = 0b0011_0001;
const WRITE: u8 = 0b0011_0010;

// Random
const RANDOM: u8 = 0b0011_0011;
const RANDOM_BYTE: u8 = 0b0011_0100;
const RANDOM_BYTES: u8 = 0b0011_0101;
const RANDOM_CHAR: u8 = 0b0011_0110;
const RANDOM_FLOAT: u8 = 0b0011_0111;
const RANDOM_INTEGER: u8 = 0b0011_1000;
const RANDOM_RANGE: u8 = 0b0011_1001;
const RANDOM_STRING: u8 = 0b0011_1010;

macro_rules! impl_from_str_for_native_function {
    ($(($name:ident, $str:expr, $returns_value:expr)),*) => {
        #[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
        pub enum NativeFunction {
            $(
                $name,
            )*
        }

        impl NativeFunction {
            pub fn as_str(&self) -> &'static str {
                match self {
                    $(
                        NativeFunction::$name => $str,
                    )*
                }
            }

            #[allow(clippy::should_implement_trait)]
            pub fn from_str(string: &str) -> Option<Self> {
                match string {
                    $(
                        $str => Some(NativeFunction::$name),
                    )*
                    _ => None,
                }
            }

            pub fn returns_value(&self) -> bool {
                match self {
                    $(
                        NativeFunction::$name => $returns_value,
                    )*
                }
            }
        }
    };
}

impl_from_str_for_native_function! {
    // Assertion
    (Assert, "assert", false),
    (AssertEq, "assert_eq", false),
    (AssertNe, "assert_ne", false),
    (Panic, "panic", true),

    // Type conversion
    (Parse, "parse", true),
    (ToByte, "to_byte", true),
    (ToFloat, "to_float", true),
    (ToInteger, "to_integer", true),
    (ToString, "to_string", true),

    // List and string
    (All, "all", true),
    (Any, "any", true),
    (Append, "append", false),
    (Contains, "contains", true),
    (Dedup, "dedup", false),
    (EndsWith, "ends_with", true),
    (Find, "find", true),
    (Get, "get", true),
    (IndexOf, "index_of", true),
    (Length, "length", true),
    (Prepend, "prepend", false),
    (Replace, "replace", false),
    (Set, "set", false),
    (StartsWith, "starts_with", true),
    (Slice, "slice", true),
    (Sort, "sort", false),
    (Split, "split", true),

    // List
    (Flatten, "flatten", true),
    (Join, "join", true),
    (Map, "map", true),
    (Reduce, "reduce", true),
    (Remove, "remove", false),
    (Reverse, "reverse", false),
    (Unzip, "unzip", true),
    (Zip, "zip", true),

    // String
    (Bytes, "bytes", true),
    (CharAt, "char_at", true),
    (CharCodeAt, "char_code_at", true),
    (Chars, "chars", true),
    (Format, "format", true),
    (Repeat, "repeat", true),
    (SplitAt, "split_at", true),
    (SplitLines, "split_lines", true),
    (SplitWhitespace, "split_whitespace", true),
    (ToLowerCase, "to_lower_case", true),
    (ToUpperCase, "to_upper_case", true),
    (Trim, "trim", true),
    (TrimEnd, "trim_end", true),
    (TrimStart, "trim_start", true),

    // I/O
    (ReadLine, "read_line", true),
    (WriteLine, "write_line", false),
    (Write, "write", false),

    // Random
    (Random, "random", true),
    (RandomByte, "random_byte", true),
    (RandomBytes, "random_bytes", true),
    (RandomChar, "random_char", true),
    (RandomFloat, "random_float", true),
    (RandomInteger, "random_integer", true),
    (RandomRange, "random_range", true),
    (RandomString, "random_string", true)
}

impl NativeFunction {
    pub fn call(
        &self,
        instruction: Instruction,
        vm: &Vm,
        position: Span,
    ) -> Result<Option<Value>, VmError> {
        let to_register = instruction.a();
        let argument_count = instruction.c();

        let return_value = match self {
            NativeFunction::Panic => {
                let message = if argument_count == 0 {
                    None
                } else {
                    let mut message = String::new();

                    for argument_index in 0..argument_count {
                        if argument_index != 0 {
                            message.push(' ');
                        }

                        let argument = vm.get(argument_index, position)?;

                        message.push_str(&argument.to_string());
                    }

                    Some(message)
                };

                return Err(VmError::NativeFunction(NativeFunctionError::Panic {
                    message,
                    position,
                }));
            }

            // Type conversion
            NativeFunction::Parse => todo!(),
            NativeFunction::ToByte => todo!(),
            NativeFunction::ToFloat => todo!(),
            NativeFunction::ToInteger => todo!(),
            NativeFunction::ToString => {
                let mut string = String::new();

                for argument_index in 0..argument_count {
                    let argument = vm.get(argument_index, position)?;

                    string.push_str(&argument.to_string());
                }

                Some(Value::Primitive(Primitive::String(string)))
            }

            // I/O
            NativeFunction::ReadLine => {
                let mut buffer = String::new();

                stdin().read_line(&mut buffer).map_err(|io_error| {
                    VmError::NativeFunction(NativeFunctionError::Io {
                        error: io_error.kind(),
                        position,
                    })
                })?;

                buffer = buffer.trim_end_matches('\n').to_string();

                Some(Value::Primitive(Primitive::String(buffer)))
            }
            NativeFunction::Write => {
                let to_register = instruction.a();
                let mut stdout = stdout();
                let map_err = |io_error: io::Error| {
                    VmError::NativeFunction(NativeFunctionError::Io {
                        error: io_error.kind(),
                        position,
                    })
                };

                let first_argument = to_register.saturating_sub(argument_count);
                let last_argument = to_register.saturating_sub(1);

                for argument_index in first_argument..=last_argument {
                    if argument_index != first_argument {
                        stdout.write(b" ").map_err(map_err)?;
                    }

                    let argument_string = vm.get(argument_index, position)?.to_string();

                    stdout
                        .write_all(argument_string.as_bytes())
                        .map_err(map_err)?;
                }

                None
            }
            NativeFunction::WriteLine => {
                let mut stdout = stdout();
                let map_err = |io_error: io::Error| {
                    VmError::NativeFunction(NativeFunctionError::Io {
                        error: io_error.kind(),
                        position,
                    })
                };

                let first_argument = to_register.saturating_sub(argument_count);
                let last_argument = to_register.saturating_sub(1);

                for argument_index in first_argument..=last_argument {
                    if argument_index != 0 {
                        stdout.write(b" ").map_err(map_err)?;
                    }

                    let argument_string = vm.get(argument_index, position)?.to_string();

                    stdout
                        .write_all(argument_string.as_bytes())
                        .map_err(map_err)?;
                }

                stdout.write(b"\n").map_err(map_err)?;

                None
            }
            _ => todo!(),
        };

        Ok(return_value)
    }
}

impl From<u8> for NativeFunction {
    fn from(byte: u8) -> Self {
        match byte {
            ALL => NativeFunction::Assert,
            ASSERT_EQ => NativeFunction::AssertEq,
            ASSERT_NE => NativeFunction::AssertNe,
            APPEND => NativeFunction::Append,
            ANY => NativeFunction::Any,
            BYTES => NativeFunction::Bytes,
            CHAR_AT => NativeFunction::CharAt,
            CHAR_CODE_AT => NativeFunction::CharCodeAt,
            CHARS => NativeFunction::Chars,
            CONTAINS => NativeFunction::Contains,
            DEDUP => NativeFunction::Dedup,
            ENDS_WITH => NativeFunction::EndsWith,
            FIND => NativeFunction::Find,
            FLATTEN => NativeFunction::Flatten,
            FORMAT => NativeFunction::Format,
            GET => NativeFunction::Get,
            INDEX_OF => NativeFunction::IndexOf,
            JOIN => NativeFunction::Join,
            LENGTH => NativeFunction::Length,
            MAP => NativeFunction::Map,
            PANIC => NativeFunction::Panic,
            PARSE => NativeFunction::Parse,
            PREPEND => NativeFunction::Prepend,
            RANDOM => NativeFunction::Random,
            RANDOM_BYTE => NativeFunction::RandomByte,
            RANDOM_BYTES => NativeFunction::RandomBytes,
            RANDOM_CHAR => NativeFunction::RandomChar,
            RANDOM_FLOAT => NativeFunction::RandomFloat,
            RANDOM_INTEGER => NativeFunction::RandomInteger,
            RANDOM_RANGE => NativeFunction::RandomRange,
            RANDOM_STRING => NativeFunction::RandomString,
            READ_LINE => NativeFunction::ReadLine,
            REDUCE => NativeFunction::Reduce,
            REMOVE => NativeFunction::Remove,
            REPEAT => NativeFunction::Repeat,
            REPLACE => NativeFunction::Replace,
            REVERSE => NativeFunction::Reverse,
            SET => NativeFunction::Set,
            SLICE => NativeFunction::Slice,
            SORT => NativeFunction::Sort,
            SPLIT => NativeFunction::Split,
            SPLIT_AT => NativeFunction::SplitAt,
            SPLIT_LINES => NativeFunction::SplitLines,
            SPLIT_WHITESPACE => NativeFunction::SplitWhitespace,
            STARTS_WITH => NativeFunction::StartsWith,
            TO_BYTE => NativeFunction::ToByte,
            TO_FLOAT => NativeFunction::ToFloat,
            TO_INTEGER => NativeFunction::ToInteger,
            TO_LOWER_CASE => NativeFunction::ToLowerCase,
            TO_STRING => NativeFunction::ToString,
            TO_UPPER_CASE => NativeFunction::ToUpperCase,
            TRIM => NativeFunction::Trim,
            TRIM_END => NativeFunction::TrimEnd,
            TRIM_START => NativeFunction::TrimStart,
            UNZIP => NativeFunction::Unzip,
            WRITE => NativeFunction::Write,
            WRITE_LINE => NativeFunction::WriteLine,
            ZIP => NativeFunction::Zip,
            _ => {
                if cfg!(test) {
                    panic!("Invalid native function byte: {}", byte)
                } else {
                    NativeFunction::Panic
                }
            }
        }
    }
}

impl From<NativeFunction> for u8 {
    fn from(native_function: NativeFunction) -> Self {
        match native_function {
            NativeFunction::All => ALL,
            NativeFunction::Any => ANY,
            NativeFunction::Append => APPEND,
            NativeFunction::Assert => ASSERT,
            NativeFunction::AssertEq => ASSERT_EQ,
            NativeFunction::AssertNe => ASSERT_NE,
            NativeFunction::Bytes => BYTES,
            NativeFunction::CharAt => CHAR_AT,
            NativeFunction::CharCodeAt => CHAR_CODE_AT,
            NativeFunction::Chars => CHARS,
            NativeFunction::Contains => CONTAINS,
            NativeFunction::Dedup => DEDUP,
            NativeFunction::EndsWith => ENDS_WITH,
            NativeFunction::Find => FIND,
            NativeFunction::Flatten => FLATTEN,
            NativeFunction::Format => FORMAT,
            NativeFunction::Get => GET,
            NativeFunction::IndexOf => INDEX_OF,
            NativeFunction::Join => JOIN,
            NativeFunction::Length => LENGTH,
            NativeFunction::Map => MAP,
            NativeFunction::Panic => PANIC,
            NativeFunction::Parse => PARSE,
            NativeFunction::Prepend => PREPEND,
            NativeFunction::Random => RANDOM,
            NativeFunction::RandomByte => RANDOM_BYTE,
            NativeFunction::RandomBytes => RANDOM_BYTES,
            NativeFunction::RandomChar => RANDOM_CHAR,
            NativeFunction::RandomFloat => RANDOM_FLOAT,
            NativeFunction::RandomInteger => RANDOM_INTEGER,
            NativeFunction::RandomRange => RANDOM_RANGE,
            NativeFunction::RandomString => RANDOM_STRING,
            NativeFunction::ReadLine => READ_LINE,
            NativeFunction::Reduce => REDUCE,
            NativeFunction::Remove => REMOVE,
            NativeFunction::Repeat => REPEAT,
            NativeFunction::Replace => REPLACE,
            NativeFunction::Reverse => REVERSE,
            NativeFunction::Set => SET,
            NativeFunction::Slice => SLICE,
            NativeFunction::Sort => SORT,
            NativeFunction::Split => SPLIT,
            NativeFunction::SplitAt => SPLIT_AT,
            NativeFunction::SplitLines => SPLIT_LINES,
            NativeFunction::SplitWhitespace => SPLIT_WHITESPACE,
            NativeFunction::StartsWith => STARTS_WITH,
            NativeFunction::ToByte => TO_BYTE,
            NativeFunction::ToFloat => TO_FLOAT,
            NativeFunction::ToInteger => TO_INTEGER,
            NativeFunction::ToLowerCase => TO_LOWER_CASE,
            NativeFunction::ToString => TO_STRING,
            NativeFunction::ToUpperCase => TO_UPPER_CASE,
            NativeFunction::Trim => TRIM,
            NativeFunction::TrimEnd => TRIM_END,
            NativeFunction::TrimStart => TRIM_START,
            NativeFunction::Unzip => UNZIP,
            NativeFunction::Write => WRITE,
            NativeFunction::WriteLine => WRITE_LINE,
            NativeFunction::Zip => ZIP,
        }
    }
}

impl Display for NativeFunction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NativeFunctionError {
    ExpectedArgumentCount {
        expected: usize,
        found: usize,
        position: Span,
    },
    Panic {
        message: Option<String>,
        position: Span,
    },
    Parse {
        error: string::ParseError,
        position: Span,
    },
    Io {
        error: io::ErrorKind,
        position: Span,
    },
}

impl AnnotatedError for NativeFunctionError {
    fn title() -> &'static str {
        "Native Function Error"
    }

    fn description(&self) -> &'static str {
        match self {
            NativeFunctionError::ExpectedArgumentCount { .. } => {
                "Expected a different number of arguments"
            }
            NativeFunctionError::Panic { .. } => "Explicit panic",
            NativeFunctionError::Parse { .. } => "Failed to parse value",
            NativeFunctionError::Io { .. } => "I/O error",
        }
    }

    fn details(&self) -> Option<String> {
        match self {
            NativeFunctionError::ExpectedArgumentCount {
                expected, found, ..
            } => Some(format!("Expected {} arguments, found {}", expected, found)),
            NativeFunctionError::Panic { message, .. } => message.clone(),
            NativeFunctionError::Parse { error, .. } => Some(format!("{}", error)),
            NativeFunctionError::Io { error, .. } => Some(format!("{}", error)),
        }
    }

    fn position(&self) -> Span {
        match self {
            NativeFunctionError::ExpectedArgumentCount { position, .. } => *position,
            NativeFunctionError::Panic { position, .. } => *position,
            NativeFunctionError::Parse { position, .. } => *position,
            NativeFunctionError::Io { position, .. } => *position,
        }
    }
}

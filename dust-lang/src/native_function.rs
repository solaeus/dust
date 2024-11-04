use std::{
    fmt::{self, Display, Formatter},
    io::{self, stdin, stdout, Write},
    string::{self},
};

use serde::{Deserialize, Serialize};

use crate::{AnnotatedError, Instruction, Primitive, Span, Value, Vm, VmError};

macro_rules! impl_from_str_for_native_function {
    ($(($name:ident, $byte:literal, $str:expr, $returns_value:expr)),*) => {
        #[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
        pub enum NativeFunction {
            $(
                $name = $byte as isize,
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

        impl From<u8> for NativeFunction {
            fn from(byte: u8) -> Self {
                match byte {
                    $(
                        $byte => NativeFunction::$name,
                    )*
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
                    $(
                        NativeFunction::$name => $byte,
                    )*
                }
            }
        }
    };
}

impl_from_str_for_native_function! {
    // Assertion
    (Assert, 0_u8, "assert", false),
    (AssertEqual, 1_u8, "assert_equal", false),
    (AssertNotEqual, 2_u8, "assert_not_equal", false),
    (Panic, 3_u8, "panic", true),

    // Type conversion
    (Parse, 4_u8, "parse", true),
    (ToByte, 5_u8, "to_byte", true),
    (ToFloat, 6_u8, "to_float", true),
    (ToInteger, 7_u8, "to_integer", true),
    (ToString, 8_u8, "to_string", true),

    // List and string
    (All, 9_u8, "all", true),
    (Any, 10_u8, "any", true),
    (Append, 11_u8, "append", false),
    (Contains, 12_u8, "contains", true),
    (Dedup, 13_u8, "dedup", false),
    (EndsWith, 14_u8, "ends_with", true),
    (Find, 15_u8, "find", true),
    (Get, 16_u8, "get", true),
    (IndexOf, 17_u8, "index_of", true),
    (Length, 18_u8, "length", true),
    (Prepend, 19_u8, "prepend", false),
    (Replace, 20_u8, "replace", false),
    (Set, 21_u8, "set", false),
    (StartsWith, 22_u8, "starts_with", true),
    (Slice, 23_u8, "slice", true),
    (Sort, 24_u8, "sort", false),
    (Split, 25_u8, "split", true),

    // List
    (Flatten, 26_u8, "flatten", false),
    (Join, 27_u8, "join", true),
    (Map, 28_u8, "map", true),
    (Reduce, 29_u8, "reduce", true),
    (Remove, 30_u8, "remove", false),
    (Reverse, 31_u8, "reverse", false),
    (Unzip, 32_u8, "unzip", true),
    (Zip, 33_u8, "zip", true),

    // String
    (Bytes, 34_u8, "bytes", true),
    (CharAt, 35_u8, "char_at", true),
    (CharCodeAt, 36_u8, "char_code_at", true),
    (Chars, 37_u8, "chars", true),
    (Format, 38_u8, "format", true),
    (Repeat, 39_u8, "repeat", true),
    (SplitAt, 40_u8, "split_at", true),
    (SplitLines, 41_u8, "split_lines", true),
    (SplitWhitespace, 42_u8, "split_whitespace", true),
    (ToLowerCase, 43_u8, "to_lower_case", true),
    (ToUpperCase, 44_u8, "to_upper_case", true),
    (Trim, 45_u8, "trim", true),
    (TrimEnd, 46_u8, "trim_end", true),
    (TrimStart, 47_u8, "trim_start", true),

    // I/O
    (Read, 48_u8, "read", true),
    (ReadFile, 49_u8, "read_file", true),
    (ReadLine, 50_u8, "read_line", true),
    (ReadTo, 51_u8, "read_until", false),
    (ReadUntil, 52_u8, "read_to", false),
    (WriteLine, 53_u8, "write_line", false),
    (Write, 54_u8, "write", false),

    // Random
    (Random, 55_u8, "random", true),
    (RandomByte, 56_u8, "random_byte", true),
    (RandomBytes, 57_u8, "random_bytes", true),
    (RandomChar, 58_u8, "random_char", true),
    (RandomFloat, 59_u8, "random_float", true),
    (RandomInteger, 60_u8, "random_integer", true),
    (RandomRange, 61_u8, "random_range", true),
    (RandomString, 62_u8, "random_string", true)
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

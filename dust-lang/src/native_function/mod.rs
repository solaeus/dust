//! Built-in functions that implement extended functionality.
//!
//! Native functions are used to implement features that are not possible to implement in Dust
//! itself or that are more efficient to implement in Rust.
mod assertion;
mod io;
mod string;

use std::{
    fmt::{self, Display, Formatter},
    io::ErrorKind as IoErrorKind,
    string::ParseError,
};

use serde::{Deserialize, Serialize};
use smallvec::{smallvec, SmallVec};

use crate::{AnnotatedError, FunctionType, Span, Type, Value, ValueRef, Vm, VmError};

macro_rules! define_native_function {
    ($(($name:ident, $bytes:literal, $str:expr, $type:expr, $function:expr)),*) => {
        /// A dust-native function.
        ///
        /// See the [module-level documentation](index.html) for more information.
        #[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
        pub enum NativeFunction {
            $(
                $name = $bytes as isize,
            )*
        }

        impl NativeFunction {
            pub fn call<'a>(
                &self,
                vm: &Vm<'a>,
                arguments: SmallVec<[ValueRef<'a>; 4]>,
            ) -> Result<Option<Value>, NativeFunctionError> {
                match self {
                    $(
                        NativeFunction::$name => $function(vm, arguments),
                    )*
                }
            }

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

            pub fn r#type(&self) -> FunctionType {
                match self {
                    $(
                        NativeFunction::$name => $type,
                    )*
                }
            }

            pub fn returns_value(&self) -> bool {
                match self {
                    $(
                        NativeFunction::$name => $type.return_type != Type::None,
                    )*
                }
            }
        }

        impl From<u8> for NativeFunction {
            fn from(bytes: u8) -> Self {
                match bytes {
                    $(
                        $bytes => NativeFunction::$name,
                    )*
                    _ => {
                        if cfg!(test) {
                            panic!("Invalid native function byte: {}", bytes)
                        } else {
                            NativeFunction::Panic
                        }
                    }
                }
            }
        }

        impl From<NativeFunction> for u16 {
            fn from(native_function: NativeFunction) -> Self {
                match native_function {
                    $(
                        NativeFunction::$name => $bytes,
                    )*
                }
            }
        }
    };
}

impl Display for NativeFunction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

define_native_function! {
    // Assertion
    // (
    //     Assert,
    //     0_u8,
    //     "assert",
    //     FunctionType {
    //         type_parameters: None,
    //         value_parameters: None,
    //         return_type: Box::new(Type::None)
    //     },
    //     assert
    // ),
    // (AssertEqual, 1_u8, "assert_equal", false),
    // (AssertNotEqual, 2_u8, "assert_not_equal", false),
    (
        Panic,
        3,
        "panic",
        FunctionType {
            type_parameters: None,
            value_parameters: None,
            return_type: Type::None
        },
        assertion::panic
    ),

    // // Type conversion
    // (Parse, 4_u8, "parse", true),
    // (ToByte, 5_u8, "to_byte", true),
    // (ToFloat, 6_u8, "to_float", true),
    // (ToInteger, 7_u8, "to_integer", true),
    (
        ToString,
        8,
        "to_string",
        FunctionType {
            type_parameters: None,
            value_parameters: Some(smallvec![(0, Type::Any)]),
            return_type: Type::String
        },
        string::to_string
    ),

    // // List and string
    // (All, 9_u8, "all", true),
    // (Any, 10_u8, "any", true),
    // (Append, 11_u8, "append", false),
    // (Contains, 12_u8, "contains", true),
    // (Dedup, 13_u8, "dedup", false),
    // (EndsWith, 14_u8, "ends_with", true),
    // (Find, 15_u8, "find", true),
    // (Get, 16_u8, "get", true),
    // (IndexOf, 17_u8, "index_of", true),
    // (Length, 18_u8, "length", true),
    // (Prepend, 19_u8, "prepend", false),
    // (Replace, 20_u8, "replace", false),
    // (Set, 21_u8, "set", false),
    // (StartsWith, 22_u8, "starts_with", true),
    // (Slice, 23_u8, "slice", true),
    // (Sort, 24_u8, "sort", false),
    // (Split, 25_u8, "split", true),

    // // List
    // (Flatten, 26_u8, "flatten", false),
    // (Join, 27_u8, "join", true),
    // (Map, 28_u8, "map", true),
    // (Reduce, 29_u8, "reduce", true),
    // (Remove, 30_u8, "remove", false),
    // (Reverse, 31_u8, "reverse", false),
    // (Unzip, 32_u8, "unzip", true),
    // (Zip, 33_u8, "zip", true),

    // // String
    // (Bytes, 34_u8, "bytes", true),
    // (CharAt, 35_u8, "char_at", true),
    // (CharCodeAt, 36_u8, "char_code_at", true),
    // (Chars, 37_u8, "chars", true),
    // (Format, 38_u8, "format", true),
    // (Repeat, 39_u8, "repeat", true),
    // (SplitAt, 40_u8, "split_at", true),
    // (SplitLines, 41_u8, "split_lines", true),
    // (SplitWhitespace, 42_u8, "split_whitespace", true),
    // (ToLowerCase, 43_u8, "to_lower_case", true),
    // (ToUpperCase, 44_u8, "to_upper_case", true),
    // (Trim, 45_u8, "trim", true),
    // (TrimEnd, 46_u8, "trim_end", true),
    // (TrimStart, 47_u8, "trim_start", true),

    // // I/O
    // // Read
    // (Read, 48_u8, "read", true),
    // (ReadFile, 49_u8, "read_file", true),
    (
        ReadLine,
        50,
        "read_line",
        FunctionType {
            type_parameters: None,
            value_parameters: None,
            return_type: Type::String
        },
        io::read_line
    ),
    // (ReadTo, 51_u8, "read_to", false),
    // (ReadUntil, 52_u8, "read_until", true),
    // // Write
    // (AppendFile, 53_u8, "append_file", false),
    // (PrependFile, 54_u8, "prepend_file", false),
    (
        Write,
        55,
        "write",
        FunctionType {
            type_parameters: None,
            value_parameters: Some(smallvec![(0, Type::String)]),
            return_type: Type::None
        },
        io::write
    ),
    // (WriteFile, 56_u8, "write_file", false),
    (
        WriteLine,
        57,
        "write_line",
        FunctionType {
            type_parameters: None,
            value_parameters: Some(smallvec![(0, Type::String)]),
            return_type: Type::None
        },
        io::write_line
    )

    // // Random
    // (Random, 58_u8, "random", true),
    // (RandomInRange, 59_u8, "random_in_range", true)
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
        error: ParseError,
        position: Span,
    },
    Io {
        error: IoErrorKind,
        position: Span,
    },
    Vm(Box<VmError>),
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
            NativeFunctionError::Vm(error) => error.description(),
        }
    }

    fn detail_snippets(&self) -> SmallVec<[(String, Span); 2]> {
        todo!()
    }

    fn help_snippets(&self) -> SmallVec<[(String, Span); 2]> {
        todo!()
    }
}

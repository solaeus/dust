use std::{
    fmt::{self, Display, Formatter},
    io::{self, stdin, stdout, Write},
};

use serde::{Deserialize, Serialize};

use crate::{Instruction, Primitive, Span, Value, Vm, VmError};

const PANIC: u8 = 0b0000_0000;

// Type conversion
const PARSE: u8 = 0b0000_0001;
const TO_BYTE: u8 = 0b0000_0010;
const TO_FLOAT: u8 = 0b0000_0011;
const TO_INTEGER: u8 = 0b0000_0100;
const TO_STRING: u8 = 0b0000_0101;

// List
const ALL: u8 = 0b0000_0110;
const ANY: u8 = 0b0000_0111;
const APPEND: u8 = 0b0000_1000;
const CONTAINS: u8 = 0b0000_1001;
const FIND: u8 = 0b0000_1010;
const FLATTEN: u8 = 0b0000_1011;
const GET: u8 = 0b0000_1100;
const INDEX_OF: u8 = 0b0000_1101;
const JOIN: u8 = 0b0000_1110;
const LENGTH: u8 = 0b0000_1111;
const MAP: u8 = 0b0001_0000;
const PREPEND: u8 = 0b0001_0001;
const REDUCE: u8 = 0b0001_0010;
const REMOVE: u8 = 0b0001_0011;
const REVERSE: u8 = 0b0001_0100;
const SET: u8 = 0b0001_0101;
const SLICE: u8 = 0b0001_0110;
const SORT: u8 = 0b0001_0111;
const SPLIT: u8 = 0b0001_1000;
const UNZIP: u8 = 0b0001_1001;
const ZIP: u8 = 0b0001_1010;

// String
const CHAR_AT: u8 = 0b0001_1011;
const CHAR_CODE_AT: u8 = 0b0001_1100;
const CHARS: u8 = 0b0001_1101;
const ENDS_WITH: u8 = 0b0001_1110;
const FORMAT: u8 = 0b0001_1111;
const INCLUDES: u8 = 0b0010_0000;
const MATCH: u8 = 0b0010_0001;
const PAD_END: u8 = 0b0010_0010;
const PAD_START: u8 = 0b0010_0011;
const REPEAT: u8 = 0b0010_0100;
const REPLACE: u8 = 0b0010_0101;
const SPLIT_AT: u8 = 0b0010_0110;
const SPLIT_LINES: u8 = 0b0010_0111;
const SPLIT_WHITESPACE: u8 = 0b0010_1000;
const STARTS_WITH: u8 = 0b0010_1001;
const TO_LOWER_CASE: u8 = 0b0010_1010;
const TO_UPPER_CASE: u8 = 0b0010_1011;
const TRIM: u8 = 0b0010_1100;
const TRIM_END: u8 = 0b0010_1101;
const TRIM_START: u8 = 0b0010_1110;

// I/O
const READ_LINE: u8 = 0b0010_1111;
const WRITE_LINE: u8 = 0b0011_0001;
const WRITE: u8 = 0b0011_0000;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum NativeFunction {
    Panic = PANIC as isize,

    // Type conversion
    Parse = PARSE as isize,
    ToByte = TO_BYTE as isize,
    ToFloat = TO_FLOAT as isize,
    ToInteger = TO_INTEGER as isize,
    ToString = TO_STRING as isize,

    // List
    All = ALL as isize,
    Any = ANY as isize,
    Append = APPEND as isize,
    Contains = CONTAINS as isize,
    Find = FIND as isize,
    Flatten = FLATTEN as isize,
    Get = GET as isize,
    IndexOf = INDEX_OF as isize,
    Join = JOIN as isize,
    Length = LENGTH as isize,
    Map = MAP as isize,
    Prepend = PREPEND as isize,
    Reduce = REDUCE as isize,
    Remove = REMOVE as isize,
    Reverse = REVERSE as isize,
    Set = SET as isize,
    Slice = SLICE as isize,
    Sort = SORT as isize,
    Split = SPLIT as isize,
    Unzip = UNZIP as isize,
    Zip = ZIP as isize,

    // String
    CharAt = CHAR_AT as isize,
    CharCodeAt = CHAR_CODE_AT as isize,
    Chars = CHARS as isize,
    EndsWith = ENDS_WITH as isize,
    Format = FORMAT as isize,
    Includes = INCLUDES as isize,
    Match = MATCH as isize,
    PadEnd = PAD_END as isize,
    PadStart = PAD_START as isize,
    Repeat = REPEAT as isize,
    Replace = REPLACE as isize,
    SplitAt = SPLIT_AT as isize,
    SplitLines = SPLIT_LINES as isize,
    SplitWhitespace = SPLIT_WHITESPACE as isize,
    StartsWith = STARTS_WITH as isize,
    ToLowerCase = TO_LOWER_CASE as isize,
    ToUpperCase = TO_UPPER_CASE as isize,
    Trim = TRIM as isize,
    TrimEnd = TRIM_END as isize,
    TrimStart = TRIM_START as isize,

    // I/O
    ReadLine = READ_LINE as isize,
    WriteLine = WRITE_LINE as isize,
    Write = WRITE as isize,
}

macro_rules! impl_from_str_for_native_function {
    ($(($name:ident, $str:expr, $returns_value:expr)),*) => {
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

// Use the macro to implement From<&str> for NativeFunction
impl_from_str_for_native_function! {
    (Panic, "panic", false),

    // Type conversion
    (Parse, "parse", true),
    (ToByte, "to_byte", true),
    (ToFloat, "to_float", true),
    (ToInteger, "to_integer", true),
    (ToString, "to_string", true),

    // List
    (All, "all", true),
    (Any, "any", true),
    (Append, "append", true),
    (Contains, "contains", true),
    (Find, "find", true),
    (Flatten, "flatten", true),
    (Get, "get", true),
    (IndexOf, "index_of", true),
    (Join, "join", true),
    (Length, "length", true),
    (Map, "map", true),
    (Prepend, "prepend", true),
    (Reduce, "reduce", true),
    (Remove, "remove", true),
    (Reverse, "reverse", true),
    (Set, "set", true),
    (Slice, "slice", true),
    (Sort, "sort", true),
    (Split, "split", true),
    (Unzip, "unzip", true),
    (Zip, "zip", true),

    // String
    (CharAt, "char_at", true),
    (CharCodeAt, "char_code_at", true),
    (Chars, "chars", true),
    (EndsWith, "ends_with", true),
    (Format, "format", true),
    (Includes, "includes", true),
    (Match, "match", true),
    (PadEnd, "pad_end", true),
    (PadStart, "pad_start", true),
    (Repeat, "repeat", true),
    (Replace, "replace", true),
    (SplitAt, "split_at", true),
    (SplitLines, "split_lines", true),
    (SplitWhitespace, "split_whitespace", true),
    (StartsWith, "starts_with", true),
    (ToLowerCase, "to_lower_case", true),
    (ToUpperCase, "to_upper_case", true),
    (Trim, "trim", true),
    (TrimEnd, "trim_end", true),
    (TrimStart, "trim_start", true),

    // I/O
    (ReadLine, "read_line", true),
    (WriteLine, "write_line", false),
    (Write, "write", false)
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

                return Err(VmError::Panic { message, position });
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

            // List
            NativeFunction::All => todo!(),
            NativeFunction::Any => todo!(),
            NativeFunction::Append => todo!(),
            NativeFunction::Contains => todo!(),
            NativeFunction::Find => todo!(),
            NativeFunction::Flatten => todo!(),
            NativeFunction::Get => todo!(),
            NativeFunction::IndexOf => todo!(),
            NativeFunction::Join => todo!(),
            NativeFunction::Length => todo!(),
            NativeFunction::Map => todo!(),
            NativeFunction::Prepend => todo!(),
            NativeFunction::Reduce => todo!(),
            NativeFunction::Remove => todo!(),
            NativeFunction::Reverse => todo!(),
            NativeFunction::Set => todo!(),
            NativeFunction::Slice => todo!(),
            NativeFunction::Sort => todo!(),
            NativeFunction::Split => todo!(),
            NativeFunction::Unzip => todo!(),
            NativeFunction::Zip => todo!(),

            // String
            NativeFunction::CharAt => todo!(),
            NativeFunction::CharCodeAt => todo!(),
            NativeFunction::Chars => todo!(),
            NativeFunction::EndsWith => todo!(),
            NativeFunction::Format => todo!(),
            NativeFunction::Includes => todo!(),
            NativeFunction::Match => todo!(),
            NativeFunction::PadEnd => todo!(),
            NativeFunction::PadStart => todo!(),
            NativeFunction::Repeat => todo!(),
            NativeFunction::Replace => todo!(),
            NativeFunction::SplitAt => todo!(),
            NativeFunction::SplitLines => todo!(),
            NativeFunction::SplitWhitespace => todo!(),
            NativeFunction::StartsWith => todo!(),
            NativeFunction::ToLowerCase => todo!(),
            NativeFunction::ToUpperCase => todo!(),
            NativeFunction::Trim => todo!(),
            NativeFunction::TrimEnd => todo!(),
            NativeFunction::TrimStart => todo!(),

            // I/O
            NativeFunction::ReadLine => {
                let mut buffer = String::new();

                stdin()
                    .read_line(&mut buffer)
                    .map_err(|io_error| VmError::Io {
                        error: io_error.kind(),
                        position,
                    })?;

                buffer = buffer.trim_end_matches('\n').to_string();

                Some(Value::Primitive(Primitive::String(buffer)))
            }
            NativeFunction::Write => {
                let to_register = instruction.a();
                let mut stdout = stdout();
                let map_err = |io_error: io::Error| VmError::Io {
                    error: io_error.kind(),
                    position,
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
                let map_err = |io_error: io::Error| VmError::Io {
                    error: io_error.kind(),
                    position,
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
        };

        Ok(return_value)
    }
}

impl From<u8> for NativeFunction {
    fn from(byte: u8) -> Self {
        match byte {
            PANIC => NativeFunction::Panic,

            // Type conversion
            PARSE => NativeFunction::Parse,
            TO_BYTE => NativeFunction::ToByte,
            TO_FLOAT => NativeFunction::ToFloat,
            TO_INTEGER => NativeFunction::ToInteger,
            TO_STRING => NativeFunction::ToString,

            // List
            ALL => NativeFunction::All,
            ANY => NativeFunction::Any,
            APPEND => NativeFunction::Append,
            CONTAINS => NativeFunction::Contains,
            FIND => NativeFunction::Find,
            FLATTEN => NativeFunction::Flatten,
            GET => NativeFunction::Get,
            INDEX_OF => NativeFunction::IndexOf,
            JOIN => NativeFunction::Join,
            LENGTH => NativeFunction::Length,
            MAP => NativeFunction::Map,
            PREPEND => NativeFunction::Prepend,
            REDUCE => NativeFunction::Reduce,
            REMOVE => NativeFunction::Remove,
            REVERSE => NativeFunction::Reverse,
            SET => NativeFunction::Set,
            SLICE => NativeFunction::Slice,
            SORT => NativeFunction::Sort,
            SPLIT => NativeFunction::Split,
            UNZIP => NativeFunction::Unzip,
            ZIP => NativeFunction::Zip,

            // String
            CHAR_AT => NativeFunction::CharAt,
            CHAR_CODE_AT => NativeFunction::CharCodeAt,
            CHARS => NativeFunction::Chars,
            ENDS_WITH => NativeFunction::EndsWith,
            FORMAT => NativeFunction::Format,
            INCLUDES => NativeFunction::Includes,
            MATCH => NativeFunction::Match,
            PAD_END => NativeFunction::PadEnd,
            PAD_START => NativeFunction::PadStart,
            REPEAT => NativeFunction::Repeat,
            REPLACE => NativeFunction::Replace,
            SPLIT_AT => NativeFunction::SplitAt,
            SPLIT_LINES => NativeFunction::SplitLines,
            SPLIT_WHITESPACE => NativeFunction::SplitWhitespace,
            STARTS_WITH => NativeFunction::StartsWith,
            TO_LOWER_CASE => NativeFunction::ToLowerCase,
            TO_UPPER_CASE => NativeFunction::ToUpperCase,
            TRIM => NativeFunction::Trim,
            TRIM_END => NativeFunction::TrimEnd,
            TRIM_START => NativeFunction::TrimStart,

            // I/O
            READ_LINE => NativeFunction::ReadLine,
            WRITE => NativeFunction::Write,
            WRITE_LINE => NativeFunction::WriteLine,
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
            NativeFunction::Panic => PANIC,
            NativeFunction::Parse => PARSE,
            NativeFunction::ToByte => TO_BYTE,
            NativeFunction::ToFloat => TO_FLOAT,
            NativeFunction::ToInteger => TO_INTEGER,
            NativeFunction::ToString => TO_STRING,
            NativeFunction::All => ALL,
            NativeFunction::Any => ANY,
            NativeFunction::Append => APPEND,
            NativeFunction::Contains => CONTAINS,
            NativeFunction::Find => FIND,
            NativeFunction::Flatten => FLATTEN,
            NativeFunction::Get => GET,
            NativeFunction::IndexOf => INDEX_OF,
            NativeFunction::Join => JOIN,
            NativeFunction::Length => LENGTH,
            NativeFunction::Map => MAP,
            NativeFunction::Prepend => PREPEND,
            NativeFunction::Reduce => REDUCE,
            NativeFunction::Remove => REMOVE,
            NativeFunction::Reverse => REVERSE,
            NativeFunction::Set => SET,
            NativeFunction::Slice => SLICE,
            NativeFunction::Sort => SORT,
            NativeFunction::Split => SPLIT,
            NativeFunction::Unzip => UNZIP,
            NativeFunction::Zip => ZIP,
            NativeFunction::CharAt => CHAR_AT,
            NativeFunction::CharCodeAt => CHAR_CODE_AT,
            NativeFunction::Chars => CHARS,
            NativeFunction::EndsWith => ENDS_WITH,
            NativeFunction::Format => FORMAT,
            NativeFunction::Includes => INCLUDES,
            NativeFunction::Match => MATCH,
            NativeFunction::PadEnd => PAD_END,
            NativeFunction::PadStart => PAD_START,
            NativeFunction::Repeat => REPEAT,
            NativeFunction::Replace => REPLACE,
            NativeFunction::SplitAt => SPLIT_AT,
            NativeFunction::SplitLines => SPLIT_LINES,
            NativeFunction::SplitWhitespace => SPLIT_WHITESPACE,
            NativeFunction::StartsWith => STARTS_WITH,
            NativeFunction::ToLowerCase => TO_LOWER_CASE,
            NativeFunction::ToUpperCase => TO_UPPER_CASE,
            NativeFunction::Trim => TRIM,
            NativeFunction::TrimEnd => TRIM_END,
            NativeFunction::TrimStart => TRIM_START,
            NativeFunction::ReadLine => READ_LINE,
            NativeFunction::WriteLine => WRITE_LINE,
            NativeFunction::Write => WRITE,
        }
    }
}

impl Display for NativeFunction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

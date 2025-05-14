use std::sync::Arc;

use crate::{
    Chunk, ConcreteValue, DustString, FunctionType, Instruction, Local, Scope, Span, Type, Value,
    compile, run,
};

const LOAD_BOOLEAN_TRUE: &str = "true";

const LOAD_BOOLEAN_FALSE: &str = "false";

const LOAD_BYTE: &str = "0x2a";

const LOAD_CHARACTER: &str = "'a'";

const LOAD_FLOAT: &str = "42.42";

const LOAD_INTEGER: &str = "42";

const LOAD_STRING: &str = "\"Hello, World!\"";

const LOAD_BOOLEAN_LIST: &str = "[true, false]";

const LOAD_BYTE_LIST: &str = "[0x2a, 0x42]";

const LOAD_CHARACTER_LIST: &str = "['a', 'b']";

const LOAD_FLOAT_LIST: &str = "[42.42, 24.24]";

const LOAD_INTEGER_LIST: &str = "[1, 2, 3]";

const LOAD_STRING_LIST: &str = "[\"Hello\", \"World\"]";

const LOAD_NESTED_LIST: &str = "[[1, 2], [3, 4]]";

const LOAD_DEEPLY_NESTED_LIST: &str = "[[[1, 2], [3, 4]], [[5, 6], [7, 8]]]";

const LOAD_FUNCTION: &str = "fn () {}";

const LOAD_BOOLEAN_IN_FUNCTION: &str = "fn () { true }";

const LOAD_INTEGER_IN_FUNCTION: &str = "fn () { 42 }";

const LOAD_STRING_IN_FUNCTION: &str = "fn () { \"Hello\" }";

const LOAD_LIST_IN_FUNCTION: &str = "fn () { [1, 2, 3] }";

const LOAD_BYTE_IN_FUNCTION: &str = "fn () { 0x2a }";

const LOAD_CHARACTER_IN_FUNCTION: &str = "fn () { 'a' }";

const LOAD_FLOAT_IN_FUNCTION: &str = "fn () { 42.42 }";

const LOAD_NESTED_LIST_IN_FUNCTION: &str = "fn () { [[1, 2], [3, 4]] }";

const LOAD_DEEPLY_NESTED_LIST_IN_FUNCTION: &str = "fn () { [[[1, 2], [3, 4]], [[5, 6], [7, 8]]] }";

const LOAD_FUNCTION_IN_FUNCTION: &str = "fn outer() { fn inner() -> int { 42 } }";

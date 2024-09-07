use crate::{vm::VmError, LexError, ParseError};

#[derive(Debug, PartialEq)]
pub enum DustError<'src> {
    Lex {
        error: LexError,
        source: &'src str,
    },
    Parse {
        error: ParseError,
        source: &'src str,
    },
    Runtime {
        error: VmError,
        source: &'src str,
    },
}

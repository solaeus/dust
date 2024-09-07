use crate::{bytecode::VmError, LexError, ParseError};

pub enum DustError<'src> {
    LexError {
        error: LexError,
        source: &'src str,
    },
    ParseError {
        error: ParseError,
        source: &'src str,
    },
    VmError {
        error: VmError,
        source: &'src str,
    },
}

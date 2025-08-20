use std::{
    fmt::{self, Display, Formatter},
    range::Range,
};

use serde::{Deserialize, Serialize};

const OVERFLOW_ERROR_TEXT: &str =
    "The source code position is out of bounds because the source file is too large.";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Span(pub u32, pub u32);

impl Span {
    pub fn new<T: TryInto<u32>>(start: T, end: T) -> Self {
        Self(
            start
                .try_into()
                .unwrap_or_else(|_| panic!("{}", OVERFLOW_ERROR_TEXT)),
            end.try_into()
                .unwrap_or_else(|_| panic!("{}", OVERFLOW_ERROR_TEXT)),
        )
    }

    pub fn as_usize_range(&self) -> Range<usize> {
        Range {
            start: self.0 as usize,
            end: self.1 as usize,
        }
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

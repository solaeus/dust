//! Built-in functions that implement extended functionality.

use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NativeFunction(u16);

impl NativeFunction {
    pub const NO_OP: Self = Self(0);

    pub const VEC_WITH_CAPACITY: Self = Self(1);
    pub const VEC_LENGTH: Self = Self(2);
    pub const VEC_GET: Self = Self(3);
    pub const VEC_INSERT: Self = Self(4);
    pub const VEC_REMOVE: Self = Self(5);
    pub const VEC_CLEAR: Self = Self(6);

    pub const READ_LINE: Self = Self(7);
    pub const WRITE_LINE: Self = Self(8);

    pub const SPAWN_THREAD: Self = Self(9);

    pub(super) const fn from_u16(value: u16) -> Self {
        Self(value)
    }

    pub fn as_u16(self) -> u16 {
        self.0
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::NO_OP => "no_op",
            Self::VEC_WITH_CAPACITY => "vec_with_capacity",
            Self::VEC_LENGTH => "vec_length",
            Self::VEC_GET => "vec_get",
            Self::VEC_INSERT => "vec_insert",
            Self::VEC_REMOVE => "vec_remove",
            Self::VEC_CLEAR => "vec_clear",
            Self::READ_LINE => "read_line",
            Self::WRITE_LINE => "write_line",
            Self::SPAWN_THREAD => "spawn_thread",
            _ => "unknown_native_function",
        }
    }
}

impl Display for NativeFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

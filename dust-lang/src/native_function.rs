//! Built-in functions that implement extended functionality.

use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::resolver::{Resolver, declaration_graph::DeclarationId};

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NativeFunction(pub u16);

impl NativeFunction {
    pub const NO_OP: Self = Self(0);

    // `Vec`
    pub const VEC_WITH_CAPACITY: Self = Self(1);
    pub const VEC_LENGTH: Self = Self(2);
    pub const VEC_INSERT: Self = Self(3);
    pub const VEC_REMOVE: Self = Self(4);
    pub const VEC_CLEAR: Self = Self(5);

    // I/O
    pub const READ_LINE: Self = Self(100);
    pub const WRITE_LINE: Self = Self(101);

    // Threads
    pub const SPAWN_THREAD: Self = Self(200);

    pub fn as_str(self) -> &'static str {
        match self {
            Self::NO_OP => "no_op",
            Self::VEC_WITH_CAPACITY => "Vec::with_capacity",
            Self::VEC_LENGTH => "Vec::len",
            Self::VEC_INSERT => "Vec::insert",
            Self::VEC_REMOVE => "Vec::remove",
            Self::VEC_CLEAR => "Vec::clear",
            Self::READ_LINE => "io::read_line",
            Self::WRITE_LINE => "io::write_line",
            Self::SPAWN_THREAD => "io::spawn_thread",
            _ => "<unknown native function>",
        }
    }

    pub fn signature(self, _resolver: &mut Resolver) -> DeclarationId {
        todo!()
    }
}

impl Display for NativeFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

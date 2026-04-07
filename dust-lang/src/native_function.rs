//! Built-in functions that implement extended functionality.

use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::resolver::{Resolver, declarations::DeclarationId};

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum NativeFunction {
    NoOp = 0,

    // Vec
    VecWithCapacity = 1,
    VecLength = 2,
    VecInsert = 3,
    VecRemove = 4,
    VecClear = 5,

    // I/O
    ReadLine = 100,
    WriteLine = 101,

    // Parallelism
    SpawnThread = 200,
}

impl NativeFunction {
    pub fn from_id(id: u16) -> Self {
        match id {
            1 => Self::VecWithCapacity,
            2 => Self::VecLength,
            3 => Self::VecInsert,
            4 => Self::VecRemove,
            5 => Self::VecClear,
            100 => Self::ReadLine,
            101 => Self::WriteLine,
            200 => Self::SpawnThread,
            _ => Self::NoOp,
        }
    }

    pub fn id(self) -> u16 {
        self as u16
    }

    pub fn symbol(self) -> &'static str {
        match self {
            NativeFunction::NoOp => "no_op",
            NativeFunction::VecWithCapacity => "with_capacity",
            NativeFunction::VecLength => "length",
            NativeFunction::VecInsert => "insert",
            NativeFunction::VecRemove => "remove",
            NativeFunction::VecClear => "clear",
            NativeFunction::ReadLine => "read_line",
            NativeFunction::WriteLine => "write_line",
            NativeFunction::SpawnThread => "spawn_thread",
        }
    }

    pub fn signature(self, _resolver: &mut Resolver) -> DeclarationId {
        todo!()
    }
}

impl Display for NativeFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let type_or_module_str = match *self {
            NativeFunction::NoOp => "",
            NativeFunction::VecWithCapacity
            | NativeFunction::VecLength
            | NativeFunction::VecInsert
            | NativeFunction::VecRemove
            | NativeFunction::VecClear => "Vec",
            NativeFunction::ReadLine | NativeFunction::WriteLine => "io",
            NativeFunction::SpawnThread => "thread",
        };

        write!(f, "{type_or_module_str}::{}", self.symbol())
    }
}

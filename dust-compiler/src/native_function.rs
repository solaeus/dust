//! Built-in functions that implement extended functionality.

use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum NativeFunction {
    NoOp = 0,

    // Vec
    VecWithCapacity = 1,
    VecLength = 2,
    VecGet = 3,
    VecInsert = 4,
    VecRemove = 5,
    VecClear = 6,

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
}

impl Display for NativeFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoOp => write!(f, "no_op"),
            Self::VecWithCapacity => write!(f, "Vec::with_capacity"),
            Self::VecLength => write!(f, "Vec::length"),
            Self::VecGet => write!(f, "Vec::get"),
            Self::VecInsert => write!(f, "Vec::insert"),
            Self::VecRemove => write!(f, "Vec::remove"),
            Self::VecClear => write!(f, "Vec::clear"),
            Self::ReadLine => write!(f, "io::read_line"),
            Self::WriteLine => write!(f, "io::write_line"),
            Self::SpawnThread => write!(f, "thread::spawn_thread"),
        }
    }
}

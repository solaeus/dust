use std::{
    fmt::{self, Debug, Display, Formatter},
    sync::Arc,
};

use smallvec::{SmallVec, smallvec};

use crate::{Chunk, DustString};

use super::{Register, action::ActionSequence};

#[derive(Debug)]
pub struct CallFrame {
    pub chunk: Arc<Chunk>,
    pub ip: usize,
    pub return_register: u16,
    pub registers: SmallVec<[Register; 64]>,
    pub action_sequence: ActionSequence,
}

impl CallFrame {
    pub fn new(chunk: Arc<Chunk>, return_register: u16) -> Self {
        let registers = smallvec![Register::Empty; chunk.register_count];
        let action_sequence = ActionSequence::new(&chunk.instructions);

        Self {
            chunk,
            ip: 0,
            return_register,
            registers,
            action_sequence,
        }
    }
}

impl Display for CallFrame {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "FunctionCall: {} | IP: {} | Registers: {}",
            self.chunk
                .name
                .as_ref()
                .unwrap_or(&DustString::from("anonymous")),
            self.ip,
            self.registers.len()
        )
    }
}

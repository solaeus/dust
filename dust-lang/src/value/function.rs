use std::fmt::{self, Formatter};

use tracing::error;

use crate::risky_vm::Thread;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Function {
    pub prototype_index: u16,
}

impl Function {
    pub fn display(&self, f: &mut Formatter, thread: &Thread) -> fmt::Result {
        let (function_name, mut type_display) = if let Some(chunk) = thread
            .current_frame()
            .chunk
            .prototypes
            .get(self.prototype_index as usize)
        {
            (chunk.name.as_ref(), chunk.r#type.to_string())
        } else {
            error!(
                "Failed to display function because its prototype could not be found in the current call frame."
            );

            return Ok(());
        };

        debug_assert!(type_display.starts_with("fn"));

        if let Some(name) = function_name {
            type_display.insert(2, ' ');
            type_display.insert_str(3, name);
        }

        write!(f, "{type_display}")
    }
}

use std::{error::Error, fmt::Display};

use crate::VmError;

#[derive(Debug, Clone, PartialEq)]
pub struct DustError<'src> {
    vm_error: VmError,
    source: &'src str,
}

impl Error for DustError<'_> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.vm_error)
    }
}

impl Display for DustError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}", self.vm_error, self.source)
    }
}

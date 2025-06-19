use crate::Module;

use super::Path;

/// Indication of what the compiler will produce when it finishes.
#[derive(Debug)]
pub enum CompileMode<C> {
    /// Indicates that the compiler should produce a function prototype.
    Function { name: Option<Path> },

    /// Indicates that the compiler should produce a stand-alone Dust program.
    Main { name: Path },

    /// Indicates that the compiler should parse values and place them in the namespace.
    Module { name: Path, module: Module<C> },
}

impl<C> CompileMode<C> {
    pub fn into_name(self) -> Option<Path> {
        match self {
            CompileMode::Function { name } => name,
            CompileMode::Main { name } => Some(name),
            CompileMode::Module { name, .. } => Some(name),
        }
    }
}

use crate::Module;

use super::Path;

/// Indication of what the compiler will produce when it finishes.
#[derive(Debug)]
pub enum CompileMode {
    /// Indicates that the compiler should produce a function prototype.
    Function { name: Option<Path> },

    /// Indicates that the compiler should produce a stand-alone Dust program.
    Main,

    /// Indicates that the compiler should parse values and place them in the namespace.
    Module { name: Path, module: Module },
}

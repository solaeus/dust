use crate::Module;

use super::Path;

/// Indication of what the compiler will produce when it finishes.
#[derive(Debug)]
pub enum CompileMode<'a, C> {
    /// Indicates that the compiler should produce a function prototype.
    Function { name: Option<Path<'a>> },

    /// Indicates that the compiler should produce a stand-alone Dust program.
    Main { name: Option<&'a str> },

    /// Indicates that the compiler should parse values and place them in the namespace.
    Module {
        name: Path<'a>,
        module: Module<'a, C>,
    },
}

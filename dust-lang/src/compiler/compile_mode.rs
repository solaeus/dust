use crate::{DustString, Module};

/// Indication of what the compiler will produce when it finishes.
#[derive(Debug)]
pub enum CompileMode<'a, C> {
    /// Indicates that the compiler should produce a function prototype.
    Function { name: Option<DustString> },

    /// Indicates that the compiler should produce a stand-alone Dust program.
    Main { name: Option<&'a str> },

    /// Indicates that the compiler should parse values and place them in the namespace.
    Module {
        name: &'a str,
        module: Module<'a, C>,
    },
}

impl<'a, C> CompileMode<'a, C> {
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Function { name } => name.as_ref().map(|dust_string| dust_string.as_str()),
            Self::Main { name } => *name,
            Self::Module { name, .. } => Some(name),
        }
    }

    pub fn is_module(&self) -> bool {
        matches!(self, Self::Module { .. })
    }
}

use super::Module;

/// Indication of what the compiler will produce when it finishes.
#[derive(Debug)]
pub enum CompileMode<'a> {
    /// Indicates that the compiler should produce a function prototype.
    Function { name: Option<&'a str> },

    /// Indicates that the compiler should produce a stand-alone Dust program.
    Main { name: Option<&'a str> },

    /// Indicates that the compiler should parse values and place them in the namespace.
    Module { name: &'a str, module: Module<'a> },
}

impl<'a> CompileMode<'a> {
    pub fn into_name(self) -> Option<&'a str> {
        match self {
            Self::Function { name } => name,
            Self::Main { name } => name,
            Self::Module { name, .. } => Some(name),
        }
    }

    pub fn is_module(&self) -> bool {
        matches!(self, Self::Module { .. })
    }
}

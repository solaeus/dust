use crate::DustString;

/// Indication of what the compiler will produce when it finishes.
#[derive(Debug)]
pub enum CompileMode {
    /// Indicates that the compiler should produce a function prototype.
    Function { name: Option<DustString> },

    /// Indicates that the compiler should produce a stand-alone Dust program.
    Main { name: Option<DustString> },
}

impl CompileMode {
    pub fn into_name(self) -> Option<DustString> {
        match self {
            Self::Function { name } => name,
            Self::Main { name } => name,
        }
    }
}

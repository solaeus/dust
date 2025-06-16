use crate::Type;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Global {
    pub r#type: Type,
    pub is_mutable: bool,
}

impl Global {
    pub fn new(r#type: Type, is_mutable: bool) -> Self {
        Self { r#type, is_mutable }
    }
}

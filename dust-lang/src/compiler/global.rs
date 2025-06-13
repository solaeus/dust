use crate::Type;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Global {
    pub cell_index: u16,
    pub r#type: Type,
    pub is_mutable: bool,
}

impl Global {
    pub fn new(cell_index: u16, r#type: Type, is_mutable: bool) -> Self {
        Self {
            cell_index,
            r#type,
            is_mutable,
        }
    }
}

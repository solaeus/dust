use crate::resolver::TypeId;

#[derive(Clone, Copy, Debug)]
pub struct Local {
    pub register: u16,

    pub r#type: TypeId,
}

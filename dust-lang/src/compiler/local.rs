use crate::resolver::{DeclarationId, TypeId};

#[derive(Clone, Copy, Debug)]
pub struct Local {
    pub declaration_id: DeclarationId,

    pub register: u16,

    pub r#type: TypeId,
}

use super::{Expression, Type};

pub struct As {
    expression: Expression,
    r#type: Type,
}

impl As {
    pub fn new(expression: Expression, r#type: Type) -> Self {
        Self { expression, r#type }
    }
}

use std::fmt::{self, Display, Formatter};

use crate::FunctionType;

use super::DustString;

#[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Function {
    pub name: Option<DustString>,
    pub r#type: FunctionType,
    pub prototype_index: u16,
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut type_string = self.r#type.to_string();

        if let Some(name) = &self.name {
            debug_assert!(type_string.starts_with("fn"));

            type_string.insert(2, ' ');
            type_string.insert_str(3, name);
        }

        write!(f, "{type_string}")
    }
}

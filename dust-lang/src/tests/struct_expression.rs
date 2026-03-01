use crate::function_wrapper;

pub const EMPTY_FIELDS: &[u8] = function_wrapper!("Foo {}");
pub const FIELDS: &[u8] = function_wrapper!("Foo { x: 42, y: 666 }");

use crate::function_wrapper;

pub const IF: &[u8] = function_wrapper!("if condition { x + y }");
pub const IF_ELSE: &[u8] = function_wrapper!("if condition { x + y } else { x - y }");
pub const IF_ELSE_IF: &[u8] =
    function_wrapper!("if left { x + y } else if right { x - y } else { x * y }");

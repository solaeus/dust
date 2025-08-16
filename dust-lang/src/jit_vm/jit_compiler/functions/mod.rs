pub mod lists;
pub mod strings;

pub use lists::*;
pub use strings::*;
use tracing::trace;

use crate::Operation;

#[unsafe(no_mangle)]
pub extern "C" fn log_operation(op_code: i64) {
    trace!("Running operation: {}", Operation(op_code as u8));
}

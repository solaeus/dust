pub mod return_value_setters;
pub mod string_manipulation;

pub use return_value_setters::*;
pub use string_manipulation::*;
use tracing::trace;

use crate::Operation;

pub extern "C" fn log_operation(op_code: i64) {
    trace!("Running operation: {}", Operation(op_code as u8));
}

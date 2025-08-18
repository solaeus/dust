pub mod lists;
pub mod strings;

pub use lists::*;
pub use strings::*;
use tracing::info;

use crate::Operation;

#[unsafe(no_mangle)]
pub extern "C" fn log_operation_and_ip(op_code: i8, ip: i64) {
    let operation = Operation(op_code as u8);

    info!("Running {operation} at {ip}");
}

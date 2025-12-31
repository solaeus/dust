pub mod io;
pub mod lists;
pub mod math;
pub mod strings;
pub mod threads;

pub use io::*;
pub use lists::*;
pub use math::*;
pub use strings::*;
pub use threads::*;

use tracing::info;

use crate::instruction::Operation;

#[unsafe(no_mangle)]
pub extern "C" fn log_operation_and_ip(op_code: i8, ip: i64) {
    let operation = Operation(op_code as u8);

    info!("Running {operation} at {ip}");
}

#[unsafe(no_mangle)]
pub extern "C" fn log_integer(value: i64) {
    info!("Integer: {}", value);
}

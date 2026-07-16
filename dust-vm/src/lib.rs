#![feature(thread_id_value, current_thread_id)]

mod call;
pub mod error;
mod object;
mod object_pool;
mod register;
mod thread;
mod thread_pool;
mod vm;

pub use vm::*;

pub const MINIMUM_OBJECT_HEAP_DEFAULT: usize = if cfg!(debug_assertions) {
    1024
} else {
    1024 * 1024 * 4
};
pub const MINIMUM_OBJECT_SWEEP_DEFAULT: usize = if cfg!(debug_assertions) {
    256
} else {
    1024 * 1024
};

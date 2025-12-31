use std::sync::Arc;

use crate::{dust_error::DustError, jit_vm::thread_pool::ThreadContext};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn spawn(prototype_index: i64, thread_context: *mut ThreadContext) {
    let thread_context = unsafe { &mut *thread_context };
    let thread_spawner = unsafe { &*thread_context.thread_spawner_pointer };
    let _ = thread_spawner
        .lock()
        .expect("Failed to lock thread spawner")
        .spawn_thread(prototype_index as u16, Arc::clone(thread_spawner))
        .map_err(|error| {
            let report = DustError::jit(error).report();

            eprintln!("{report}");
        });
}

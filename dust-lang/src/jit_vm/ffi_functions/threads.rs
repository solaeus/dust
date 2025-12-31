use std::sync::Arc;

use crate::{
    dust_error::DustError,
    jit_vm::{ERROR_REPLACEMENT_STR, Object, thread_pool::ThreadContext},
};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn spawn(
    thread_name: *const Object,
    prototype_index: i64,
    thread_context: *mut ThreadContext,
) {
    let thread_context = unsafe { &mut *thread_context };
    let thread_spawner = unsafe { &*thread_context.thread_spawner_pointer };
    let thread_name = unsafe { &*thread_name }
        .as_string()
        .cloned()
        .unwrap_or_else(|| ERROR_REPLACEMENT_STR.to_string());
    let _ = thread_spawner
        .lock()
        .expect("Failed to lock thread spawner")
        .spawn_thread(
            thread_name,
            prototype_index as u16,
            Arc::clone(thread_spawner),
        )
        .map_err(|error| {
            let report = DustError::jit(error).report();

            eprintln!("{report}");
        });
}

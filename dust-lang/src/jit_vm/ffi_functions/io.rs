use std::io::{Write, stdin, stdout};

use crate::jit_vm::{ERROR_REPLACEMENT_STR, Object, thread_pool::ThreadContext};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn read_line(thread_context: *mut ThreadContext) -> i64 {
    let thread_context = unsafe { &mut *thread_context };

    let object_pool = unsafe { &mut *thread_context.object_pool_pointer };
    let register_stack = unsafe { &mut *thread_context.register_vec_pointer };
    let register_tags = unsafe { &mut *thread_context.register_tag_vec_pointer };
    let register_window = &register_stack[0..thread_context.registers_used];
    let register_tags_window = &register_tags[0..thread_context.registers_used];

    let mut input = String::new();
    let read_result = stdin().read_line(&mut input);

    if read_result.is_err() {
        input.push_str(ERROR_REPLACEMENT_STR);
    }

    #[cfg(not(target_os = "windows"))]
    input.pop();

    #[cfg(target_os = "windows")]
    if input.ends_with("\r\n") {
        input.pop();
        input.pop();
    } else {
        input.pop();
    }

    let object = Object::string(input);
    let object_pointer = object_pool.allocate(object, register_window, register_tags_window);

    object_pointer as i64
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn write_line(message: *const Object) {
    let string = unsafe { &*message }
        .as_string()
        .map(|string| string.as_str())
        .unwrap_or(ERROR_REPLACEMENT_STR);

    let mut stdout = stdout().lock();
    let _ = stdout.write_all(string.as_bytes());
    let _ = stdout.write_all(b"\n");
    let _ = stdout.flush();
}

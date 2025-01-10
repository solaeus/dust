use std::{
    ops::Range,
    thread::{Builder, JoinHandle},
};

use tracing::{Level, info, span};

use crate::{
    DustString,
    vm::{Thread, ThreadData, get_next_action},
};

fn start_thread(data: &mut ThreadData, argument_range: Range<u8>) -> JoinHandle<()> {
    let mut argument_range_iter = argument_range.into_iter();
    let function_argument = {
        loop {
            let register_index = argument_range_iter
                .next()
                .unwrap_or_else(|| panic!("No argument was passed to \"spawn\""));
            let value_option = data.open_register_allow_empty_unchecked(register_index);

            if let Some(argument) = value_option {
                break argument;
            }
        }
    };
    let function = function_argument.as_function().unwrap();
    let prototype_index = function.prototype_index as usize;
    let current_call = data.call_stack.last_unchecked();
    let prototype = current_call.chunk.prototypes[prototype_index].clone();

    info!(
        "Spawning thread for \"{}\"",
        function
            .name
            .as_ref()
            .cloned()
            .unwrap_or_else(|| DustString::from("anonymous"))
    );

    let thread_name = prototype
        .name
        .as_ref()
        .map(|name| name.to_string())
        .unwrap_or_else(|| "anonymous".to_string());
    let mut thread = Thread::new(prototype);

    Builder::new()
        .name(thread_name)
        .spawn(move || {
            let span = span!(Level::INFO, "Spawned thread");
            let _enter = span.enter();

            thread.run();
        })
        .expect("Critical VM Error: Failed to spawn thread")
}

pub fn spawn(data: &mut ThreadData, _: u8, argument_range: Range<u8>) -> bool {
    let _ = start_thread(data, argument_range);
    data.next_action = get_next_action(data);

    false
}

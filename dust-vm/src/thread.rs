use std::{sync::Arc, thread::current_id};

use crossbeam::channel::Sender;
use dust_compiler::program::Program;

use crate::{
    call::{Call, CallFrame},
    error::VmError,
    register::Register,
    thread_pool::ThreadMessage,
};

pub struct Thread {
    program: Arc<Program>,
    main_prototype_index: u32,

    call_stack: Vec<CallFrame>,
    register_stack: Vec<Register>,

    message_sender: Arc<Sender<ThreadMessage>>,
}

impl Thread {
    pub fn new(
        program: Arc<Program>,
        main_prototype_index: u32,
        message_sender: Arc<Sender<ThreadMessage>>,
    ) -> Self {
        let call_stack_capacity = if program.prototypes().len() == 1 {
            0
        } else {
            256
        };
        let register_count = if program.prototypes().len() == 1 {
            program.prototypes()[0].register_count as usize
        } else {
            (program.prototypes()[0].register_count as usize).max(256)
        };

        Thread {
            program,
            main_prototype_index,
            call_stack: Vec::with_capacity(call_stack_capacity),
            register_stack: vec![Register::new(0); register_count],
            message_sender,
        }
    }

    pub fn run(mut self) {
        let call = match Call::new(
            self.program.prototypes(),
            self.program.constants(),
            &mut self.register_stack,
            &mut self.call_stack,
            self.main_prototype_index,
        ) {
            Ok(call) => call,
            Err(error) => {
                let _ = self.message_sender.send(ThreadMessage::ThreadError {
                    thread_id: current_id(),
                    error,
                });

                return;
            }
        };

        match call.run() {
            Ok(()) => {
                let base_frame = match self.call_stack.pop() {
                    Some(frame) => frame,
                    None => {
                        let _ = self.message_sender.send(ThreadMessage::ThreadError {
                            thread_id: current_id(),
                            error: VmError::CallStackUnderflow,
                        });

                        return;
                    }
                };

                self.register_stack
                    .truncate(base_frame.register_count as usize);

                let _ = self.message_sender.send(ThreadMessage::ThreadFinished {
                    thread_id: current_id(),
                    return_registers: self.register_stack,
                });
            }
            Err(error) => {
                let _ = self.message_sender.send(ThreadMessage::ThreadError {
                    thread_id: current_id(),
                    error,
                });
            }
        }
    }
}

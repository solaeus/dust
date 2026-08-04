use std::{sync::Arc, thread::current_id};

use crossbeam::channel::Sender;
use dust_compiler::program::Program;

use crate::{
    call::{Call, CallFrame},
    register::Register,
    thread_pool::ThreadMessage,
};

pub struct Thread {
    program: Arc<Program>,
    main_prototype_index: usize,

    call_stack: Vec<CallFrame>,
    register_stack: Vec<Register>,

    message_sender: Arc<Sender<ThreadMessage>>,
}

impl Thread {
    pub fn new(
        program: Arc<Program>,
        main_prototype_index: usize,
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
            1024
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
        let starting_call_frame = CallFrame {
            prototype_index: self.main_prototype_index,
            base_register: 0,
            instruction_pointer: 0,
        };

        self.call_stack.push(starting_call_frame);

        let return_registers = loop {
            let call = match Call::new(
                self.program.prototypes(),
                self.program.constants(),
                &mut self.register_stack,
                &mut self.call_stack,
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
                Ok(Some(return_registers)) => break return_registers,
                Ok(None) => {}
                Err(error) => {
                    let _ = self.message_sender.send(ThreadMessage::ThreadError {
                        thread_id: current_id(),
                        error,
                    });

                    return;
                }
            }
        };

        let _ = self.message_sender.send(ThreadMessage::ThreadFinished {
            thread_id: current_id(),
            return_registers,
        });
    }
}

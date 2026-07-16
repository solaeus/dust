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
    main_prototype_index: u16,

    call_stack: Vec<CallFrame>,
    register_stack: Vec<Register>,

    message_sender: Arc<Sender<ThreadMessage>>,
}

impl Thread {
    pub fn new(
        program: Arc<Program>,
        main_prototype_index: u16,
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

    pub fn run(mut self) -> Result<(), VmError> {
        let starting_prototype = self
            .program
            .prototypes()
            .as_slice()
            .get(self.main_prototype_index as usize)
            .ok_or(VmError::InvalidPrototypeIndex {
                index: self.main_prototype_index,
            })?;
        let starting_call_frame = CallFrame {
            prototype_id: self.main_prototype_index,
            regsiter_range_start: 0,
            register_range_end: starting_prototype.register_count,
            instruction_pointer: 0,
        };

        self.call_stack.push(starting_call_frame);

        let return_registers = loop {
            let current_call_frame = self.call_stack.last().ok_or(VmError::CallStackUnderflow)?;
            let prototype = self
                .program
                .prototypes()
                .get(current_call_frame.prototype_id as usize)
                .ok_or(VmError::InvalidPrototypeIndex {
                    index: current_call_frame.prototype_id,
                })?;
            let call = Call::new(
                current_call_frame.instruction_pointer,
                prototype,
                &mut self.register_stack,
                self.program.constants(),
                &mut self.call_stack,
            )?;

            if let Some(return_registers) = call.run()? {
                break return_registers;
            }
        };

        self.message_sender.send(ThreadMessage::RemoveThread {
            thread_id: current_id(),
            return_registers,
        })?;

        Ok(())
    }
}

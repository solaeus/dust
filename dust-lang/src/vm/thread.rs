use std::{collections::HashMap, sync::Arc, thread::JoinHandle};

use tracing::{info, trace};

use crate::{AbstractList, Chunk, ConcreteValue, DustString, Span, Value, vm::CallFrame};

use super::call_frame::{Pointer, Register};

pub struct Thread {
    chunk: Arc<Chunk>,
    call_stack: Vec<CallFrame>,
    pub return_value: Option<Option<Value>>,
    pub integer_cache: HashMap<usize, *const i64>,
    _spawned_threads: Vec<JoinHandle<()>>,
}

impl Thread {
    pub fn new(chunk: Arc<Chunk>) -> Self {
        let mut call_stack = Vec::with_capacity(chunk.prototypes.len() + 1);
        let main_call = CallFrame::new(Arc::clone(&chunk), 0);

        call_stack.push(main_call);

        Thread {
            chunk,
            call_stack,
            return_value: None,
            integer_cache: HashMap::new(),
            _spawned_threads: Vec::new(),
        }
    }

    pub fn run(mut self) -> Option<Value> {
        info!(
            "Starting thread with {}",
            self.chunk
                .name
                .clone()
                .unwrap_or_else(|| DustString::from("anonymous"))
        );

        loop {
            let current_frame = self.current_frame_mut();
            let ip = {
                let ip = current_frame.ip;
                current_frame.ip += 1;

                ip
            };
            let current_action = if cfg!(debug_assertions) {
                current_frame.action_sequence.actions.get_mut(ip).unwrap()
            } else {
                unsafe { current_frame.action_sequence.actions.get_unchecked_mut(ip) }
            };

            trace!("Instruction: {}", current_action.instruction.operation);

            (current_action.logic)(current_action.instruction, &mut self);

            if let Some(return_value_option) = self.return_value {
                return return_value_option;
            }
        }
    }

    pub fn current_position(&self) -> Span {
        let current_frame = self.current_frame();

        current_frame.chunk.positions[current_frame.ip]
    }

    pub fn current_frame(&self) -> &CallFrame {
        if cfg!(debug_assertions) {
            self.call_stack.last().unwrap()
        } else {
            unsafe { self.call_stack.last().unwrap_unchecked() }
        }
    }

    pub fn current_frame_mut(&mut self) -> &mut CallFrame {
        if cfg!(debug_assertions) {
            self.call_stack.last_mut().unwrap()
        } else {
            unsafe { self.call_stack.last_mut().unwrap_unchecked() }
        }
    }

    pub fn get_value_from_pointer(&self, pointer: &Pointer) -> ConcreteValue {
        match pointer {
            Pointer::RegisterBoolean(register_index) => {
                let boolean = *self.get_boolean_register(*register_index);

                ConcreteValue::Boolean(boolean)
            }
            Pointer::RegisterByte(register_index) => {
                let byte = *self.get_byte_register(*register_index);

                ConcreteValue::Byte(byte)
            }
            Pointer::RegisterCharacter(register_index) => {
                let character = *self.get_character_register(*register_index);

                ConcreteValue::Character(character)
            }
            Pointer::RegisterFloat(register_index) => {
                let float = *self.get_float_register(*register_index);

                ConcreteValue::Float(float)
            }
            Pointer::RegisterInteger(register_index) => {
                let integer = *self.get_integer_register(*register_index);

                ConcreteValue::Integer(integer)
            }
            Pointer::RegisterString(register_index) => {
                let string = self.get_string_register(*register_index).clone();

                ConcreteValue::String(string)
            }
            Pointer::RegisterList(register_index) => {
                let abstract_list = self.get_list_register(*register_index).clone();
                let mut items = Vec::with_capacity(abstract_list.item_pointers.len());

                for pointer in &abstract_list.item_pointers {
                    let value = self.get_value_from_pointer(pointer);

                    items.push(value);
                }

                ConcreteValue::List {
                    items,
                    item_type: abstract_list.item_type,
                }
            }
            Pointer::RegisterFunction(_) => unimplemented!(),
            Pointer::ConstantCharacter(constant_index) => {
                let character = *self.get_constant(*constant_index).as_character().unwrap();

                ConcreteValue::Character(character)
            }
            Pointer::ConstantFloat(constant_index) => {
                let float = *self.get_constant(*constant_index).as_float().unwrap();

                ConcreteValue::Float(float)
            }
            Pointer::ConstantInteger(constant_index) => {
                let integer = *self.get_constant(*constant_index).as_integer().unwrap();

                ConcreteValue::Integer(integer)
            }
            Pointer::ConstantString(constant_index) => {
                let string = self
                    .get_constant(*constant_index)
                    .as_string()
                    .unwrap()
                    .clone();

                ConcreteValue::String(string)
            }
        }
    }

    pub fn get_boolean_register(&self, register_index: usize) -> &bool {
        let register = if cfg!(debug_assertions) {
            self.call_stack
                .last()
                .unwrap()
                .registers
                .booleans
                .get(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last()
                    .unwrap_unchecked()
                    .registers
                    .booleans
                    .get_unchecked(register_index)
            }
        };

        match register {
            Register::Value(value) => value,
            Register::Closed(value) => value,
            Register::Pointer(pointer) => self.get_pointer_to_boolean(pointer),
            Register::Empty => panic!("Attempted to get value from empty register"),
        }
    }

    pub fn get_pointer_to_boolean(&self, pointer: &Pointer) -> &bool {
        match pointer {
            Pointer::RegisterBoolean(register_index) => self.get_boolean_register(*register_index),
            _ => panic!("Attempted to get boolean from non-boolean pointer"),
        }
    }

    pub fn set_boolean_register(&mut self, register_index: usize, new_register: Register<bool>) {
        let old_register = if cfg!(debug_assertions) {
            self.call_stack
                .last_mut()
                .unwrap()
                .registers
                .booleans
                .get_mut(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last_mut()
                    .unwrap_unchecked()
                    .registers
                    .booleans
                    .get_unchecked_mut(register_index)
            }
        };

        *old_register = new_register;
    }

    pub fn is_boolean_register_closed(&self, register_index: usize) -> bool {
        let register = if cfg!(debug_assertions) {
            self.call_stack
                .last()
                .unwrap()
                .registers
                .booleans
                .get(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last()
                    .unwrap_unchecked()
                    .registers
                    .booleans
                    .get_unchecked(register_index)
            }
        };

        matches!(register, Register::Closed(_))
    }

    pub fn close_boolean_register(&mut self, register_index: usize) {
        self.current_frame_mut()
            .registers
            .booleans
            .get_mut(register_index)
            .unwrap()
            .close();
    }

    pub fn get_byte_register(&self, register_index: usize) -> &u8 {
        let register = if cfg!(debug_assertions) {
            self.call_stack
                .last()
                .unwrap()
                .registers
                .bytes
                .get(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last()
                    .unwrap_unchecked()
                    .registers
                    .bytes
                    .get_unchecked(register_index)
            }
        };

        match register {
            Register::Value(value) => value,
            Register::Closed(value) => value,
            Register::Pointer(pointer) => self.get_pointer_to_byte(pointer),
            Register::Empty => panic!("Attempted to get value from empty register"),
        }
    }

    pub fn get_pointer_to_byte(&self, pointer: &Pointer) -> &u8 {
        match pointer {
            Pointer::RegisterByte(register_index) => self.get_byte_register(*register_index),
            _ => panic!("Attempted to get byte from non-byte pointer"),
        }
    }

    pub fn set_byte_register(&mut self, register_index: usize, new_register: Register<u8>) {
        let old_register = if cfg!(debug_assertions) {
            self.call_stack
                .last_mut()
                .unwrap()
                .registers
                .bytes
                .get_mut(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last_mut()
                    .unwrap_unchecked()
                    .registers
                    .bytes
                    .get_unchecked_mut(register_index)
            }
        };

        *old_register = new_register;
    }

    pub fn is_byte_register_closed(&self, register_index: usize) -> bool {
        let register = if cfg!(debug_assertions) {
            self.call_stack
                .last()
                .unwrap()
                .registers
                .bytes
                .get(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last()
                    .unwrap_unchecked()
                    .registers
                    .bytes
                    .get_unchecked(register_index)
            }
        };

        matches!(register, Register::Closed(_))
    }

    pub fn close_byte_register(&mut self, register_index: usize) {
        self.current_frame_mut()
            .registers
            .bytes
            .get_mut(register_index)
            .unwrap()
            .close();
    }

    pub fn get_character_register(&self, register_index: usize) -> &char {
        let register = if cfg!(debug_assertions) {
            self.call_stack
                .last()
                .unwrap()
                .registers
                .characters
                .get(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last()
                    .unwrap_unchecked()
                    .registers
                    .characters
                    .get_unchecked(register_index)
            }
        };

        match register {
            Register::Value(value) => value,
            Register::Closed(value) => value,
            Register::Pointer(pointer) => self.get_pointer_to_character(pointer),
            Register::Empty => panic!("Attempted to get value from empty register"),
        }
    }

    pub fn get_pointer_to_character(&self, pointer: &Pointer) -> &char {
        match pointer {
            Pointer::RegisterCharacter(register_index) => {
                self.get_character_register(*register_index)
            }
            Pointer::ConstantCharacter(constant_index) => {
                self.get_constant(*constant_index).as_character().unwrap()
            }
            _ => panic!("Attempted to get character from non-character pointer"),
        }
    }

    pub fn set_character_register(&mut self, register_index: usize, new_register: Register<char>) {
        let old_register = if cfg!(debug_assertions) {
            self.call_stack
                .last_mut()
                .unwrap()
                .registers
                .characters
                .get_mut(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last_mut()
                    .unwrap_unchecked()
                    .registers
                    .characters
                    .get_unchecked_mut(register_index)
            }
        };

        *old_register = new_register;
    }

    pub fn is_character_register_closed(&self, register_index: usize) -> bool {
        let register = if cfg!(debug_assertions) {
            self.call_stack
                .last()
                .unwrap()
                .registers
                .characters
                .get(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last()
                    .unwrap_unchecked()
                    .registers
                    .characters
                    .get_unchecked(register_index)
            }
        };

        matches!(register, Register::Closed(_))
    }

    pub fn close_character_register(&mut self, register_index: usize) {
        self.current_frame_mut()
            .registers
            .characters
            .get_mut(register_index)
            .unwrap()
            .close();
    }

    pub fn get_float_register(&self, register_index: usize) -> &f64 {
        let register = if cfg!(debug_assertions) {
            self.call_stack
                .last()
                .unwrap()
                .registers
                .floats
                .get(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last()
                    .unwrap_unchecked()
                    .registers
                    .floats
                    .get_unchecked(register_index)
            }
        };

        match register {
            Register::Value(value) => value,
            Register::Closed(value) => value,
            Register::Pointer(pointer) => self.get_pointer_to_float(pointer),
            Register::Empty => panic!("Attempted to get value from empty register"),
        }
    }

    pub fn get_pointer_to_float(&self, pointer: &Pointer) -> &f64 {
        match pointer {
            Pointer::RegisterFloat(register_index) => self.get_float_register(*register_index),
            Pointer::ConstantFloat(constant_index) => {
                self.get_constant(*constant_index).as_float().unwrap()
            }
            _ => panic!("Attempted to get float from non-float pointer"),
        }
    }

    pub fn set_float_register(&mut self, register_index: usize, new_register: Register<f64>) {
        let old_register = if cfg!(debug_assertions) {
            self.call_stack
                .last_mut()
                .unwrap()
                .registers
                .floats
                .get_mut(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last_mut()
                    .unwrap_unchecked()
                    .registers
                    .floats
                    .get_unchecked_mut(register_index)
            }
        };

        *old_register = new_register;
    }

    pub fn is_float_register_closed(&self, register_index: usize) -> bool {
        let register = if cfg!(debug_assertions) {
            self.call_stack
                .last()
                .unwrap()
                .registers
                .floats
                .get(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last()
                    .unwrap_unchecked()
                    .registers
                    .floats
                    .get_unchecked(register_index)
            }
        };

        matches!(register, Register::Closed(_))
    }

    pub fn close_float_register(&mut self, register_index: usize) {
        self.current_frame_mut()
            .registers
            .floats
            .get_mut(register_index)
            .unwrap()
            .close();
    }

    pub fn get_integer_register(&self, register_index: usize) -> &i64 {
        let register = if cfg!(debug_assertions) {
            self.call_stack
                .last()
                .unwrap()
                .registers
                .integers
                .get(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last()
                    .unwrap_unchecked()
                    .registers
                    .integers
                    .get_unchecked(register_index)
            }
        };

        match register {
            Register::Value(value) => value,
            Register::Closed(value) => value,
            Register::Pointer(pointer) => self.get_pointer_to_integer(pointer),
            Register::Empty => panic!("Attempted to get value from empty register"),
        }
    }

    pub fn get_pointer_to_integer(&self, pointer: &Pointer) -> &i64 {
        match pointer {
            Pointer::RegisterInteger(register_index) => self.get_integer_register(*register_index),
            Pointer::ConstantInteger(constant_index) => {
                self.get_constant(*constant_index).as_integer().unwrap()
            }
            _ => panic!("Attempted to get integer from non-integer pointer"),
        }
    }

    pub fn set_integer_register(&mut self, register_index: usize, new_register: Register<i64>) {
        let old_register = if cfg!(debug_assertions) {
            self.call_stack
                .last_mut()
                .unwrap()
                .registers
                .integers
                .get_mut(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last_mut()
                    .unwrap_unchecked()
                    .registers
                    .integers
                    .get_unchecked_mut(register_index)
            }
        };

        *old_register = new_register;
    }

    pub fn is_integer_register_closed(&self, register_index: usize) -> bool {
        let register = if cfg!(debug_assertions) {
            self.call_stack
                .last()
                .unwrap()
                .registers
                .integers
                .get(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last()
                    .unwrap_unchecked()
                    .registers
                    .integers
                    .get_unchecked(register_index)
            }
        };

        matches!(register, Register::Closed(_))
    }

    pub fn close_integer_register(&mut self, register_index: usize) {
        self.current_frame_mut()
            .registers
            .integers
            .get_mut(register_index)
            .unwrap()
            .close();
    }

    pub fn get_string_register(&self, register_index: usize) -> &DustString {
        let register = if cfg!(debug_assertions) {
            self.call_stack
                .last()
                .unwrap()
                .registers
                .strings
                .get(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last()
                    .unwrap_unchecked()
                    .registers
                    .strings
                    .get_unchecked(register_index)
            }
        };

        match register {
            Register::Value(value) => value,
            Register::Closed(value) => value,
            Register::Pointer(pointer) => self.get_pointer_to_string(pointer),
            Register::Empty => panic!("Attempted to get value from empty register"),
        }
    }

    pub fn get_pointer_to_string(&self, pointer: &Pointer) -> &DustString {
        match pointer {
            Pointer::RegisterString(register_index) => self.get_string_register(*register_index),
            Pointer::ConstantString(constant_index) => {
                self.get_constant(*constant_index).as_string().unwrap()
            }
            _ => panic!("Attempted to get string from non-string pointer"),
        }
    }

    pub fn set_string_register(
        &mut self,
        register_index: usize,
        new_register: Register<DustString>,
    ) {
        let old_register = if cfg!(debug_assertions) {
            self.call_stack
                .last_mut()
                .unwrap()
                .registers
                .strings
                .get_mut(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last_mut()
                    .unwrap_unchecked()
                    .registers
                    .strings
                    .get_unchecked_mut(register_index)
            }
        };

        *old_register = new_register;
    }

    pub fn is_string_register_closed(&self, register_index: usize) -> bool {
        let register = if cfg!(debug_assertions) {
            self.call_stack
                .last()
                .unwrap()
                .registers
                .strings
                .get(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last()
                    .unwrap_unchecked()
                    .registers
                    .strings
                    .get_unchecked(register_index)
            }
        };

        matches!(register, Register::Closed(_))
    }

    pub fn close_string_register(&mut self, register_index: usize) {
        let current_frame = self.current_frame_mut();

        current_frame.registers.strings.push(Register::Empty);

        let old_register = current_frame.registers.strings.swap_remove(register_index);

        if let Register::Value(value) = old_register {
            current_frame
                .registers
                .strings
                .push(Register::Closed(value));

            let _ = current_frame.registers.strings.swap_remove(register_index);
        } else {
            panic!("Attempted to close non-value register");
        }
    }

    pub fn get_list_register(&self, register_index: usize) -> &AbstractList {
        let register = if cfg!(debug_assertions) {
            self.call_stack
                .last()
                .unwrap()
                .registers
                .lists
                .get(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last()
                    .unwrap_unchecked()
                    .registers
                    .lists
                    .get_unchecked(register_index)
            }
        };

        match register {
            Register::Value(value) => value,
            Register::Closed(value) => value,
            Register::Pointer(pointer) => self.get_pointer_to_list(pointer),
            Register::Empty => panic!("Attempted to get value from empty register"),
        }
    }

    pub fn get_pointer_to_list(&self, pointer: &Pointer) -> &AbstractList {
        match pointer {
            Pointer::RegisterList(register_index) => self.get_list_register(*register_index),
            _ => panic!("Attempted to get list from non-list pointer"),
        }
    }

    pub fn set_list_register(
        &mut self,
        register_index: usize,
        new_register: Register<AbstractList>,
    ) {
        let old_register = if cfg!(debug_assertions) {
            self.call_stack
                .last_mut()
                .unwrap()
                .registers
                .lists
                .get_mut(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last_mut()
                    .unwrap_unchecked()
                    .registers
                    .lists
                    .get_unchecked_mut(register_index)
            }
        };

        *old_register = new_register;
    }

    pub fn is_list_register_closed(&self, register_index: usize) -> bool {
        let register = if cfg!(debug_assertions) {
            self.call_stack
                .last()
                .unwrap()
                .registers
                .lists
                .get(register_index)
                .unwrap()
        } else {
            unsafe {
                self.call_stack
                    .last()
                    .unwrap_unchecked()
                    .registers
                    .lists
                    .get_unchecked(register_index)
            }
        };

        matches!(register, Register::Closed(_))
    }

    pub fn close_list_register(&mut self, register_index: usize) {
        let current_frame = self.current_frame_mut();

        current_frame.registers.lists.push(Register::Empty);

        let old_register = current_frame.registers.lists.swap_remove(register_index);

        if let Register::Value(value) = old_register {
            current_frame.registers.lists.push(Register::Closed(value));

            let _ = current_frame.registers.lists.swap_remove(register_index);
        } else {
            panic!("Attempted to close non-value register");
        }
    }

    pub fn get_constant(&self, constant_index: usize) -> &ConcreteValue {
        if cfg!(debug_assertions) {
            self.chunk.constants.get(constant_index).unwrap()
        } else {
            unsafe { self.chunk.constants.get_unchecked(constant_index) }
        }
    }
}

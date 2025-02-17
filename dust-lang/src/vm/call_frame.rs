use std::{
    cell::{RefCell, RefMut},
    fmt::{self, Debug, Display, Formatter},
    ops::{Add, Index, IndexMut},
    rc::Rc,
};

use smallvec::SmallVec;

use crate::{AbstractList, Chunk, DustString, Function};

#[derive(Debug)]
pub struct CallFrame {
    pub chunk: Rc<Chunk>,
    pub ip: usize,
    pub return_register: u16,
    pub registers: RegisterTable,
    pub constants: ConstantTable,
}

impl CallFrame {
    pub fn new(chunk: Rc<Chunk>, return_register: u16) -> Self {
        let registers = RegisterTable {
            booleans: RegisterList::new(chunk.boolean_register_count as usize),
            bytes: RegisterList::new(chunk.byte_register_count as usize),
            characters: RegisterList::new(chunk.character_register_count as usize),
            floats: RegisterList::new(chunk.float_register_count as usize),
            integers: RegisterList::new(chunk.integer_register_count as usize),
            strings: RegisterList::new(chunk.string_register_count as usize),
            lists: RegisterList::new(chunk.list_register_count as usize),
            functions: RegisterList::new(chunk.function_register_count as usize),
        };
        let constants = ConstantTable {
            characters: chunk
                .character_constants
                .iter()
                .map(|&character| RuntimeValue::Raw(character))
                .collect(),
            floats: chunk
                .float_constants
                .iter()
                .map(|&float| RuntimeValue::Raw(float))
                .collect(),
            integers: chunk
                .integer_constants
                .iter()
                .map(|&integer| RuntimeValue::Raw(integer))
                .collect(),
            strings: chunk
                .string_constants
                .iter()
                .map(|string| RuntimeValue::Raw(string.clone()))
                .collect(),
        };

        Self {
            chunk,
            ip: 0,
            return_register,
            registers,
            constants,
        }
    }

    pub fn get_boolean_from_register(&self, register_index: usize) -> &RuntimeValue<bool> {
        let register = self.registers.booleans.get(register_index);

        match register {
            Register::Value { value, .. } => value,
            Register::Pointer { pointer, .. } => self.get_boolean_from_pointer(pointer),
        }
    }

    pub fn get_boolean_from_pointer(&self, pointer: &Pointer) -> &RuntimeValue<bool> {
        match pointer {
            Pointer::Register(register_index) => self.get_boolean_from_register(*register_index),
            Pointer::Constant(_) => panic!("Attempted to get boolean from constant pointer"),
        }
    }

    pub fn get_byte_from_register(&self, register_index: usize) -> &RuntimeValue<u8> {
        let register = self.registers.bytes.get(register_index);

        match register {
            Register::Value { value, .. } => value,
            Register::Pointer { pointer, .. } => self.get_byte_from_pointer(pointer),
        }
    }

    pub fn get_byte_from_pointer(&self, pointer: &Pointer) -> &RuntimeValue<u8> {
        match pointer {
            Pointer::Register(register_index) => self.get_byte_from_register(*register_index),
            Pointer::Constant(_) => panic!("Attempted to get byte from constant pointer"),
        }
    }

    pub fn get_character_from_register(&self, register_index: usize) -> &RuntimeValue<char> {
        let register = self.registers.characters.get(register_index);

        match register {
            Register::Value { value, .. } => value,
            Register::Pointer { pointer, .. } => self.get_character_from_pointer(pointer),
        }
    }

    pub fn get_character_from_pointer(&self, pointer: &Pointer) -> &RuntimeValue<char> {
        match pointer {
            Pointer::Register(register_index) => self.get_character_from_register(*register_index),
            Pointer::Constant(constant_index) => self.get_character_constant(*constant_index),
        }
    }

    pub fn get_character_constant(&self, constant_index: usize) -> &RuntimeValue<char> {
        if cfg!(debug_assertions) {
            self.constants.characters.get(constant_index).unwrap()
        } else {
            unsafe { self.constants.characters.get_unchecked(constant_index) }
        }
    }

    pub fn get_float_from_register(&self, register_index: usize) -> &RuntimeValue<f64> {
        let register = self.registers.floats.get(register_index);

        match register {
            Register::Value { value, .. } => value,
            Register::Pointer { pointer, .. } => self.get_float_from_pointer(pointer),
        }
    }

    pub fn get_float_from_pointer(&self, pointer: &Pointer) -> &RuntimeValue<f64> {
        match pointer {
            Pointer::Register(register_index) => self.get_float_from_register(*register_index),
            Pointer::Constant(constant_index) => self.get_float_constant(*constant_index),
        }
    }

    pub fn get_float_constant(&self, constant_index: usize) -> &RuntimeValue<f64> {
        if cfg!(debug_assertions) {
            self.constants.floats.get(constant_index).unwrap()
        } else {
            unsafe { self.constants.floats.get_unchecked(constant_index) }
        }
    }

    pub fn get_integer_from_register(&self, register_index: usize) -> &RuntimeValue<i64> {
        let register = self.registers.integers.get(register_index);

        match register {
            Register::Value { value, .. } => value,
            Register::Pointer { pointer, .. } => self.get_integer_from_pointer(pointer),
        }
    }

    pub fn get_integer_from_register_mut(
        &mut self,
        register_index: usize,
    ) -> &mut RuntimeValue<i64> {
        let register = self.registers.integers.get_mut(register_index);

        match register {
            Register::Value { value, .. } => value,
            Register::Pointer { .. } => {
                panic!("Attempted to get mutable integer from pointer")
            }
        }
    }

    pub fn get_integer_from_pointer(&self, pointer: &Pointer) -> &RuntimeValue<i64> {
        match pointer {
            Pointer::Register(register_index) => self.get_integer_from_register(*register_index),
            Pointer::Constant(constant_index) => self.get_integer_constant(*constant_index),
        }
    }

    pub fn get_integer_from_pointer_mut(&mut self, pointer: &Pointer) -> &mut RuntimeValue<i64> {
        match pointer {
            Pointer::Register(register_index) => {
                self.get_integer_from_register_mut(*register_index)
            }
            Pointer::Constant(constant_index) => self.get_integer_constant_mut(*constant_index),
        }
    }

    pub fn get_integer_constant(&self, constant_index: usize) -> &RuntimeValue<i64> {
        if cfg!(debug_assertions) {
            self.constants.integers.get(constant_index).unwrap()
        } else {
            unsafe { self.constants.integers.get_unchecked(constant_index) }
        }
    }

    pub fn get_integer_constant_mut(&mut self, constant_index: usize) -> &mut RuntimeValue<i64> {
        if cfg!(debug_assertions) {
            self.constants.integers.get_mut(constant_index).unwrap()
        } else {
            unsafe { self.constants.integers.get_unchecked_mut(constant_index) }
        }
    }

    pub fn get_string_from_register(&self, register_index: usize) -> &RuntimeValue<DustString> {
        let register = self.registers.strings.get(register_index);

        match register {
            Register::Value { value, .. } => value,
            Register::Pointer { pointer, .. } => self.get_string_from_pointer(pointer),
        }
    }

    pub fn get_string_from_pointer(&self, pointer: &Pointer) -> &RuntimeValue<DustString> {
        match pointer {
            Pointer::Register(register_index) => self.get_string_from_register(*register_index),
            Pointer::Constant(constant_index) => self.get_string_constant(*constant_index),
        }
    }

    pub fn get_string_constant(&self, constant_index: usize) -> &RuntimeValue<DustString> {
        if cfg!(debug_assertions) {
            self.constants.strings.get(constant_index).unwrap()
        } else {
            unsafe { self.constants.strings.get_unchecked(constant_index) }
        }
    }

    pub fn get_string_constant_mut(
        &mut self,
        constant_index: usize,
    ) -> &mut RuntimeValue<DustString> {
        if cfg!(debug_assertions) {
            self.constants.strings.get_mut(constant_index).unwrap()
        } else {
            unsafe { self.constants.strings.get_unchecked_mut(constant_index) }
        }
    }
}

impl Display for CallFrame {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "FunctionCall: {} | IP: {}",
            self.chunk
                .name
                .as_ref()
                .unwrap_or(&DustString::from("anonymous")),
            self.ip,
        )
    }
}

#[derive(Debug, Default)]
pub struct ConstantTable {
    pub characters: Vec<RuntimeValue<char>>,
    pub floats: Vec<RuntimeValue<f64>>,
    pub integers: Vec<RuntimeValue<i64>>,
    pub strings: Vec<RuntimeValue<DustString>>,
}

#[derive(Debug)]
pub struct RegisterTable {
    pub booleans: RegisterList<bool>,
    pub bytes: RegisterList<u8>,
    pub characters: RegisterList<char>,
    pub floats: RegisterList<f64>,
    pub integers: RegisterList<i64>,
    pub strings: RegisterList<DustString>,
    pub lists: RegisterList<AbstractList>,
    pub functions: RegisterList<Function>,
}

#[derive(Debug)]
pub struct RegisterList<T, const STACK_LEN: usize = 64> {
    pub registers: SmallVec<[Register<T>; STACK_LEN]>,
}

impl<T, const STACK_LEN: usize> RegisterList<T, STACK_LEN>
where
    T: Clone + Default,
{
    pub fn new(length: usize) -> Self {
        let mut registers = SmallVec::with_capacity(length);

        for _ in 0..length {
            registers.push(Register::default());
        }

        Self { registers }
    }

    pub fn get(&self, index: usize) -> &Register<T> {
        if cfg!(debug_assertions) {
            self.registers.get(index).unwrap()
        } else {
            unsafe { self.registers.get_unchecked(index) }
        }
    }

    pub fn get_mut(&mut self, index: usize) -> &mut Register<T> {
        if cfg!(debug_assertions) {
            let length = self.registers.len();

            self.registers
                .get_mut(index)
                .unwrap_or_else(|| panic!("Index out of bounds: {index}. Length is {length}"))
        } else {
            unsafe { self.registers.get_unchecked_mut(index) }
        }
    }

    pub fn close(&mut self, index: usize) {
        if cfg!(debug_assertions) {
            self.registers.get_mut(index).unwrap().close()
        } else {
            unsafe { self.registers.get_unchecked_mut(index).close() }
        }
    }

    pub fn is_closed(&self, index: usize) -> bool {
        if cfg!(debug_assertions) {
            self.registers.get(index).unwrap().is_closed()
        } else {
            unsafe { self.registers.get_unchecked(index).is_closed() }
        }
    }
}

impl<T> Index<usize> for RegisterList<T> {
    type Output = Register<T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.registers[index]
    }
}

impl<T> IndexMut<usize> for RegisterList<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.registers[index]
    }
}

#[derive(Clone, Debug)]
pub enum Register<T> {
    Value {
        value: RuntimeValue<T>,
        is_closed: bool,
    },
    Pointer {
        pointer: Pointer,
        is_closed: bool,
    },
}

impl<T> Register<T> {
    pub fn is_closed(&self) -> bool {
        match self {
            Self::Value { is_closed, .. } => *is_closed,
            Self::Pointer { is_closed, .. } => *is_closed,
        }
    }

    pub fn close(&mut self) {
        match self {
            Self::Value { is_closed, .. } => *is_closed = true,
            Self::Pointer { is_closed, .. } => *is_closed = true,
        }
    }

    pub fn set(&mut self, new_value: RuntimeValue<T>) {
        match self {
            Self::Value {
                value: old_value, ..
            } => *old_value = new_value,
            Self::Pointer { is_closed, .. } => {
                *self = Self::Value {
                    value: new_value,
                    is_closed: *is_closed,
                }
            }
        }
    }

    pub fn as_value(&self) -> &RuntimeValue<T> {
        match self {
            Self::Value { value, .. } => value,
            Self::Pointer { .. } => panic!("Attempted to use pointer as value"),
        }
    }

    pub fn as_value_mut(&mut self) -> &mut RuntimeValue<T> {
        match self {
            Self::Value { value, .. } => value,
            Self::Pointer { .. } => panic!("Attempted to use pointer as value"),
        }
    }
}

impl<T: Default> Default for Register<T> {
    fn default() -> Self {
        Self::Value {
            value: RuntimeValue::Raw(Default::default()),
            is_closed: false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum RuntimeValue<T> {
    Raw(T),
    Rc(Rc<T>),
    RefCell(Rc<RefCell<T>>),
}

impl<T: Clone> RuntimeValue<T> {
    pub fn ref_cell(value: T) -> Self {
        Self::RefCell(Rc::new(RefCell::new(value)))
    }

    pub fn rc(value: T) -> Self {
        Self::Rc(Rc::new(value))
    }

    pub fn to_ref_cell(&mut self) -> Self {
        match self {
            Self::Raw(value) => RuntimeValue::ref_cell(value.clone()),
            Self::Rc(value) => RuntimeValue::ref_cell(value.as_ref().clone()),
            Self::RefCell(_) => self.clone(),
        }
    }

    pub fn to_rc(&mut self) -> Self {
        match self {
            Self::Raw(value) => RuntimeValue::rc(value.clone()),
            Self::Rc(_) => self.clone(),
            Self::RefCell(value) => RuntimeValue::rc(value.borrow().clone()),
        }
    }

    pub fn set_inner(&mut self, new_value: T) {
        match self {
            Self::Raw(value) => *value = new_value,
            Self::Rc(value) => {
                if let Some(mutable) = Rc::get_mut(value) {
                    *mutable = new_value;
                }
            }
            Self::RefCell(value) => {
                let _ = value.replace(new_value);
            }
        }
    }

    pub fn clone_inner(&self) -> T {
        match self {
            Self::Raw(value) => value.clone(),
            Self::Rc(value) => value.as_ref().clone(),
            Self::RefCell(value) => value.borrow().clone(),
        }
    }

    pub fn borrow_mut(&self) -> RefMut<T> {
        match self {
            Self::RefCell(value) => value.borrow_mut(),
            _ => panic!("Attempted to borrow mutable reference from immutable register value"),
        }
    }
}

impl<T: Add<Output = T> + Copy> Add for &RuntimeValue<T> {
    type Output = T;

    fn add(self, other: Self) -> Self::Output {
        match (self, other) {
            (RuntimeValue::Raw(left), RuntimeValue::Raw(right)) => *left + *right,
            (RuntimeValue::Raw(left), RuntimeValue::Rc(right)) => *left + **right,
            (RuntimeValue::Raw(left), RuntimeValue::RefCell(right)) => {
                let right = right.borrow();

                *left + *right
            }
            (RuntimeValue::Rc(left), RuntimeValue::Raw(right)) => **left + *right,
            (RuntimeValue::Rc(left), RuntimeValue::Rc(right)) => **left + **right,
            (RuntimeValue::Rc(left), RuntimeValue::RefCell(right)) => {
                let right = right.borrow();

                **left + *right
            }
            (RuntimeValue::RefCell(left), RuntimeValue::RefCell(right)) => {
                let left = left.borrow();
                let right = right.borrow();

                *left + *right
            }
            (RuntimeValue::RefCell(left), RuntimeValue::Raw(right)) => {
                let left = left.borrow();

                *left + *right
            }
            (RuntimeValue::RefCell(left), RuntimeValue::Rc(right)) => {
                let left = left.borrow();

                *left + **right
            }
        }
    }
}

impl<T: PartialEq> PartialEq for RuntimeValue<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RuntimeValue::Raw(left), RuntimeValue::Raw(right)) => left == right,
            (RuntimeValue::Raw(left), RuntimeValue::Rc(right)) => left == &**right,
            (RuntimeValue::Raw(left), RuntimeValue::RefCell(right)) => {
                let right = right.borrow();

                left == &*right
            }
            (RuntimeValue::Rc(left), RuntimeValue::Raw(right)) => **left == *right,
            (RuntimeValue::Rc(left), RuntimeValue::Rc(right)) => **left == **right,
            (RuntimeValue::Rc(left), RuntimeValue::RefCell(right)) => {
                let right = right.borrow();

                **left == *right
            }
            (RuntimeValue::RefCell(left), RuntimeValue::RefCell(right)) => {
                let left = left.borrow();
                let right = right.borrow();

                *left == *right
            }
            (RuntimeValue::RefCell(left), RuntimeValue::Raw(right)) => {
                let left = left.borrow();

                *left == *right
            }
            (RuntimeValue::RefCell(left), RuntimeValue::Rc(right)) => {
                let left = left.borrow();

                *left == **right
            }
        }
    }
}

impl<T: PartialOrd> PartialOrd for RuntimeValue<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (RuntimeValue::Raw(left), RuntimeValue::Raw(right)) => left.partial_cmp(right),
            (RuntimeValue::Raw(left), RuntimeValue::Rc(right)) => left.partial_cmp(&**right),
            (RuntimeValue::Raw(left), RuntimeValue::RefCell(right)) => {
                let right = right.borrow();

                left.partial_cmp(&*right)
            }
            (RuntimeValue::Rc(left), RuntimeValue::Raw(right)) => (**left).partial_cmp(right),
            (RuntimeValue::Rc(left), RuntimeValue::Rc(right)) => left.partial_cmp(right),
            (RuntimeValue::Rc(left), RuntimeValue::RefCell(right)) => {
                let right = right.borrow();

                (**left).partial_cmp(&right)
            }
            (RuntimeValue::RefCell(left), RuntimeValue::RefCell(right)) => {
                let left = left.borrow();
                let right = right.borrow();

                left.partial_cmp(&*right)
            }
            (RuntimeValue::RefCell(left), RuntimeValue::Raw(right)) => {
                let left = left.borrow();

                left.partial_cmp(right)
            }
            (RuntimeValue::RefCell(left), RuntimeValue::Rc(right)) => {
                let left = left.borrow();

                left.partial_cmp(&**right)
            }
        }
    }
}

impl<T: Display> Display for RuntimeValue<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Raw(value) => write!(f, "{}", value),
            Self::Rc(value) => write!(f, "{}", value),
            Self::RefCell(value) => write!(f, "{}", value.borrow()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Pointer {
    Register(usize),
    Constant(usize),
}

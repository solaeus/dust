use std::fmt::{self, Display, Formatter};

#[repr(C)]
pub struct Integer32Register(pub i32);

#[repr(C)]
pub struct Integer64Register(pub i64);

#[repr(C)]
pub struct Float64Register(pub f64);

#[repr(C)]
pub struct PointerRegister(pub *mut u8);

#[derive(Clone, Copy, Debug)]
pub enum RegisterClass {
    Integer32,
    Integer64,
    Float64,
    Pointer,
}

impl RegisterClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            RegisterClass::Integer32 => "i32",
            RegisterClass::Integer64 => "i64",
            RegisterClass::Float64 => "f64",
            RegisterClass::Pointer => "ptr",
        }
    }
}

impl Display for RegisterClass {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

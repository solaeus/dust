use std::fmt::{self, Display, Formatter};

#[repr(C)]
pub struct Integer32Register(pub i32);

#[repr(C)]
pub struct Integer64Register(pub i64);

#[repr(C)]
pub struct Float64Register(pub f32);

#[repr(C)]
pub struct PointerRegister(pub *mut u8);

#[derive(Clone, Copy, Debug)]
pub enum RegisterClass {
    Integer32,
    Integer64,
    Float64,
    Pointer,
}

impl Display for RegisterClass {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            RegisterClass::Integer32 => write!(f, "i32"),
            RegisterClass::Integer64 => write!(f, "i64"),
            RegisterClass::Float64 => write!(f, "f64"),
            RegisterClass::Pointer => write!(f, "ptr"),
        }
    }
}

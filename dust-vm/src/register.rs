#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Register(u32);

impl Register {
    pub fn new(value: impl RegisterValue) -> Self {
        Self(value.to_u32())
    }

    pub fn as_value<T: RegisterValue>(&self) -> T {
        T::from_u32(self.0)
    }

    pub fn as_bits(&self) -> u32 {
        self.0
    }
}

trait RegisterValue {
    fn from_u32(value: u32) -> Self;
    fn to_u32(self) -> u32;
}

macro_rules! impl_register_value_for_integer {
    ($($t:ty),+) => {
        $(
            impl RegisterValue for $t {
                fn from_u32(value: u32) -> Self {
                    value as $t
                }

                fn to_u32(self) -> u32 {
                    self as u32
                }
            }
        )+
    };
}

impl_register_value_for_integer!(u8, u16, u32, i8, i16, i32);

impl RegisterValue for bool {
    fn from_u32(value: u32) -> Self {
        value != 0
    }

    fn to_u32(self) -> u32 {
        self as u32
    }
}

impl RegisterValue for f32 {
    fn from_u32(value: u32) -> Self {
        f32::from_bits(value)
    }

    fn to_u32(self) -> u32 {
        self.to_bits()
    }
}

impl RegisterValue for char {
    fn from_u32(value: u32) -> Self {
        char::from_u32(value).unwrap_or('\0')
    }

    fn to_u32(self) -> u32 {
        self as u32
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RegisterTag(pub u8);

impl RegisterTag {
    pub const EMPTY: RegisterTag = RegisterTag(0);
    pub const SCALAR: RegisterTag = RegisterTag(1);
    pub const OBJECT: RegisterTag = RegisterTag(2);
}

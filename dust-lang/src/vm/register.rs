#[derive(Clone, Copy)]
#[repr(C)]
pub struct RegisterIndices {
    pub start: u32,
    pub end: u32,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Register(pub(super) u32);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RegisterTag(pub u8);

impl RegisterTag {
    pub const EMPTY: RegisterTag = RegisterTag(0);
    pub const SCALAR: RegisterTag = RegisterTag(1);
    pub const OBJECT: RegisterTag = RegisterTag(2);
}

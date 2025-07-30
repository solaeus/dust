#[derive(Clone, Copy)]
#[repr(C)]
pub union Register {
    pub empty: (),
    pub boolean: bool,
    pub byte: u8,
    pub char: char,
    pub float: f64,
    pub integer: i64,
    pub prototype_index: usize,
    pub object_key: usize,
}

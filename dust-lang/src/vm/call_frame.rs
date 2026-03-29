#[derive(Clone, Copy, Debug)]
pub struct CallFrame {
    pub prototype_id: u16,
    pub regsiter_range_start: u16,
    pub register_range_end: u16,
    pub argument_count: u16,
    pub return_count: u16,

    pub instruction_pointer: usize,
}

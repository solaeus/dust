pub mod sizes {
    pub const IP_FIELD: usize = 0;
    pub const FUNCTION_INDEX_FIELD: usize = IP_FIELD + size_of::<usize>();
    pub const CALL_INSTRUCTION_FIELD: usize = FUNCTION_INDEX_FIELD + size_of::<usize>();
    pub const REGISTER_RANGE_START_FIELD: usize = CALL_INSTRUCTION_FIELD + size_of::<usize>();
    pub const REGISTER_RANGE_END_FIELD: usize = REGISTER_RANGE_START_FIELD + size_of::<usize>();

    pub const CALL_FRAME_SIZE: usize = REGISTER_RANGE_END_FIELD + size_of::<usize>();
}

use sizes::*;

use cranelift::{
    codegen::ir::{MemFlags, Value, types},
    prelude::{FunctionBuilder, InstBuilder},
};

pub fn new_call_stack(length: usize) -> Vec<u8> {
    vec![0; length * CALL_FRAME_SIZE]
}

#[expect(clippy::too_many_arguments)]
pub fn new_call_frame(
    frame_index: Value,
    ip: Value,
    function_index: Value,
    call_instruction: Value,
    register_range_start: Value,
    register_range_end: Value,
    call_stack_ptr: Value,
    builder: &mut FunctionBuilder,
) {
    let frame_size_value = builder.ins().iconst(types::I64, CALL_FRAME_SIZE as i64);

    let instruction_pointer_field_value = builder.ins().iconst(types::I64, IP_FIELD as i64);
    let instruction_pointer_multiplication = builder.ins().imul(frame_index, frame_size_value);
    let instruction_pointer_offset = builder.ins().iadd(
        instruction_pointer_multiplication,
        instruction_pointer_field_value,
    );

    let function_index_field_value = builder
        .ins()
        .iconst(types::I64, FUNCTION_INDEX_FIELD as i64);
    let function_index_multiplication = builder.ins().imul(frame_index, frame_size_value);
    let function_index_offset = builder
        .ins()
        .iadd(function_index_multiplication, function_index_field_value);

    let call_instruction_field_value = builder
        .ins()
        .iconst(types::I64, CALL_INSTRUCTION_FIELD as i64);
    let call_instruction_multiplication = builder.ins().imul(frame_index, frame_size_value);
    let call_instruction_offset = builder.ins().iadd(
        call_instruction_multiplication,
        call_instruction_field_value,
    );

    let register_range_start_field_value = builder
        .ins()
        .iconst(types::I64, REGISTER_RANGE_START_FIELD as i64);
    let register_range_start_multiplication = builder.ins().imul(frame_index, frame_size_value);
    let register_range_start_offset = builder.ins().iadd(
        register_range_start_multiplication,
        register_range_start_field_value,
    );

    let register_range_end_field_value = builder
        .ins()
        .iconst(types::I64, REGISTER_RANGE_END_FIELD as i64);
    let register_range_end_multiplication = builder.ins().imul(frame_index, frame_size_value);
    let register_range_end_offset = builder.ins().iadd(
        register_range_end_multiplication,
        register_range_end_field_value,
    );

    let instruction_pointer_address = builder
        .ins()
        .iadd(call_stack_ptr, instruction_pointer_offset);
    let function_index_address = builder.ins().iadd(call_stack_ptr, function_index_offset);
    let call_instruction_address = builder.ins().iadd(call_stack_ptr, call_instruction_offset);
    let register_range_start_address = builder
        .ins()
        .iadd(call_stack_ptr, register_range_start_offset);
    let register_range_end_address = builder
        .ins()
        .iadd(call_stack_ptr, register_range_end_offset);

    builder
        .ins()
        .store(MemFlags::new(), ip, instruction_pointer_address, 0);
    builder
        .ins()
        .store(MemFlags::new(), function_index, function_index_address, 0);
    builder.ins().store(
        MemFlags::new(),
        call_instruction,
        call_instruction_address,
        0,
    );
    builder.ins().store(
        MemFlags::new(),
        register_range_start,
        register_range_start_address,
        0,
    );
    builder.ins().store(
        MemFlags::new(),
        register_range_end,
        register_range_end_address,
        0,
    );
}

pub fn get_frame_ip(
    frame_index: Value,
    call_stack_ptr: Value,
    builder: &mut FunctionBuilder,
) -> Value {
    let frame_size_value = builder.ins().iconst(types::I64, CALL_FRAME_SIZE as i64);
    let field_offset = builder.ins().iconst(types::I64, IP_FIELD as i64);
    let index_offset = builder.ins().imul(frame_index, frame_size_value);
    let total = builder.ins().iadd(index_offset, field_offset);
    let address = builder.ins().iadd(call_stack_ptr, total);

    builder.ins().load(types::I64, MemFlags::new(), address, 0)
}

pub fn _get_frame_function_index(
    frame_index: Value,
    call_stack_ptr: Value,
    builder: &mut FunctionBuilder,
) -> Value {
    let frame_size_value = builder.ins().iconst(types::I64, CALL_FRAME_SIZE as i64);
    let field_offset = builder
        .ins()
        .iconst(types::I64, FUNCTION_INDEX_FIELD as i64);
    let index_offset = builder.ins().imul(frame_index, frame_size_value);
    let total_offset = builder.ins().iadd(index_offset, field_offset);
    let address = builder.ins().iadd(call_stack_ptr, total_offset);

    builder.ins().load(types::I64, MemFlags::new(), address, 0)
}

pub fn _get_call_instruction(
    frame_index: Value,
    call_stack_ptr: Value,
    builder: &mut FunctionBuilder,
) -> Value {
    let frame_size_value = builder.ins().iconst(types::I64, CALL_FRAME_SIZE as i64);
    let field_offset = builder
        .ins()
        .iconst(types::I64, CALL_INSTRUCTION_FIELD as i64);
    let index_offset = builder.ins().imul(frame_index, frame_size_value);
    let total_offset = builder.ins().iadd(index_offset, field_offset);
    let address = builder.ins().iadd(call_stack_ptr, total_offset);

    builder.ins().load(types::I64, MemFlags::new(), address, 0)
}

pub fn _get_frame_register_range_start(
    frame_index: Value,
    call_stack_ptr: Value,
    builder: &mut FunctionBuilder,
) -> Value {
    let frame_size_value = builder.ins().iconst(types::I64, CALL_FRAME_SIZE as i64);
    let field_offset = builder
        .ins()
        .iconst(types::I64, REGISTER_RANGE_START_FIELD as i64);
    let index_offset = builder.ins().imul(frame_index, frame_size_value);
    let total_offset = builder.ins().iadd(index_offset, field_offset);
    let address = builder.ins().iadd(call_stack_ptr, total_offset);

    builder.ins().load(types::I64, MemFlags::new(), address, 0)
}

pub fn _get_frame_register_range_end(
    frame_index: Value,
    call_stack_ptr: Value,
    builder: &mut FunctionBuilder,
) -> Value {
    let frame_size_value = builder.ins().iconst(types::I64, CALL_FRAME_SIZE as i64);
    let field_offset = builder
        .ins()
        .iconst(types::I64, REGISTER_RANGE_END_FIELD as i64);
    let index_offset = builder.ins().imul(frame_index, frame_size_value);
    let total_offset = builder.ins().iadd(index_offset, field_offset);
    let address = builder.ins().iadd(call_stack_ptr, total_offset);

    builder.ins().load(types::I64, MemFlags::new(), address, 0)
}

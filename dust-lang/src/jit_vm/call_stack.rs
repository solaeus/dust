pub mod sizes {
    pub const IP_FIELD: usize = 0;
    pub const FUNCTION_INDEX_FIELD: usize = IP_FIELD + size_of::<usize>();
    pub const REGISTER_RANGE_START_FIELD: usize = FUNCTION_INDEX_FIELD + size_of::<usize>();
    pub const REGISTER_RANGE_END_FIELD: usize = REGISTER_RANGE_START_FIELD + size_of::<usize>();
    pub const ARGUMENTS_INDEX_FIELD: usize = REGISTER_RANGE_END_FIELD + size_of::<usize>();

    pub const CALL_FRAME_SIZE: usize = ARGUMENTS_INDEX_FIELD + size_of::<usize>();
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
pub fn push_call_frame(
    frame_index: Value,
    ip: Value,
    function_index: Value,
    register_range_start: Value,
    register_range_end: Value,
    arguments_index: Value,
    call_stack_pointer: Value,
    call_stack_length_pointer: Value,
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

    let arguments_index_field_value = builder
        .ins()
        .iconst(types::I64, ARGUMENTS_INDEX_FIELD as i64);
    let arguments_index_multiplication = builder.ins().imul(frame_index, frame_size_value);
    let arguments_index_offset = builder
        .ins()
        .iadd(arguments_index_multiplication, arguments_index_field_value);

    let instruction_pointer_address = builder
        .ins()
        .iadd(call_stack_pointer, instruction_pointer_offset);
    let function_index_address = builder
        .ins()
        .iadd(call_stack_pointer, function_index_offset);
    let register_range_start_address = builder
        .ins()
        .iadd(call_stack_pointer, register_range_start_offset);
    let register_range_end_address = builder
        .ins()
        .iadd(call_stack_pointer, register_range_end_offset);
    let arguments_index_address = builder
        .ins()
        .iadd(call_stack_pointer, arguments_index_offset);

    builder
        .ins()
        .store(MemFlags::new(), ip, instruction_pointer_address, 0);
    builder
        .ins()
        .store(MemFlags::new(), function_index, function_index_address, 0);
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
    builder
        .ins()
        .store(MemFlags::new(), arguments_index, arguments_index_address, 0);

    let call_stack_length =
        builder
            .ins()
            .load(types::I64, MemFlags::new(), call_stack_length_pointer, 0);
    let one = builder.ins().iconst(types::I64, 1);
    let new_length = builder.ins().iadd(call_stack_length, one);

    builder
        .ins()
        .store(MemFlags::new(), new_length, call_stack_length_pointer, 0);
}

pub fn _get_frame_ip(
    frame_index: Value,
    call_stack_pointer: Value,
    builder: &mut FunctionBuilder,
) -> Value {
    let frame_size_value = builder.ins().iconst(types::I64, CALL_FRAME_SIZE as i64);
    let field_offset = builder.ins().iconst(types::I64, IP_FIELD as i64);
    let index_offset = builder.ins().imul(frame_index, frame_size_value);
    let total = builder.ins().iadd(index_offset, field_offset);
    let address = builder.ins().iadd(call_stack_pointer, total);

    builder.ins().load(types::I64, MemFlags::new(), address, 0)
}

pub fn get_frame_function_index(
    frame_index: Value,
    call_stack_pointer: Value,
    builder: &mut FunctionBuilder,
) -> Value {
    let frame_size_value = builder.ins().iconst(types::I64, CALL_FRAME_SIZE as i64);
    let field_offset = builder
        .ins()
        .iconst(types::I64, FUNCTION_INDEX_FIELD as i64);
    let index_offset = builder.ins().imul(frame_index, frame_size_value);
    let total_offset = builder.ins().iadd(index_offset, field_offset);
    let address = builder.ins().iadd(call_stack_pointer, total_offset);

    builder.ins().load(types::I64, MemFlags::new(), address, 0)
}

pub fn _get_frame_register_range_start(
    frame_index: Value,
    call_stack_pointer: Value,
    builder: &mut FunctionBuilder,
) -> Value {
    let frame_size_value = builder.ins().iconst(types::I64, CALL_FRAME_SIZE as i64);
    let field_offset = builder
        .ins()
        .iconst(types::I64, REGISTER_RANGE_START_FIELD as i64);
    let index_offset = builder.ins().imul(frame_index, frame_size_value);
    let total_offset = builder.ins().iadd(index_offset, field_offset);
    let address = builder.ins().iadd(call_stack_pointer, total_offset);

    builder.ins().load(types::I64, MemFlags::new(), address, 0)
}

pub fn _get_frame_register_range_end(
    frame_index: Value,
    call_stack_pointer: Value,
    builder: &mut FunctionBuilder,
) -> Value {
    let frame_size_value = builder.ins().iconst(types::I64, CALL_FRAME_SIZE as i64);
    let field_offset = builder
        .ins()
        .iconst(types::I64, REGISTER_RANGE_END_FIELD as i64);
    let index_offset = builder.ins().imul(frame_index, frame_size_value);
    let total_offset = builder.ins().iadd(index_offset, field_offset);
    let address = builder.ins().iadd(call_stack_pointer, total_offset);

    builder.ins().load(types::I64, MemFlags::new(), address, 0)
}

pub fn _get_frame_arguments_index(
    frame_index: Value,
    call_stack_pointer: Value,
    builder: &mut FunctionBuilder,
) -> Value {
    let frame_size_value = builder.ins().iconst(types::I64, CALL_FRAME_SIZE as i64);
    let field_offset = builder
        .ins()
        .iconst(types::I64, ARGUMENTS_INDEX_FIELD as i64);
    let index_offset = builder.ins().imul(frame_index, frame_size_value);
    let total_offset = builder.ins().iadd(index_offset, field_offset);
    let address = builder.ins().iadd(call_stack_pointer, total_offset);

    builder.ins().load(types::I64, MemFlags::new(), address, 0)
}

pub fn get_call_frame(
    frame_index: Value,
    call_stack_pointer: Value,
    builder: &mut FunctionBuilder,
) -> (Value, Value, Value, Value, Value) {
    let frame_size_value = builder.ins().iconst(types::I64, CALL_FRAME_SIZE as i64);

    let instruction_pointer_field_offset = builder.ins().iconst(types::I64, IP_FIELD as i64);
    let instruction_pointer_index_offset = builder.ins().imul(frame_index, frame_size_value);
    let instruction_pointer_total_offset = builder.ins().iadd(
        instruction_pointer_index_offset,
        instruction_pointer_field_offset,
    );
    let instruction_pointer_address = builder
        .ins()
        .iadd(call_stack_pointer, instruction_pointer_total_offset);
    let ip = builder
        .ins()
        .load(types::I64, MemFlags::new(), instruction_pointer_address, 0);

    let function_index_field_offset = builder
        .ins()
        .iconst(types::I64, FUNCTION_INDEX_FIELD as i64);
    let function_index_index_offset = builder.ins().imul(frame_index, frame_size_value);
    let function_index_total_offset = builder
        .ins()
        .iadd(function_index_index_offset, function_index_field_offset);
    let function_index_address = builder
        .ins()
        .iadd(call_stack_pointer, function_index_total_offset);
    let function_index = builder
        .ins()
        .load(types::I64, MemFlags::new(), function_index_address, 0);

    let register_range_start_field_offset = builder
        .ins()
        .iconst(types::I64, REGISTER_RANGE_START_FIELD as i64);
    let register_range_start_index_offset = builder.ins().imul(frame_index, frame_size_value);
    let register_range_start_total_offset = builder.ins().iadd(
        register_range_start_index_offset,
        register_range_start_field_offset,
    );
    let register_range_start_address = builder
        .ins()
        .iadd(call_stack_pointer, register_range_start_total_offset);
    let register_range_start =
        builder
            .ins()
            .load(types::I64, MemFlags::new(), register_range_start_address, 0);

    let register_range_end_field_offset = builder
        .ins()
        .iconst(types::I64, REGISTER_RANGE_END_FIELD as i64);
    let register_range_end_index_offset = builder.ins().imul(frame_index, frame_size_value);
    let register_range_end_total_offset = builder.ins().iadd(
        register_range_end_index_offset,
        register_range_end_field_offset,
    );
    let register_range_end_address = builder
        .ins()
        .iadd(call_stack_pointer, register_range_end_total_offset);
    let register_range_end =
        builder
            .ins()
            .load(types::I64, MemFlags::new(), register_range_end_address, 0);

    let arguments_index_field_offset = builder
        .ins()
        .iconst(types::I64, ARGUMENTS_INDEX_FIELD as i64);
    let arguments_index_index_offset = builder.ins().imul(frame_index, frame_size_value);
    let arguments_index_total_offset = builder
        .ins()
        .iadd(arguments_index_index_offset, arguments_index_field_offset);
    let arguments_index_address = builder
        .ins()
        .iadd(call_stack_pointer, arguments_index_total_offset);
    let arguments_index =
        builder
            .ins()
            .load(types::I64, MemFlags::new(), arguments_index_address, 0);

    (
        ip,
        function_index,
        register_range_start,
        register_range_end,
        arguments_index,
    )
}

pub fn pop_call_frame(
    call_stack_pointer: Value,
    call_stack_length_pointer: Value,
    builder: &mut FunctionBuilder,
) -> (Value, Value, Value, Value, Value) {
    let one = builder.ins().iconst(types::I64, 1);
    let call_stack_length =
        builder
            .ins()
            .load(types::I64, MemFlags::new(), call_stack_length_pointer, 0);
    let top_call_frame_index = builder.ins().isub(call_stack_length, one);
    let new_length = builder.ins().isub(call_stack_length, one);

    builder
        .ins()
        .store(MemFlags::new(), new_length, call_stack_length_pointer, 0);
    get_call_frame(top_call_frame_index, call_stack_pointer, builder)
}

use std::{array, mem::offset_of};

use cranelift::{
    codegen::{CodegenError, ir::FuncRef},
    frontend::Switch,
    prelude::{
        AbiParam, FloatCC, FunctionBuilder, FunctionBuilderContext, InstBuilder, IntCC, MemFlags,
        Signature, Value as CraneliftValue, Variable,
        types::{F64, I8, I64},
    },
};
use cranelift_module::{FuncId, Module, ModuleError};
use smallvec::SmallVec;
use tracing::{Level, info, span};

use crate::{
    constant_table::ConstantTable,
    instruction::{
        Address, Call, CallNative, Drop, GetList, Jump, MemoryKind, Move, Negate, NewList,
        OperandType, Operation, Return, SetList, Test, ToString,
    },
    jit_vm::{
        JitCompiler, JitError, Register, RegisterTag, ThreadStatus,
        call_stack::get_call_frame,
        jit_compiler::{FunctionIds, HotRegisters},
        thread::ThreadContext,
    },
    prototype::Prototype,
    r#type::Type,
};

pub fn compile_stackless_function(
    function_id: FuncId,
    prototype: &Prototype,
    is_main: bool,
    compiler: &mut JitCompiler,
) -> Result<(), JitError> {
    let span = span!(Level::INFO, "stackless");
    let _enter = span.enter();

    let mut function_builder_context = FunctionBuilderContext::new();
    let mut compilation_context = compiler.module.make_context();
    let pointer_type = compiler.module.isa().pointer_type();

    compilation_context
        .func
        .signature
        .params
        .push(AbiParam::new(pointer_type));

    let mut function_builder =
        FunctionBuilder::new(&mut compilation_context.func, &mut function_builder_context);

    let allocate_list_function = {
        let mut allocate_list_signature = Signature::new(compiler.module.isa().default_call_conv());

        allocate_list_signature.params.extend([
            AbiParam::new(I8),
            AbiParam::new(I64),
            AbiParam::new(pointer_type),
        ]);
        allocate_list_signature.returns.push(AbiParam::new(I64));

        compiler.declare_imported_function(
            &mut function_builder,
            "allocate_list",
            allocate_list_signature,
        )?
    };

    let instert_into_list_function = {
        let mut insert_into_list_signature =
            Signature::new(compiler.module.isa().default_call_conv());

        insert_into_list_signature.params.extend([
            AbiParam::new(I64),
            AbiParam::new(I64),
            AbiParam::new(I64),
        ]);
        insert_into_list_signature.returns = vec![];

        compiler.declare_imported_function(
            &mut function_builder,
            "insert_into_list",
            insert_into_list_signature,
        )?
    };

    let get_from_list_function = {
        let mut get_from_list_signature = Signature::new(compiler.module.isa().default_call_conv());

        get_from_list_signature.params.extend([
            AbiParam::new(I64),
            AbiParam::new(I64),
            AbiParam::new(pointer_type),
        ]);
        get_from_list_signature.returns.push(AbiParam::new(I64));

        compiler.declare_imported_function(
            &mut function_builder,
            "get_from_list",
            get_from_list_signature,
        )?
    };

    let allocate_string_function = {
        let mut allocate_string_signature =
            Signature::new(compiler.module.isa().default_call_conv());

        allocate_string_signature.params.extend([
            AbiParam::new(pointer_type),
            AbiParam::new(I64),
            AbiParam::new(pointer_type),
        ]);
        allocate_string_signature.returns.push(AbiParam::new(I64)); // return value

        compiler.declare_imported_function(
            &mut function_builder,
            "allocate_string",
            allocate_string_signature,
        )?
    };

    let concatenate_strings_function = {
        let mut concatenate_strings_signature =
            Signature::new(compiler.module.isa().default_call_conv());

        concatenate_strings_signature.params.extend([
            AbiParam::new(pointer_type),
            AbiParam::new(pointer_type),
            AbiParam::new(pointer_type),
        ]);
        concatenate_strings_signature
            .returns
            .push(AbiParam::new(I64));

        compiler.declare_imported_function(
            &mut function_builder,
            "concatenate_strings",
            concatenate_strings_signature,
        )?
    };

    let concatenate_character_string_function = {
        let mut concatenate_character_string_signature =
            Signature::new(compiler.module.isa().default_call_conv());

        concatenate_character_string_signature.params.extend([
            AbiParam::new(I64),
            AbiParam::new(pointer_type),
            AbiParam::new(pointer_type),
        ]);
        concatenate_character_string_signature
            .returns
            .push(AbiParam::new(I64));

        compiler.declare_imported_function(
            &mut function_builder,
            "concatenate_character_string",
            concatenate_character_string_signature,
        )?
    };

    let concatenate_string_character_function = {
        let mut concatenate_string_character_signature =
            Signature::new(compiler.module.isa().default_call_conv());

        concatenate_string_character_signature.params.extend([
            AbiParam::new(pointer_type),
            AbiParam::new(I64),
            AbiParam::new(pointer_type),
        ]);
        concatenate_string_character_signature
            .returns
            .push(AbiParam::new(I64));

        compiler.declare_imported_function(
            &mut function_builder,
            "concatenate_string_character",
            concatenate_string_character_signature,
        )?
    };

    let concatenate_characters_function = {
        let mut concatenate_characters_signature =
            Signature::new(compiler.module.isa().default_call_conv());

        concatenate_characters_signature.params.extend([
            AbiParam::new(I64),
            AbiParam::new(I64),
            AbiParam::new(pointer_type),
        ]);
        concatenate_characters_signature
            .returns
            .push(AbiParam::new(I64));

        compiler.declare_imported_function(
            &mut function_builder,
            "concatenate_characters",
            concatenate_characters_signature,
        )?
    };

    let compare_strings_equal_function = {
        let mut compare_strings_equal_signature =
            Signature::new(compiler.module.isa().default_call_conv());

        compare_strings_equal_signature
            .params
            .extend([AbiParam::new(pointer_type), AbiParam::new(pointer_type)]);
        compare_strings_equal_signature
            .returns
            .push(AbiParam::new(I8));

        compiler.declare_imported_function(
            &mut function_builder,
            "compare_strings_equal",
            compare_strings_equal_signature,
        )?
    };

    let compare_strings_less_than_function = {
        let mut compare_strings_less_than_signature =
            Signature::new(compiler.module.isa().default_call_conv());

        compare_strings_less_than_signature
            .params
            .extend([AbiParam::new(pointer_type), AbiParam::new(pointer_type)]);
        compare_strings_less_than_signature
            .returns
            .push(AbiParam::new(I8));

        compiler.declare_imported_function(
            &mut function_builder,
            "compare_strings_less_than",
            compare_strings_less_than_signature,
        )?
    };

    let compare_strings_less_than_equal_function = {
        let mut compare_strings_less_than_equal_signature =
            Signature::new(compiler.module.isa().default_call_conv());

        compare_strings_less_than_equal_signature
            .params
            .extend([AbiParam::new(pointer_type), AbiParam::new(pointer_type)]);
        compare_strings_less_than_equal_signature
            .returns
            .push(AbiParam::new(I8));

        compiler.declare_imported_function(
            &mut function_builder,
            "compare_strings_less_than_equal",
            compare_strings_less_than_equal_signature,
        )?
    };

    let integer_to_string_function = {
        let mut integer_to_string_signature =
            Signature::new(compiler.module.isa().default_call_conv());

        integer_to_string_signature
            .params
            .extend([AbiParam::new(I64), AbiParam::new(pointer_type)]);
        integer_to_string_signature.returns.push(AbiParam::new(I64));

        compiler.declare_imported_function(
            &mut function_builder,
            "integer_to_string",
            integer_to_string_signature,
        )?
    };

    let read_line_function = {
        let mut read_line_signature = Signature::new(compiler.module.isa().default_call_conv());

        read_line_signature.params.push(AbiParam::new(pointer_type));
        read_line_signature.returns.push(AbiParam::new(I64));

        compiler.declare_imported_function(
            &mut function_builder,
            "read_line",
            read_line_signature,
        )?
    };

    let write_line_integer_function = {
        let mut write_line_signature = Signature::new(compiler.module.isa().default_call_conv());

        write_line_signature
            .params
            .extend([AbiParam::new(I64), AbiParam::new(pointer_type)]);
        write_line_signature.returns = vec![];

        compiler.declare_imported_function(
            &mut function_builder,
            "write_line_integer",
            write_line_signature,
        )?
    };

    let write_line_string_function = {
        let mut write_line_signature = Signature::new(compiler.module.isa().default_call_conv());

        write_line_signature
            .params
            .extend([AbiParam::new(pointer_type), AbiParam::new(pointer_type)]);
        write_line_signature.returns = vec![];

        compiler.declare_imported_function(
            &mut function_builder,
            "write_line_string",
            write_line_signature,
        )?
    };

    let integer_power_function = {
        let mut integer_power_signature = Signature::new(compiler.module.isa().default_call_conv());

        integer_power_signature
            .params
            .extend([AbiParam::new(I64), AbiParam::new(I64)]);
        integer_power_signature.returns.push(AbiParam::new(I64));

        compiler.declare_imported_function(
            &mut function_builder,
            "integer_power",
            integer_power_signature,
        )?
    };

    let float_power_function = {
        let mut float_power_signature = Signature::new(compiler.module.isa().default_call_conv());

        float_power_signature
            .params
            .extend([AbiParam::new(F64), AbiParam::new(F64)]);
        float_power_signature.returns.push(AbiParam::new(F64));

        compiler.declare_imported_function(
            &mut function_builder,
            "float_power",
            float_power_signature,
        )?
    };

    #[cfg(debug_assertions)]
    let log_operation_and_ip_function = {
        let mut log_operation_signature = Signature::new(compiler.module.isa().default_call_conv());

        log_operation_signature.params.push(AbiParam::new(I8));
        log_operation_signature.params.push(AbiParam::new(I64));
        log_operation_signature.returns = vec![];

        compiler.declare_imported_function(
            &mut function_builder,
            "log_operation_and_ip",
            log_operation_signature,
        )?
    };

    let bytecode_instructions = &prototype.instructions;
    let instruction_count = bytecode_instructions.len();

    let function_entry_block = function_builder.create_block();
    let mut instruction_blocks = Vec::with_capacity(instruction_count);
    let return_block = function_builder.create_block();
    let division_by_zero_block = function_builder.create_block();
    let mut switch = Switch::new();

    for ip in 0..instruction_count {
        let block = function_builder.create_block();

        instruction_blocks.push(block);
        switch.set_entry(ip as u128, block);
    }

    function_builder.switch_to_block(function_entry_block);
    function_builder.append_block_params_for_function_params(function_entry_block);

    let one = function_builder.ins().iconst(I64, 1);

    let thread_context = function_builder.block_params(function_entry_block)[0];
    let call_stack_buffer_pointer = function_builder.ins().load(
        pointer_type,
        MemFlags::new(),
        thread_context,
        offset_of!(ThreadContext, call_stack_buffer_pointer) as i32,
    );
    let call_stack_used_length_pointer = function_builder.ins().load(
        pointer_type,
        MemFlags::new(),
        thread_context,
        offset_of!(ThreadContext, call_stack_used_length_pointer) as i32,
    );
    let call_stack_used_length =
        function_builder
            .ins()
            .load(I64, MemFlags::new(), call_stack_used_length_pointer, 0);
    let top_call_frame_index = function_builder.ins().isub(call_stack_used_length, one);
    let register_stack_buffer_pointer = function_builder.ins().load(
        pointer_type,
        MemFlags::new(),
        thread_context,
        offset_of!(ThreadContext, register_stack_buffer_pointer) as i32,
    );
    let register_stack_used_length_pointer = function_builder.ins().load(
        pointer_type,
        MemFlags::new(),
        thread_context,
        offset_of!(ThreadContext, register_stack_used_length_pointer) as i32,
    );

    let register_count = function_builder
        .ins()
        .iconst(I64, prototype.register_count as i64);

    function_builder.ins().store(
        MemFlags::new(),
        register_count,
        register_stack_used_length_pointer,
        0,
    );

    let _register_tags_vec_pointer = function_builder.ins().load(
        pointer_type,
        MemFlags::new(),
        thread_context,
        offset_of!(ThreadContext, register_tags_vec_pointer) as i32,
    );
    let register_tags_buffer_pointer = function_builder.ins().load(
        pointer_type,
        MemFlags::new(),
        thread_context,
        offset_of!(ThreadContext, register_tags_buffer_pointer) as i32,
    );

    let (
        current_frame_ip,
        _current_frame_function_index,
        current_frame_register_range_start,
        current_frame_register_range_end,
        _current_frame_arguments_index,
        current_frame_destination_index,
    ) = get_call_frame(
        top_call_frame_index,
        call_stack_buffer_pointer,
        &mut function_builder,
    );

    let current_frame_base_register_offset = function_builder.ins().imul_imm(
        current_frame_register_range_start,
        size_of::<Register>() as i64,
    );
    let current_frame_base_register_address = function_builder.ins().iadd(
        register_stack_buffer_pointer,
        current_frame_base_register_offset,
    );

    let current_frame_base_tag_offset = function_builder.ins().imul_imm(
        current_frame_register_range_start,
        size_of::<RegisterTag>() as i64,
    );
    let current_frame_base_tag_address = function_builder
        .ins()
        .iadd(register_tags_buffer_pointer, current_frame_base_tag_offset);

    let return_register_pointer = function_builder.ins().load(
        pointer_type,
        MemFlags::new(),
        thread_context,
        offset_of!(ThreadContext, return_register_pointer) as i32,
    );
    let return_type_pointer = function_builder.ins().load(
        pointer_type,
        MemFlags::new(),
        thread_context,
        offset_of!(ThreadContext, return_type_pointer) as i32,
    );

    let hot_registers = HotRegisters {
        boolean: array::from_fn(|_| function_builder.declare_var(I64)),
        byte: array::from_fn(|_| function_builder.declare_var(I64)),
        character: array::from_fn(|_| function_builder.declare_var(I64)),
        float: array::from_fn(|_| function_builder.declare_var(F64)),
        integer: array::from_fn(|_| function_builder.declare_var(I64)),
        function: array::from_fn(|_| function_builder.declare_var(I64)),
    };

    switch.emit(&mut function_builder, current_frame_ip, return_block);

    for ip in 0..instruction_count {
        let current_instruction = &bytecode_instructions[ip];
        let operation = current_instruction.operation();
        let instruction_block = instruction_blocks[ip];

        function_builder.switch_to_block(instruction_block);

        info!("Compiling {operation} at IP {ip} in stackless function");

        #[cfg(debug_assertions)]
        {
            let op_code_value = function_builder.ins().iconst(I8, operation.0 as i64);
            let ip_value = function_builder.ins().iconst(I64, ip as i64);

            function_builder
                .ins()
                .call(log_operation_and_ip_function, &[op_code_value, ip_value]);
        }

        match operation {
            Operation::MOVE => {
                let Move {
                    destination,
                    operand,
                    r#type,
                    jump_distance,
                    jump_is_positive,
                } = Move::from(*current_instruction);
                let result_register = match r#type {
                    OperandType::BOOLEAN => get_boolean(
                        operand,
                        current_frame_base_register_address,
                        &hot_registers.boolean,
                        &mut function_builder,
                    )?,
                    OperandType::BYTE => get_byte(
                        operand,
                        current_frame_base_register_address,
                        &hot_registers.byte,
                        &mut function_builder,
                    )?,
                    OperandType::CHARACTER => get_character(
                        operand,
                        current_frame_base_register_address,
                        &compiler.program.constants,
                        &hot_registers.character,
                        &mut function_builder,
                    )?,
                    OperandType::FLOAT => get_float(
                        operand,
                        current_frame_base_register_address,
                        &compiler.program.constants,
                        &hot_registers.float,
                        &mut function_builder,
                    )?,
                    OperandType::INTEGER => get_integer(
                        operand,
                        current_frame_base_register_address,
                        &compiler.program.constants,
                        &hot_registers.integer,
                        &mut function_builder,
                    )?,
                    OperandType::STRING => get_string(
                        operand,
                        &compiler.program.constants,
                        allocate_string_function,
                        &mut function_builder,
                        thread_context,
                        current_frame_base_register_address,
                    )?,
                    OperandType::LIST_BOOLEAN
                    | OperandType::LIST_BYTE
                    | OperandType::LIST_CHARACTER
                    | OperandType::LIST_FLOAT
                    | OperandType::LIST_INTEGER
                    | OperandType::LIST_STRING
                    | OperandType::LIST_LIST
                    | OperandType::LIST_FUNCTION => get_list(
                        operand.index,
                        current_frame_base_register_address,
                        &mut function_builder,
                    ),
                    _ => {
                        return Err(JitError::UnsupportedOperandType {
                            operand_type: r#type,
                        });
                    }
                };

                JitCompiler::set_register(
                    destination,
                    result_register,
                    r#type.destination_type(),
                    current_frame_base_register_address,
                    current_frame_base_tag_address,
                    &hot_registers,
                    &mut function_builder,
                )?;

                if jump_distance > 0 {
                    let distance = (jump_distance + 1) as usize;

                    if jump_is_positive {
                        function_builder
                            .ins()
                            .jump(instruction_blocks[ip + distance], &[]);
                    } else {
                        function_builder
                            .ins()
                            .jump(instruction_blocks[ip - distance], &[]);
                    }
                } else {
                    function_builder.ins().jump(instruction_blocks[ip + 1], &[]);
                }
            }
            Operation::DROP => {
                let Drop {
                    drop_list_start,
                    drop_list_end,
                } = Drop::from(*current_instruction);
                let drop_list_range = drop_list_start as usize..drop_list_end as usize;

                let safepoint_registers = prototype.drop_lists.get(drop_list_range).ok_or(
                    JitError::DropListRangeOutOfBounds {
                        drop_list_start,
                        drop_list_end,
                        total_safepoint_count: prototype.drop_lists.len(),
                    },
                )?;

                for register_index in safepoint_registers {
                    let register_index_value =
                        function_builder.ins().iconst(I64, *register_index as i64);
                    let empty_tag_value = function_builder
                        .ins()
                        .iconst(RegisterTag::CRANELIFT_TYPE, RegisterTag::EMPTY.0 as i64);
                    let tag_offset = function_builder
                        .ins()
                        .imul_imm(register_index_value, size_of::<RegisterTag>() as i64);
                    let tag_address = function_builder
                        .ins()
                        .iadd(current_frame_base_tag_address, tag_offset);

                    function_builder
                        .ins()
                        .store(MemFlags::new(), empty_tag_value, tag_address, 0);
                }

                function_builder.ins().jump(instruction_blocks[ip + 1], &[]);
            }
            Operation::NEW_LIST => {
                let NewList {
                    destination,
                    initial_length,
                    list_type,
                } = NewList::from(*current_instruction);
                let list_type_value = function_builder.ins().iconst(I8, list_type.0 as i64);
                let list_length_value = function_builder.ins().iconst(I64, initial_length as i64);
                let call_allocate_list_instruction = function_builder.ins().call(
                    allocate_list_function,
                    &[
                        list_type_value,
                        list_length_value,
                        thread_context,
                        current_frame_register_range_start,
                        current_frame_register_range_end,
                    ],
                );
                let list_object_pointer =
                    function_builder.inst_results(call_allocate_list_instruction)[0];

                JitCompiler::set_register(
                    destination,
                    list_object_pointer,
                    list_type,
                    current_frame_base_register_address,
                    current_frame_base_tag_address,
                    &hot_registers,
                    &mut function_builder,
                )?;

                function_builder.ins().jump(instruction_blocks[ip + 1], &[]);
            }
            Operation::SET_LIST => {
                let SetList {
                    destination_list,
                    item_source,
                    list_index,
                    item_type,
                } = SetList::from(*current_instruction);
                let list_pointer = get_list(
                    destination_list,
                    current_frame_base_register_address,
                    &mut function_builder,
                );
                let item_value = match item_type {
                    OperandType::INTEGER => get_integer(
                        item_source,
                        current_frame_base_register_address,
                        &compiler.program.constants,
                        &hot_registers.integer,
                        &mut function_builder,
                    )?,
                    OperandType::BOOLEAN => get_boolean(
                        item_source,
                        current_frame_base_register_address,
                        &hot_registers.boolean,
                        &mut function_builder,
                    )?,
                    OperandType::BYTE => get_byte(
                        item_source,
                        current_frame_base_register_address,
                        &hot_registers.byte,
                        &mut function_builder,
                    )?,
                    OperandType::CHARACTER => get_character(
                        item_source,
                        current_frame_base_register_address,
                        &compiler.program.constants,
                        &hot_registers.character,
                        &mut function_builder,
                    )?,
                    OperandType::FLOAT => get_float(
                        item_source,
                        current_frame_base_register_address,
                        &compiler.program.constants,
                        &hot_registers.float,
                        &mut function_builder,
                    )?,
                    OperandType::STRING => get_string(
                        item_source,
                        &compiler.program.constants,
                        allocate_string_function,
                        &mut function_builder,
                        thread_context,
                        current_frame_base_register_address,
                    )?,
                    OperandType::LIST_BOOLEAN
                    | OperandType::LIST_BYTE
                    | OperandType::LIST_CHARACTER
                    | OperandType::LIST_FLOAT
                    | OperandType::LIST_INTEGER
                    | OperandType::LIST_STRING
                    | OperandType::LIST_LIST
                    | OperandType::LIST_FUNCTION => get_list(
                        item_source.index,
                        current_frame_base_register_address,
                        &mut function_builder,
                    ),
                    _ => {
                        return Err(JitError::UnsupportedOperandType {
                            operand_type: item_type,
                        });
                    }
                };
                let list_index = function_builder.ins().iconst(I64, list_index as i64);

                function_builder.ins().call(
                    instert_into_list_function,
                    &[list_pointer, list_index, item_value],
                );

                function_builder.ins().jump(instruction_blocks[ip + 1], &[]);
            }
            Operation::GET_LIST => {
                let GetList {
                    destination,
                    list,
                    list_index,
                    item_type,
                } = GetList::from(*current_instruction);

                let list_pointer = get_list(
                    list.index,
                    current_frame_base_register_address,
                    &mut function_builder,
                );
                let list_index = get_integer(
                    list_index,
                    current_frame_base_register_address,
                    &compiler.program.constants,
                    &hot_registers.integer,
                    &mut function_builder,
                )?;

                let call_get_list_instruction = function_builder.ins().call(
                    get_from_list_function,
                    &[list_pointer, list_index, thread_context],
                );
                let element_value = function_builder.inst_results(call_get_list_instruction)[0];

                if item_type == OperandType::FLOAT {
                    let element_value_as_float =
                        function_builder
                            .ins()
                            .bitcast(F64, MemFlags::new(), element_value);

                    JitCompiler::set_register(
                        destination,
                        element_value_as_float,
                        item_type,
                        current_frame_base_register_address,
                        current_frame_base_tag_address,
                        &hot_registers,
                        &mut function_builder,
                    )?;
                } else {
                    JitCompiler::set_register(
                        destination,
                        element_value,
                        item_type,
                        current_frame_base_register_address,
                        current_frame_base_tag_address,
                        &hot_registers,
                        &mut function_builder,
                    )?;
                }

                function_builder.ins().jump(instruction_blocks[ip + 1], &[]);
            }
            Operation::EQUAL | Operation::LESS | Operation::LESS_EQUAL => {
                let comparator = current_instruction.a_field();
                let left = current_instruction.b_address();
                let right = current_instruction.c_address();
                let operand_type = current_instruction.operand_type();
                let operation = current_instruction.operation();
                let comparison = match (operation, comparator != 0) {
                    (Operation::EQUAL, true) => IntCC::Equal,
                    (Operation::EQUAL, false) => IntCC::NotEqual,
                    (Operation::LESS, true) => IntCC::SignedLessThan,
                    (Operation::LESS, false) => IntCC::SignedGreaterThanOrEqual,
                    (Operation::LESS_EQUAL, true) => IntCC::SignedLessThanOrEqual,
                    (Operation::LESS_EQUAL, false) => IntCC::SignedGreaterThan,
                    _ => unreachable!(),
                };
                let comparison_result = match operand_type {
                    OperandType::BOOLEAN => {
                        let left_value = get_boolean(
                            left,
                            current_frame_base_register_address,
                            &hot_registers.boolean,
                            &mut function_builder,
                        )?;
                        let right_value = get_boolean(
                            right,
                            current_frame_base_register_address,
                            &hot_registers.boolean,
                            &mut function_builder,
                        )?;

                        function_builder
                            .ins()
                            .icmp(comparison, left_value, right_value)
                    }
                    OperandType::BYTE => {
                        let left_value = get_byte(
                            left,
                            current_frame_base_register_address,
                            &hot_registers.byte,
                            &mut function_builder,
                        )?;
                        let right_value = get_byte(
                            right,
                            current_frame_base_register_address,
                            &hot_registers.byte,
                            &mut function_builder,
                        )?;

                        function_builder
                            .ins()
                            .icmp(comparison, left_value, right_value)
                    }
                    OperandType::CHARACTER => {
                        let left_value = get_character(
                            left,
                            current_frame_base_register_address,
                            &compiler.program.constants,
                            &hot_registers.character,
                            &mut function_builder,
                        )?;
                        let right_value = get_character(
                            right,
                            current_frame_base_register_address,
                            &compiler.program.constants,
                            &hot_registers.character,
                            &mut function_builder,
                        )?;

                        function_builder
                            .ins()
                            .icmp(comparison, left_value, right_value)
                    }
                    OperandType::FLOAT => {
                        let comparison = match comparison {
                            IntCC::Equal => FloatCC::Equal,
                            IntCC::NotEqual => FloatCC::NotEqual,
                            IntCC::SignedLessThan => FloatCC::LessThan,
                            IntCC::SignedGreaterThanOrEqual => FloatCC::GreaterThanOrEqual,
                            IntCC::SignedLessThanOrEqual => FloatCC::LessThanOrEqual,
                            IntCC::SignedGreaterThan => FloatCC::GreaterThan,
                            _ => {
                                unreachable!();
                            }
                        };
                        let left_value = get_float(
                            left,
                            current_frame_base_register_address,
                            &compiler.program.constants,
                            &hot_registers.float,
                            &mut function_builder,
                        )?;
                        let right_value = get_float(
                            right,
                            current_frame_base_register_address,
                            &compiler.program.constants,
                            &hot_registers.float,
                            &mut function_builder,
                        )?;

                        function_builder
                            .ins()
                            .fcmp(comparison, left_value, right_value)
                    }
                    OperandType::INTEGER => {
                        let left_value = get_integer(
                            left,
                            current_frame_base_register_address,
                            &compiler.program.constants,
                            &hot_registers.integer,
                            &mut function_builder,
                        )?;
                        let right_value = get_integer(
                            right,
                            current_frame_base_register_address,
                            &compiler.program.constants,
                            &hot_registers.integer,
                            &mut function_builder,
                        )?;

                        function_builder
                            .ins()
                            .icmp(comparison, left_value, right_value)
                    }
                    OperandType::STRING => {
                        let left_pointer = get_string(
                            left,
                            &compiler.program.constants,
                            allocate_string_function,
                            &mut function_builder,
                            thread_context,
                            current_frame_base_register_address,
                        )?;
                        let right_pointer = get_string(
                            right,
                            &compiler.program.constants,
                            allocate_string_function,
                            &mut function_builder,
                            thread_context,
                            current_frame_base_register_address,
                        )?;
                        let compare_function = match operation {
                            Operation::EQUAL => compare_strings_equal_function,
                            Operation::LESS => compare_strings_less_than_function,
                            Operation::LESS_EQUAL => compare_strings_less_than_equal_function,
                            _ => {
                                return Err(JitError::UnhandledOperation { operation });
                            }
                        };

                        let call_instruction = function_builder
                            .ins()
                            .call(compare_function, &[left_pointer, right_pointer]);
                        let comparison_result = function_builder.inst_results(call_instruction)[0];

                        if comparator != 0 {
                            function_builder
                                .ins()
                                .icmp_imm(IntCC::Equal, comparison_result, 1)
                        } else {
                            function_builder
                                .ins()
                                .icmp_imm(IntCC::Equal, comparison_result, 0)
                        }
                    }
                    _ => {
                        return Err(JitError::UnsupportedOperandType { operand_type });
                    }
                };

                function_builder.ins().brif(
                    comparison_result,
                    instruction_blocks[ip + 2],
                    &[],
                    instruction_blocks[ip + 1],
                    &[],
                );
            }
            Operation::ADD
            | Operation::SUBTRACT
            | Operation::MULTIPLY
            | Operation::DIVIDE
            | Operation::MODULO
            | Operation::POWER => {
                let destination = current_instruction.destination();
                let left = current_instruction.b_address();
                let right = current_instruction.c_address();
                let r#type = current_instruction.operand_type();

                let result_register = match r#type {
                    OperandType::BYTE => {
                        let left_value = get_byte(
                            left,
                            current_frame_base_register_address,
                            &hot_registers.byte,
                            &mut function_builder,
                        )?;
                        let right_value = get_byte(
                            right,
                            current_frame_base_register_address,
                            &hot_registers.byte,
                            &mut function_builder,
                        )?;

                        match operation {
                            Operation::ADD => function_builder.ins().iadd(left_value, right_value),
                            Operation::SUBTRACT => {
                                function_builder.ins().isub(left_value, right_value)
                            }
                            Operation::MULTIPLY => {
                                function_builder.ins().imul(left_value, right_value)
                            }
                            Operation::DIVIDE => {
                                let division_block = function_builder.create_block();
                                let right_is_zero =
                                    function_builder
                                        .ins()
                                        .icmp_imm(IntCC::Equal, right_value, 0);

                                function_builder.ins().brif(
                                    right_is_zero,
                                    division_by_zero_block,
                                    &[],
                                    division_block,
                                    &[],
                                );
                                function_builder.switch_to_block(division_block);
                                function_builder.ins().udiv(left_value, right_value)
                            }
                            Operation::MODULO => {
                                let modulo_block = function_builder.create_block();
                                let right_is_zero =
                                    function_builder
                                        .ins()
                                        .icmp_imm(IntCC::Equal, right_value, 0);

                                function_builder.ins().brif(
                                    right_is_zero,
                                    division_by_zero_block,
                                    &[],
                                    modulo_block,
                                    &[],
                                );
                                function_builder.switch_to_block(modulo_block);
                                function_builder.ins().urem(left_value, right_value)
                            }
                            _ => {
                                return Err(JitError::UnhandledOperation { operation });
                            }
                        }
                    }
                    OperandType::INTEGER => {
                        let left_value = get_integer(
                            left,
                            current_frame_base_register_address,
                            &compiler.program.constants,
                            &hot_registers.integer,
                            &mut function_builder,
                        )?;
                        let right_value = get_integer(
                            right,
                            current_frame_base_register_address,
                            &compiler.program.constants,
                            &hot_registers.integer,
                            &mut function_builder,
                        )?;

                        match operation {
                            Operation::ADD => function_builder.ins().iadd(left_value, right_value),
                            Operation::SUBTRACT => {
                                function_builder.ins().isub(left_value, right_value)
                            }
                            Operation::MULTIPLY => {
                                function_builder.ins().imul(left_value, right_value)
                            }
                            Operation::DIVIDE => {
                                let division_block = function_builder.create_block();
                                let right_is_zero =
                                    function_builder
                                        .ins()
                                        .icmp_imm(IntCC::Equal, right_value, 0);

                                function_builder.ins().brif(
                                    right_is_zero,
                                    division_by_zero_block,
                                    &[],
                                    division_block,
                                    &[],
                                );
                                function_builder.switch_to_block(division_block);
                                function_builder.ins().sdiv(left_value, right_value)
                            }
                            Operation::MODULO => {
                                let modulo_block = function_builder.create_block();
                                let right_is_zero =
                                    function_builder
                                        .ins()
                                        .icmp_imm(IntCC::Equal, right_value, 0);

                                function_builder.ins().brif(
                                    right_is_zero,
                                    division_by_zero_block,
                                    &[],
                                    modulo_block,
                                    &[],
                                );
                                function_builder.switch_to_block(modulo_block);
                                function_builder.ins().srem(left_value, right_value)
                            }
                            Operation::POWER => {
                                let call_instruction = function_builder
                                    .ins()
                                    .call(integer_power_function, &[left_value, right_value]);

                                function_builder.inst_results(call_instruction)[0]
                            }
                            _ => {
                                return Err(JitError::UnhandledOperation { operation });
                            }
                        }
                    }
                    OperandType::FLOAT => {
                        let left_value = get_float(
                            left,
                            current_frame_base_register_address,
                            &compiler.program.constants,
                            &hot_registers.float,
                            &mut function_builder,
                        )?;
                        let right_value = get_float(
                            right,
                            current_frame_base_register_address,
                            &compiler.program.constants,
                            &hot_registers.float,
                            &mut function_builder,
                        )?;

                        match operation {
                            Operation::ADD => function_builder.ins().fadd(left_value, right_value),
                            Operation::SUBTRACT => {
                                function_builder.ins().fsub(left_value, right_value)
                            }
                            Operation::MULTIPLY => {
                                function_builder.ins().fmul(left_value, right_value)
                            }
                            Operation::DIVIDE => {
                                let division_block = function_builder.create_block();
                                let zero = function_builder.ins().f64const(0.0);
                                let right_is_zero =
                                    function_builder
                                        .ins()
                                        .fcmp(FloatCC::Equal, right_value, zero);

                                function_builder.ins().brif(
                                    right_is_zero,
                                    division_by_zero_block,
                                    &[],
                                    division_block,
                                    &[],
                                );
                                function_builder.switch_to_block(division_block);
                                function_builder.ins().fdiv(left_value, right_value)
                            }
                            Operation::MODULO => {
                                let modulo_block = function_builder.create_block();
                                let zero = function_builder.ins().f64const(0.0);
                                let right_is_zero =
                                    function_builder
                                        .ins()
                                        .fcmp(FloatCC::Equal, right_value, zero);

                                function_builder.ins().brif(
                                    right_is_zero,
                                    division_by_zero_block,
                                    &[],
                                    modulo_block,
                                    &[],
                                );
                                function_builder.switch_to_block(modulo_block);

                                let divided = function_builder.ins().fdiv(left_value, right_value);
                                let truncated = function_builder.ins().floor(divided);
                                let multiplied =
                                    function_builder.ins().fmul(truncated, right_value);

                                function_builder.ins().fsub(left_value, multiplied)
                            }
                            Operation::POWER => {
                                let call_instruction = function_builder
                                    .ins()
                                    .call(float_power_function, &[left_value, right_value]);

                                function_builder.inst_results(call_instruction)[0]
                            }
                            _ => {
                                return Err(JitError::UnhandledOperation { operation });
                            }
                        }
                    }
                    OperandType::STRING => {
                        if operation != Operation::ADD {
                            return Err(JitError::UnhandledOperation { operation });
                        }

                        let left_pointer = get_string(
                            left,
                            &compiler.program.constants,
                            allocate_string_function,
                            &mut function_builder,
                            thread_context,
                            current_frame_base_register_address,
                        )?;
                        let right_pointer = get_string(
                            right,
                            &compiler.program.constants,
                            allocate_string_function,
                            &mut function_builder,
                            thread_context,
                            current_frame_base_register_address,
                        )?;
                        let call_instruction = function_builder.ins().call(
                            concatenate_strings_function,
                            &[left_pointer, right_pointer, thread_context],
                        );

                        function_builder.inst_results(call_instruction)[0]
                    }
                    OperandType::CHARACTER_STRING => {
                        if operation != Operation::ADD {
                            return Err(JitError::UnhandledOperation { operation });
                        }

                        let left_value = get_character(
                            left,
                            current_frame_base_register_address,
                            &compiler.program.constants,
                            &hot_registers.character,
                            &mut function_builder,
                        )?;
                        let right_pointer = get_string(
                            right,
                            &compiler.program.constants,
                            allocate_string_function,
                            &mut function_builder,
                            thread_context,
                            current_frame_base_register_address,
                        )?;
                        let call_instruction = function_builder.ins().call(
                            concatenate_character_string_function,
                            &[left_value, right_pointer, thread_context],
                        );

                        function_builder.inst_results(call_instruction)[0]
                    }
                    OperandType::STRING_CHARACTER => {
                        if operation != Operation::ADD {
                            return Err(JitError::UnhandledOperation { operation });
                        }

                        let left_pointer = get_string(
                            left,
                            &compiler.program.constants,
                            allocate_string_function,
                            &mut function_builder,
                            thread_context,
                            current_frame_base_register_address,
                        )?;
                        let right_value = get_character(
                            right,
                            current_frame_base_register_address,
                            &compiler.program.constants,
                            &hot_registers.character,
                            &mut function_builder,
                        )?;
                        let call_instruction = function_builder.ins().call(
                            concatenate_string_character_function,
                            &[left_pointer, right_value, thread_context],
                        );

                        function_builder.inst_results(call_instruction)[0]
                    }
                    OperandType::CHARACTER => {
                        if operation != Operation::ADD {
                            return Err(JitError::UnhandledOperation { operation });
                        }

                        let left_value = get_character(
                            left,
                            current_frame_base_register_address,
                            &compiler.program.constants,
                            &hot_registers.character,
                            &mut function_builder,
                        )?;
                        let right_value = get_character(
                            right,
                            current_frame_base_register_address,
                            &compiler.program.constants,
                            &hot_registers.character,
                            &mut function_builder,
                        )?;

                        let call_instruction = function_builder.ins().call(
                            concatenate_characters_function,
                            &[left_value, right_value, thread_context],
                        );

                        function_builder.inst_results(call_instruction)[0]
                    }
                    _ => {
                        return Err(JitError::UnsupportedOperandType {
                            operand_type: r#type,
                        });
                    }
                };

                JitCompiler::set_register(
                    destination.index,
                    result_register,
                    r#type.destination_type(),
                    current_frame_base_register_address,
                    current_frame_base_tag_address,
                    &hot_registers,
                    &mut function_builder,
                )?;

                function_builder.ins().jump(instruction_blocks[ip + 1], &[]);
            }
            Operation::TEST => {
                let Test {
                    comparator,
                    operand,
                    jump_distance,
                } = Test::from(*current_instruction);

                let operand_value = get_boolean(
                    operand,
                    current_frame_base_register_address,
                    &hot_registers.boolean,
                    &mut function_builder,
                )?;
                let comparator_value = function_builder.ins().iconst(I64, comparator as i64);
                let comparison_result =
                    function_builder
                        .ins()
                        .icmp(IntCC::Equal, operand_value, comparator_value);

                let distance = (jump_distance + 1) as usize;

                function_builder.ins().brif(
                    comparison_result,
                    instruction_blocks[ip + distance],
                    &[],
                    instruction_blocks[ip + 1],
                    &[],
                );
            }
            Operation::NEGATE => {
                let Negate {
                    destination,
                    operand,
                    r#type,
                } = Negate::from(*current_instruction);

                let result_value = match r#type {
                    OperandType::BOOLEAN => {
                        let boolean_value = get_boolean(
                            operand,
                            current_frame_base_register_address,
                            &hot_registers.boolean,
                            &mut function_builder,
                        )?;
                        let is_zero =
                            function_builder
                                .ins()
                                .icmp_imm(IntCC::Equal, boolean_value, 0);
                        let one = function_builder.ins().iconst(I64, 1);
                        let zero = function_builder.ins().iconst(I64, 0);

                        function_builder.ins().select(is_zero, one, zero)
                    }
                    _ => {
                        return Err(JitError::UnsupportedOperandType {
                            operand_type: r#type,
                        });
                    }
                };

                JitCompiler::set_register(
                    destination,
                    result_value,
                    r#type,
                    current_frame_base_register_address,
                    current_frame_base_tag_address,
                    &hot_registers,
                    &mut function_builder,
                )?;

                function_builder.ins().jump(instruction_blocks[ip + 1], &[]);
            }
            Operation::CALL => {
                let Call {
                    destination_index,
                    prototype_index,
                    arguments_start,
                    argument_count,
                    return_type,
                } = Call::from(*current_instruction);
                let callee_function_ids = compiler
                    .function_ids
                    .get(prototype_index as usize)
                    .ok_or(JitError::FunctionIndexOutOfBounds {
                        ip,
                        function_index: prototype_index,
                        total_function_count: compiler.function_ids.len(),
                    })?;
                let FunctionIds::Other { direct, stackless } = callee_function_ids else {
                    unreachable!();
                };
                let is_recursive = *stackless == function_id;

                let callee_function_reference = if is_recursive {
                    compiler
                        .module
                        .declare_func_in_func(*stackless, function_builder.func)
                } else {
                    compiler
                        .module
                        .declare_func_in_func(*direct, function_builder.func)
                };
                let arguments_range =
                    arguments_start as usize..(arguments_start as usize + argument_count as usize);
                let call_arguments_list = prototype.call_arguments.get(arguments_range).ok_or(
                    JitError::ArgumentsRangeOutOfBounds {
                        arguments_start,
                        arguments_end: arguments_start + argument_count,
                        total_argument_count: prototype.call_arguments.len(),
                    },
                )?;

                let mut arguments =
                    SmallVec::<[CraneliftValue; 4]>::with_capacity(call_arguments_list.len() + 1);

                for (address, r#type) in call_arguments_list {
                    let argument_value = match *r#type {
                        OperandType::INTEGER => {
                            let integer_value = get_integer(
                                *address,
                                current_frame_base_register_address,
                                &compiler.program.constants,
                                &hot_registers.integer,
                                &mut function_builder,
                            )?;

                            Ok(integer_value)
                        }?,
                        _ => {
                            return Err(JitError::UnsupportedOperandType {
                                operand_type: *r#type,
                            });
                        }
                    };

                    arguments.push(argument_value);
                }

                arguments.push(thread_context);

                let call_instruction = function_builder
                    .ins()
                    .call(callee_function_reference, &arguments);

                if return_type != OperandType::NONE {
                    let return_value = function_builder.inst_results(call_instruction)[0];

                    JitCompiler::set_register(
                        destination_index,
                        return_value,
                        return_type,
                        current_frame_base_register_address,
                        current_frame_base_tag_address,
                        &hot_registers,
                        &mut function_builder,
                    )?;
                }

                function_builder.ins().jump(instruction_blocks[ip + 1], &[]);
            }
            Operation::CALL_NATIVE => {
                let CallNative {
                    destination,
                    function,
                    arguments_index,
                } = CallNative::from(*current_instruction);

                let function_type = function.r#type();
                let argument_count = function_type.value_parameters.len();
                let arguments_range =
                    arguments_index as usize..(arguments_index as usize + argument_count);
                let call_arguments_list = prototype.call_arguments.get(arguments_range).ok_or(
                    JitError::ArgumentsRangeOutOfBounds {
                        arguments_start: arguments_index,
                        arguments_end: arguments_index + argument_count as u16,
                        total_argument_count: prototype.call_arguments.len(),
                    },
                )?;
                let mut arguments = Vec::with_capacity(call_arguments_list.len() + 1);

                for (address, r#type) in call_arguments_list {
                    let argument_value = match *r#type {
                        OperandType::STRING => get_string(
                            *address,
                            &compiler.program.constants,
                            allocate_string_function,
                            &mut function_builder,
                            thread_context,
                            current_frame_base_register_address,
                        )?,
                        OperandType::INTEGER => get_integer(
                            *address,
                            current_frame_base_register_address,
                            &compiler.program.constants,
                            &hot_registers.integer,
                            &mut function_builder,
                        )?,
                        _ => {
                            return Err(JitError::UnsupportedOperandType {
                                operand_type: *r#type,
                            });
                        }
                    };

                    arguments.push(argument_value);
                }

                arguments.push(thread_context);

                let function_reference = match function.name() {
                    "read_line" => read_line_function,
                    "write_line"
                        if call_arguments_list.len() == 1
                            && call_arguments_list[0].1 == OperandType::STRING =>
                    {
                        write_line_string_function
                    }
                    "write_line"
                        if call_arguments_list.len() == 1
                            && call_arguments_list[0].1 == OperandType::INTEGER =>
                    {
                        write_line_integer_function
                    }
                    _ => {
                        return Err(JitError::UnhandledNativeFunction {
                            function_name: function.name().to_string(),
                        });
                    }
                };

                let call_instruction = function_builder.ins().call(function_reference, &arguments);

                if function_type.return_type != Type::None {
                    let return_value = function_builder.inst_results(call_instruction)[0];

                    JitCompiler::set_register(
                        destination,
                        return_value,
                        function_type.return_type.as_operand_type(),
                        current_frame_base_register_address,
                        current_frame_base_tag_address,
                        &hot_registers,
                        &mut function_builder,
                    )?;
                }

                function_builder.ins().jump(instruction_blocks[ip + 1], &[]);
            }
            Operation::JUMP => {
                let Jump {
                    offset,
                    is_positive,
                    drop_list_start,
                    drop_list_end,
                } = Jump::from(*current_instruction);
                let offset = offset as usize + 1;
                let next_ip = if is_positive {
                    ip + offset
                } else {
                    ip - offset
                };

                if drop_list_end > drop_list_start {
                    let drop_list_range = drop_list_start as usize..drop_list_end as usize;

                    let safepoint_registers = prototype.drop_lists.get(drop_list_range).ok_or(
                        JitError::DropListRangeOutOfBounds {
                            drop_list_start,
                            drop_list_end,
                            total_safepoint_count: prototype.drop_lists.len(),
                        },
                    )?;

                    for register_index in safepoint_registers {
                        let register_index_value =
                            function_builder.ins().iconst(I64, *register_index as i64);
                        let empty_tag_value = function_builder
                            .ins()
                            .iconst(RegisterTag::CRANELIFT_TYPE, RegisterTag::EMPTY.0 as i64);
                        let tag_offset = function_builder
                            .ins()
                            .imul_imm(register_index_value, size_of::<RegisterTag>() as i64);
                        let tag_address = function_builder
                            .ins()
                            .iadd(current_frame_base_tag_address, tag_offset);

                        function_builder.ins().store(
                            MemFlags::new(),
                            empty_tag_value,
                            tag_address,
                            0,
                        );
                    }
                }

                function_builder
                    .ins()
                    .jump(instruction_blocks[next_ip], &[]);
            }
            Operation::RETURN => {
                let Return {
                    should_return_value,
                    return_value_address,
                    r#type,
                } = Return::from(*current_instruction);

                if should_return_value {
                    let (value_to_return, return_type) = match r#type {
                        OperandType::BOOLEAN => {
                            let boolean_value = get_boolean(
                                return_value_address,
                                current_frame_base_register_address,
                                &hot_registers.boolean,
                                &mut function_builder,
                            )?;
                            let boolean_type = function_builder
                                .ins()
                                .iconst(I8, OperandType::BOOLEAN.0 as i64);

                            (boolean_value, boolean_type)
                        }
                        OperandType::BYTE => {
                            let byte_value = get_byte(
                                return_value_address,
                                current_frame_base_register_address,
                                &hot_registers.byte,
                                &mut function_builder,
                            )?;
                            let byte_type = function_builder
                                .ins()
                                .iconst(I8, OperandType::BYTE.0 as i64);

                            (byte_value, byte_type)
                        }
                        OperandType::CHARACTER => {
                            let character_value = get_character(
                                return_value_address,
                                current_frame_base_register_address,
                                &compiler.program.constants,
                                &hot_registers.character,
                                &mut function_builder,
                            )?;
                            let character_type = function_builder
                                .ins()
                                .iconst(I8, OperandType::CHARACTER.0 as i64);

                            (character_value, character_type)
                        }
                        OperandType::FLOAT => {
                            let float_value = get_float(
                                return_value_address,
                                current_frame_base_register_address,
                                &compiler.program.constants,
                                &hot_registers.float,
                                &mut function_builder,
                            )?;
                            let float_type = function_builder
                                .ins()
                                .iconst(I8, OperandType::FLOAT.0 as i64);

                            (float_value, float_type)
                        }
                        OperandType::INTEGER => {
                            let integer_value = get_integer(
                                return_value_address,
                                current_frame_base_register_address,
                                &compiler.program.constants,
                                &hot_registers.integer,
                                &mut function_builder,
                            )?;
                            let integer_type = function_builder
                                .ins()
                                .iconst(I8, OperandType::INTEGER.0 as i64);

                            (integer_value, integer_type)
                        }
                        OperandType::STRING => {
                            let string_value = get_string(
                                return_value_address,
                                &compiler.program.constants,
                                allocate_string_function,
                                &mut function_builder,
                                thread_context,
                                current_frame_base_register_address,
                            )?;
                            let string_type = function_builder
                                .ins()
                                .iconst(I8, OperandType::STRING.0 as i64);

                            (string_value, string_type)
                        }
                        OperandType::LIST_BOOLEAN => {
                            let list_value = get_list(
                                return_value_address.index,
                                current_frame_base_register_address,
                                &mut function_builder,
                            );
                            let list_type = function_builder
                                .ins()
                                .iconst(I8, OperandType::LIST_BOOLEAN.0 as i64);

                            (list_value, list_type)
                        }
                        OperandType::LIST_BYTE => {
                            let list_value = get_list(
                                return_value_address.index,
                                current_frame_base_register_address,
                                &mut function_builder,
                            );
                            let list_type = function_builder
                                .ins()
                                .iconst(I8, OperandType::LIST_BYTE.0 as i64);

                            (list_value, list_type)
                        }
                        OperandType::LIST_CHARACTER => {
                            let list_value = get_list(
                                return_value_address.index,
                                current_frame_base_register_address,
                                &mut function_builder,
                            );
                            let list_type = function_builder
                                .ins()
                                .iconst(I8, OperandType::LIST_CHARACTER.0 as i64);

                            (list_value, list_type)
                        }
                        OperandType::LIST_FLOAT => {
                            let list_value = get_list(
                                return_value_address.index,
                                current_frame_base_register_address,
                                &mut function_builder,
                            );
                            let list_type = function_builder
                                .ins()
                                .iconst(I8, OperandType::LIST_FLOAT.0 as i64);

                            (list_value, list_type)
                        }
                        OperandType::LIST_INTEGER => {
                            let list_value = get_list(
                                return_value_address.index,
                                current_frame_base_register_address,
                                &mut function_builder,
                            );
                            let list_type = function_builder
                                .ins()
                                .iconst(I8, OperandType::LIST_INTEGER.0 as i64);

                            (list_value, list_type)
                        }
                        OperandType::LIST_STRING => {
                            let list_value = get_list(
                                return_value_address.index,
                                current_frame_base_register_address,
                                &mut function_builder,
                            );
                            let list_type = function_builder
                                .ins()
                                .iconst(I8, OperandType::LIST_STRING.0 as i64);

                            (list_value, list_type)
                        }
                        OperandType::LIST_LIST => {
                            let list_value = get_list(
                                return_value_address.index,
                                current_frame_base_register_address,
                                &mut function_builder,
                            );
                            let list_type = function_builder
                                .ins()
                                .iconst(I8, OperandType::LIST_LIST.0 as i64);

                            (list_value, list_type)
                        }
                        _ => {
                            return Err(JitError::UnsupportedOperandType {
                                operand_type: r#type,
                            });
                        }
                    };

                    if is_main {
                        function_builder.ins().store(
                            MemFlags::new(),
                            value_to_return,
                            return_register_pointer,
                            0,
                        );
                        function_builder.ins().store(
                            MemFlags::new(),
                            return_type,
                            return_type_pointer,
                            0,
                        );
                    } else {
                        let return_destination_offset = function_builder.ins().imul_imm(
                            current_frame_destination_index,
                            size_of::<Register>() as i64,
                        );
                        let return_destination_address = function_builder
                            .ins()
                            .iadd(register_stack_buffer_pointer, return_destination_offset);

                        function_builder.ins().store(
                            MemFlags::new(),
                            value_to_return,
                            return_destination_address,
                            0,
                        );

                        let return_tag_offset = function_builder.ins().imul_imm(
                            current_frame_destination_index,
                            size_of::<RegisterTag>() as i64,
                        );
                        let return_tag_address = function_builder
                            .ins()
                            .iadd(register_tags_buffer_pointer, return_tag_offset);
                        let return_tag = if r#type.is_scalar() {
                            function_builder
                                .ins()
                                .iconst(I8, RegisterTag::SCALAR.0 as i64)
                        } else {
                            function_builder
                                .ins()
                                .iconst(I8, RegisterTag::OBJECT.0 as i64)
                        };

                        function_builder.ins().store(
                            MemFlags::new(),
                            return_tag,
                            return_tag_address,
                            0,
                        );
                    }
                }

                let current_length = function_builder.ins().load(
                    I64,
                    MemFlags::new(),
                    call_stack_used_length_pointer,
                    0,
                );
                let new_length = function_builder.ins().isub(current_length, one);

                function_builder.ins().store(
                    MemFlags::new(),
                    new_length,
                    call_stack_used_length_pointer,
                    0,
                );

                let register_stack_used_length = function_builder.ins().load(
                    I64,
                    MemFlags::new(),
                    register_stack_used_length_pointer,
                    0,
                );
                let current_frame_register_window_length = function_builder.ins().isub(
                    current_frame_register_range_end,
                    current_frame_register_range_start,
                );
                let new_register_stack_length = function_builder.ins().isub(
                    register_stack_used_length,
                    current_frame_register_window_length,
                );

                function_builder.ins().store(
                    MemFlags::new(),
                    new_register_stack_length,
                    register_stack_used_length_pointer,
                    0,
                );

                function_builder.ins().return_(&[]);
            }
            Operation::TO_STRING => {
                let ToString {
                    destination,
                    operand,
                    r#type,
                } = ToString::from(*current_instruction);

                let string_value = match r#type {
                    OperandType::INTEGER => {
                        let integer_operand = get_integer(
                            operand,
                            current_frame_base_register_address,
                            &compiler.program.constants,
                            &hot_registers.integer,
                            &mut function_builder,
                        )?;
                        let call_instruction = function_builder.ins().call(
                            integer_to_string_function,
                            &[integer_operand, thread_context],
                        );

                        function_builder.inst_results(call_instruction)[0]
                    }
                    OperandType::STRING => get_string(
                        operand,
                        &compiler.program.constants,
                        allocate_string_function,
                        &mut function_builder,
                        thread_context,
                        current_frame_base_register_address,
                    )?,
                    _ => {
                        return Err(JitError::UnsupportedOperandType {
                            operand_type: r#type,
                        });
                    }
                };

                JitCompiler::set_register(
                    destination,
                    string_value,
                    OperandType::STRING,
                    current_frame_base_register_address,
                    current_frame_base_tag_address,
                    &hot_registers,
                    &mut function_builder,
                )?;

                function_builder.ins().jump(instruction_blocks[ip + 1], &[]);
            }

            _ => {
                return Err(JitError::UnhandledOperation { operation });
            }
        }
    }

    {
        function_builder.switch_to_block(return_block);
        function_builder.ins().return_(&[]);
    }

    {
        function_builder.switch_to_block(division_by_zero_block);

        let division_by_zero_status = function_builder
            .ins()
            .iconst(I8, ThreadStatus::ErrorDivisionByZero as i64);

        function_builder.ins().store(
            MemFlags::new(),
            division_by_zero_status,
            thread_context,
            offset_of!(ThreadContext, status) as i32,
        );
        function_builder.ins().return_(&[]);
    }

    function_builder.seal_all_blocks();

    compiler
        .module
        .define_function(function_id, &mut compilation_context)
        .map_err(|error| {
            if let ModuleError::Compilation(CodegenError::Verifier(errors)) = error {
                let message = errors
                    .0
                    .iter()
                    .map(|error| format!("\n{error}"))
                    .collect::<String>();
                let cranelift_ir = compilation_context.func.display().to_string();

                JitError::CompilationError {
                    message,
                    cranelift_ir,
                }
            } else {
                let cranelift_ir = compilation_context.func.display().to_string();

                JitError::CompilationError {
                    message: error.to_string(),
                    cranelift_ir,
                }
            }
        })?;

    compiler.module.clear_context(&mut compilation_context);

    Ok(())
}

fn get_boolean(
    address: Address,
    frame_base_address: CraneliftValue,
    hot_registers: &[Variable],
    function_builder: &mut FunctionBuilder,
) -> Result<CraneliftValue, JitError> {
    let jit_value = match address.memory {
        MemoryKind::REGISTER => {
            if let Some(variable) = hot_registers.get(address.index as usize) {
                return Ok(function_builder.use_var(*variable));
            }

            let relative_index = function_builder.ins().iconst(I64, address.index as i64);
            let byte_offset = function_builder
                .ins()
                .imul_imm(relative_index, size_of::<Register>() as i64);
            let address = function_builder.ins().iadd(frame_base_address, byte_offset);

            function_builder
                .ins()
                .load(I64, MemFlags::new(), address, 0)
        }
        MemoryKind::ENCODED => {
            let boolean_value = address.index != 0;

            function_builder.ins().iconst(I64, boolean_value as i64)
        }
        _ => {
            return Err(JitError::UnsupportedMemoryKind {
                memory_kind: address.memory,
            });
        }
    };

    Ok(jit_value)
}

fn get_byte(
    address: Address,
    frame_base_address: CraneliftValue,
    hot_registers: &[Variable],
    function_builder: &mut FunctionBuilder,
) -> Result<CraneliftValue, JitError> {
    let jit_value = match address.memory {
        MemoryKind::REGISTER => {
            if let Some(variable) = hot_registers.get(address.index as usize) {
                return Ok(function_builder.use_var(*variable));
            }

            let relative_index = function_builder.ins().iconst(I64, address.index as i64);
            let byte_offset = function_builder
                .ins()
                .imul_imm(relative_index, size_of::<Register>() as i64);
            let address = function_builder.ins().iadd(frame_base_address, byte_offset);

            function_builder
                .ins()
                .load(I64, MemFlags::new(), address, 0)
        }
        MemoryKind::ENCODED => function_builder.ins().iconst(I64, address.index as i64),
        _ => {
            return Err(JitError::UnsupportedMemoryKind {
                memory_kind: address.memory,
            });
        }
    };

    Ok(jit_value)
}

fn get_character(
    address: Address,
    frame_base_address: CraneliftValue,
    constants: &ConstantTable,
    hot_registers: &[Variable],
    function_builder: &mut FunctionBuilder,
) -> Result<CraneliftValue, JitError> {
    let jit_value = match address.memory {
        MemoryKind::REGISTER => {
            if let Some(variable) = hot_registers.get(address.index as usize) {
                return Ok(function_builder.use_var(*variable));
            }

            let relative_index = function_builder.ins().iconst(I64, address.index as i64);
            let byte_offset = function_builder
                .ins()
                .imul_imm(relative_index, size_of::<Register>() as i64);
            let address = function_builder.ins().iadd(frame_base_address, byte_offset);

            function_builder
                .ins()
                .load(I64, MemFlags::new(), address, 0)
        }
        MemoryKind::CONSTANT => {
            let character = constants.get_character(address.index).ok_or(
                JitError::ConstantIndexOutOfBounds {
                    constant_index: address.index,
                    total_constant_count: constants.len(),
                },
            )?;

            function_builder.ins().iconst(I64, character as i64)
        }
        _ => {
            return Err(JitError::UnsupportedMemoryKind {
                memory_kind: address.memory,
            });
        }
    };

    Ok(jit_value)
}

fn get_float(
    address: Address,
    frame_base_address: CraneliftValue,
    constants: &ConstantTable,
    hot_registers: &[Variable],
    function_builder: &mut FunctionBuilder,
) -> Result<CraneliftValue, JitError> {
    let jit_value = match address.memory {
        MemoryKind::REGISTER => {
            if let Some(variable) = hot_registers.get(address.index as usize) {
                return Ok(function_builder.use_var(*variable));
            }

            let relative_index = function_builder.ins().iconst(I64, address.index as i64);
            let byte_offset = function_builder
                .ins()
                .imul_imm(relative_index, size_of::<Register>() as i64);
            let address = function_builder.ins().iadd(frame_base_address, byte_offset);

            function_builder
                .ins()
                .load(F64, MemFlags::new(), address, 0)
        }
        MemoryKind::CONSTANT => {
            let float =
                constants
                    .get_float(address.index)
                    .ok_or(JitError::ConstantIndexOutOfBounds {
                        constant_index: address.index,
                        total_constant_count: constants.len(),
                    })?;

            function_builder.ins().f64const(float)
        }
        _ => {
            return Err(JitError::UnsupportedMemoryKind {
                memory_kind: address.memory,
            });
        }
    };

    Ok(jit_value)
}

fn get_integer(
    address: Address,
    frame_base_address: CraneliftValue,
    constants: &ConstantTable,
    hot_registers: &[Variable],
    function_builder: &mut FunctionBuilder,
) -> Result<CraneliftValue, JitError> {
    let jit_value = match address.memory {
        MemoryKind::REGISTER => {
            if let Some(variable) = hot_registers.get(address.index as usize) {
                return Ok(function_builder.use_var(*variable));
            }

            let relative_index = function_builder.ins().iconst(I64, address.index as i64);
            let byte_offset = function_builder
                .ins()
                .imul_imm(relative_index, size_of::<Register>() as i64);
            let address = function_builder.ins().iadd(frame_base_address, byte_offset);

            function_builder
                .ins()
                .load(I64, MemFlags::new(), address, 0)
        }
        MemoryKind::CONSTANT => {
            let integer =
                constants
                    .get_integer(address.index)
                    .ok_or(JitError::ConstantIndexOutOfBounds {
                        constant_index: address.index,
                        total_constant_count: constants.len(),
                    })?;

            function_builder.ins().iconst(I64, integer)
        }
        _ => {
            return Err(JitError::UnsupportedMemoryKind {
                memory_kind: address.memory,
            });
        }
    };

    Ok(jit_value)
}

fn get_string(
    address: Address,
    constants: &ConstantTable,
    allocate_string_function: FuncRef,
    function_builder: &mut FunctionBuilder,
    thread_conxtext_pointer: CraneliftValue,
    current_frame_base_register_address: CraneliftValue,
) -> Result<CraneliftValue, JitError> {
    let register_value = match address.memory {
        MemoryKind::REGISTER => {
            let relative_index = function_builder.ins().iconst(I64, address.index as i64);
            let relative_register_offset = function_builder
                .ins()
                .imul_imm(relative_index, size_of::<Register>() as i64);
            let register_address = function_builder.ins().iadd(
                current_frame_base_register_address,
                relative_register_offset,
            );

            function_builder
                .ins()
                .load(I64, MemFlags::new(), register_address, 0)
        }
        MemoryKind::CONSTANT => {
            let (string_pointer, string_length) = constants
                .get_string_raw_parts(address.index)
                .ok_or(JitError::ConstantIndexOutOfBounds {
                    constant_index: address.index,
                    total_constant_count: constants.len(),
                })?;
            let string_pointer = function_builder.ins().iconst(I64, string_pointer as i64);
            let string_length = function_builder.ins().iconst(I64, string_length as i64);
            let call_allocate_string_instruction = function_builder.ins().call(
                allocate_string_function,
                &[string_pointer, string_length, thread_conxtext_pointer],
            );

            function_builder.inst_results(call_allocate_string_instruction)[0]
        }
        _ => {
            return Err(JitError::UnsupportedMemoryKind {
                memory_kind: address.memory,
            });
        }
    };

    Ok(register_value)
}

fn get_list(
    register: u16,
    frame_base_address: CraneliftValue,
    function_builder: &mut FunctionBuilder,
) -> CraneliftValue {
    let relative_index = function_builder.ins().iconst(I64, register as i64);
    let byte_offset = function_builder
        .ins()
        .imul_imm(relative_index, size_of::<Register>() as i64);
    let address = function_builder.ins().iadd(frame_base_address, byte_offset);

    function_builder
        .ins()
        .load(I64, MemFlags::new(), address, 0)
}

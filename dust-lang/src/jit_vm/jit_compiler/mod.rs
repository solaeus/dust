mod compile_direct_function;
mod compile_stackless_function;
mod functions;
mod jit_error;

use std::mem::{offset_of, transmute};

use compile_direct_function::compile_direct_function;
use compile_stackless_function::compile_stackless_function;
use functions::*;
pub use jit_error::{JIT_ERROR_TEXT, JitError};

use cranelift::{
    codegen::{
        CodegenError,
        ir::{FuncRef, InstBuilder},
    },
    frontend::Switch,
    prelude::{
        AbiParam, Block, FunctionBuilder, FunctionBuilderContext, IntCC, MemFlags, Signature,
        Value as CraneliftValue, types::I64,
    },
};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module, ModuleError};
use tracing::Level;

use crate::{
    Program, Register, ThreadResult,
    jit_vm::{
        call_stack::{get_frame_function_index, push_call_frame},
        thread::ThreadContext,
    },
};

pub struct JitCompiler<'a> {
    module: JITModule,
    program: &'a Program,
    main_function_id: FuncId,
    function_ids: Vec<FunctionIds>,
}

impl<'a> JitCompiler<'a> {
    pub fn new(program: &'a Program) -> Self {
        let mut builder = JITBuilder::new(cranelift_module::default_libcall_names()).unwrap();

        builder.symbol("allocate_list", allocate_list as *const u8);
        builder.symbol("insert_into_list", insert_into_list as *const u8);

        builder.symbol("allocate_string", allocate_string as *const u8);
        builder.symbol("concatenate_strings", concatenate_strings as *const u8);

        #[cfg(debug_assertions)]
        builder.symbol("log_operation", log_operation as *const u8);

        let module = JITModule::new(builder);

        Self {
            module,
            program,
            main_function_id: FuncId::from_u32(0),
            function_ids: Vec::with_capacity(program.prototypes.len()),
        }
    }

    pub fn compile(&mut self) -> Result<JitLogic, JitError> {
        let span = tracing::span!(Level::INFO, "JIT_Compiler");
        let _enter = span.enter();

        let loop_pointer = self.compile_loop()?;

        Ok(unsafe { transmute::<*const u8, JitLogic>(loop_pointer) })
    }

    fn compile_loop(&mut self) -> Result<*const u8, JitError> {
        let mut context = self.module.make_context();
        let pointer_type = self.module.isa().pointer_type();
        let mut stackless_signature = Signature::new(self.module.isa().default_call_conv());

        stackless_signature.params.push(AbiParam::new(pointer_type));

        self.main_function_id = self
            .module
            .declare_function("main", Linkage::Local, &stackless_signature)
            .map_err(|error| JitError::CraneliftModuleError {
                message: error.to_string(),
            })?;

        for (index, chunk) in self.program.prototypes.iter().enumerate() {
            let name = chunk
                .name
                .as_ref()
                .map_or_else(|| format!("proto_{index}"), |path| path.to_string());
            let direct_name = format!("{name}_direct");
            let stackless_name = format!("{name}_stackless");
            let mut direct_signature = Signature::new(self.module.isa().default_call_conv());

            direct_signature.returns.push(AbiParam::new(I64));

            for _ in 0..chunk.r#type.value_parameters.len() {
                direct_signature.params.push(AbiParam::new(I64));
            }

            let direct_function_id = self
                .module
                .declare_function(&direct_name, Linkage::Local, &direct_signature)
                .map_err(|error| JitError::CraneliftModuleError {
                    message: error.to_string(),
                })?;
            let stackless_function_id = self
                .module
                .declare_function(&stackless_name, Linkage::Local, &stackless_signature)
                .map_err(|error| JitError::CraneliftModuleError {
                    message: error.to_string(),
                })?;

            self.function_ids.push(FunctionIds {
                direct: direct_function_id,
                stackless: stackless_function_id,
            });
        }

        let main_function_reference = {
            let reference = self
                .module
                .declare_func_in_func(self.main_function_id, &mut context.func);

            compile_stackless_function(
                self,
                self.main_function_id,
                &self.program.main_chunk,
                true,
            )?;

            reference
        };
        let function_references = {
            let mut references = Vec::with_capacity(self.program.prototypes.len());

            for (index, FunctionIds { direct, stackless }) in
                self.function_ids.clone().into_iter().enumerate()
            {
                let direct_reference = self.module.declare_func_in_func(direct, &mut context.func);
                let stackless_reference = self
                    .module
                    .declare_func_in_func(stackless, &mut context.func);

                references.push((direct_reference, stackless_reference));

                let chunk = &self.program.prototypes[index];

                compile_direct_function(self, direct, chunk)?;
                compile_stackless_function(self, stackless, chunk, false)?;
            }

            references
        };

        context.func.signature = stackless_signature;
        context
            .func
            .signature
            .returns
            .push(AbiParam::new(ThreadResult::CRANELIFT_TYPE));

        let loop_function_id = self
            .module
            .declare_function("loop", Linkage::Local, &context.func.signature)
            .map_err(|error| JitError::CraneliftModuleError {
                message: error.to_string(),
            })?;
        let mut function_builder_context = FunctionBuilderContext::new();
        let mut function_builder =
            FunctionBuilder::new(&mut context.func, &mut function_builder_context);
        let mut switch = Switch::new();

        let entry_block = {
            let block = function_builder.create_block();

            function_builder.append_block_params_for_function_params(block);

            block
        };
        let check_for_empty_call_stack_block = function_builder.create_block();
        let check_for_error_function_index_out_of_bounds_block = function_builder.create_block();
        let loop_block = function_builder.create_block();
        let main_function_block = function_builder.create_block();
        let function_blocks = {
            let mut blocks = Vec::with_capacity(self.program.prototypes.len());

            for index in 0..function_references.len() {
                let block = function_builder.create_block();

                blocks.push(block);
                switch.set_entry(index as u128, block);
            }

            blocks
        };
        let return_block = function_builder.create_block();

        let (thread_context_pointer, call_stack_buffer_pointer, call_stack_used_length_pointer) = {
            function_builder.switch_to_block(entry_block);

            let thread_context = function_builder.block_params(entry_block)[0];

            let call_stack_buffer_pointer = function_builder.ins().load(
                pointer_type,
                MemFlags::new(),
                thread_context,
                offset_of!(ThreadContext, call_stack_buffer_pointer) as i32,
            );
            let call_stack_used_length_pointer = function_builder.ins().load(
                I64,
                MemFlags::new(),
                thread_context,
                offset_of!(ThreadContext, call_stack_used_length_pointer) as i32,
            );

            let zero = function_builder.ins().iconst(I64, 0);
            let register_count = function_builder
                .ins()
                .iconst(I64, self.program.main_chunk.register_count as i64);
            let null_function_index = function_builder.ins().iconst(I64, u32::MAX as i64);

            push_call_frame(
                zero,
                zero,
                null_function_index,
                zero,
                register_count,
                zero,
                zero,
                call_stack_buffer_pointer,
                call_stack_used_length_pointer,
                &mut function_builder,
            );
            function_builder
                .ins()
                .jump(check_for_empty_call_stack_block, &[]);

            (
                thread_context,
                call_stack_buffer_pointer,
                call_stack_used_length_pointer,
            )
        };

        {
            function_builder.switch_to_block(check_for_empty_call_stack_block);

            let call_stack_length = function_builder.ins().load(
                I64,
                MemFlags::new(),
                call_stack_used_length_pointer,
                0,
            );
            let call_stack_is_empty =
                function_builder
                    .ins()
                    .icmp_imm(IntCC::Equal, call_stack_length, 0);
            let return_thread_status = function_builder
                .ins()
                .iconst(ThreadResult::CRANELIFT_TYPE, ThreadResult::Return as i64);

            function_builder.ins().brif(
                call_stack_is_empty,
                return_block,
                &[return_thread_status.into()],
                check_for_error_function_index_out_of_bounds_block,
                &[],
            );
        }

        {
            function_builder.switch_to_block(check_for_error_function_index_out_of_bounds_block);

            let call_stack_length = function_builder.ins().load(
                I64,
                MemFlags::new(),
                call_stack_used_length_pointer,
                0,
            );
            let call_stack_is_empty =
                function_builder
                    .ins()
                    .icmp_imm(IntCC::Equal, call_stack_length, 0);
            let return_thread_status = function_builder.ins().iconst(
                ThreadResult::CRANELIFT_TYPE,
                ThreadResult::ErrorFunctionIndexOutOfBounds as i64,
            );

            function_builder.ins().brif(
                call_stack_is_empty,
                return_block,
                &[return_thread_status.into()],
                loop_block,
                &[],
            );
        }

        {
            function_builder.switch_to_block(loop_block);

            let top_call_frame_index = {
                let call_stack_length = function_builder.ins().load(
                    I64,
                    MemFlags::new(),
                    call_stack_used_length_pointer,
                    0,
                );
                let one = function_builder.ins().iconst(I64, 1);

                function_builder.ins().isub(call_stack_length, one)
            };
            let function_index = get_frame_function_index(
                top_call_frame_index,
                call_stack_buffer_pointer,
                &mut function_builder,
            );

            switch.emit(&mut function_builder, function_index, main_function_block);
        }

        {
            function_builder.switch_to_block(main_function_block);
            function_builder
                .ins()
                .call(main_function_reference, &[thread_context_pointer]);
            function_builder
                .ins()
                .jump(check_for_empty_call_stack_block, &[]);
        }

        {
            for (block, (_direct, stackless)) in function_blocks
                .into_iter()
                .zip(function_references.into_iter())
            {
                function_builder.switch_to_block(block);
                function_builder
                    .ins()
                    .call(stackless, &[thread_context_pointer]);
                function_builder
                    .ins()
                    .jump(check_for_empty_call_stack_block, &[]);
            }
        }

        {
            function_builder.switch_to_block(return_block);
            function_builder.append_block_param(return_block, ThreadResult::CRANELIFT_TYPE);

            let return_thread_status = function_builder.block_params(return_block)[0];

            function_builder.ins().nop();
            function_builder.ins().return_(&[return_thread_status]);
        }

        function_builder.seal_all_blocks();
        function_builder.finalize();
        self.module
            .define_function(loop_function_id, &mut context)
            .map_err(|error| {
                if let ModuleError::Compilation(CodegenError::Verifier(errors)) = error {
                    let message = errors
                        .0
                        .iter()
                        .map(|error| format!("\n{error}"))
                        .collect::<String>();

                    JitError::CraneliftModuleError { message }
                } else {
                    JitError::CraneliftModuleError {
                        message: error.to_string(),
                    }
                }
            })?;
        self.module
            .finalize_definitions()
            .map_err(|error| JitError::CraneliftModuleError {
                message: error.to_string(),
            })?;

        Ok(self.module.get_finalized_function(loop_function_id))
    }

    fn emit_jump(
        &self,
        ip: usize,
        jump_distance: isize,
        function_builder: &mut FunctionBuilder,
        instruction_blocks: &[Block],
    ) -> Result<(), JitError> {
        let target_ip = ip as isize + jump_distance;

        if target_ip < 0 {
            return Err(JitError::JumpTargetOutOfBounds {
                target_instruction_pointer: target_ip,
                total_instruction_count: instruction_blocks.len(),
            });
        }

        let target_ip = target_ip as usize;

        if target_ip >= instruction_blocks.len() {
            return Err(JitError::JumpTargetOutOfBounds {
                target_instruction_pointer: target_ip as isize,
                total_instruction_count: instruction_blocks.len(),
            });
        }

        function_builder
            .ins()
            .jump(instruction_blocks[target_ip], &[]);

        Ok(())
    }

    fn set_register(
        &self,
        register_index: usize,
        value: CraneliftValue,
        frame_base_address: CraneliftValue,
        function_builder: &mut FunctionBuilder,
    ) -> Result<(), JitError> {
        let relative_index = function_builder.ins().iconst(I64, register_index as i64);
        let byte_offset = function_builder
            .ins()
            .imul_imm(relative_index, size_of::<Register>() as i64);
        let address = function_builder.ins().iadd(frame_base_address, byte_offset);

        function_builder
            .ins()
            .store(MemFlags::new(), value, address, 0);

        Ok(())
    }

    fn declare_imported_function(
        &mut self,
        function_builder: &mut FunctionBuilder,
        name: &str,
        signature: Signature,
    ) -> Result<FuncRef, JitError> {
        let function_id = self
            .module
            .declare_function(name, Linkage::Import, &signature)
            .map_err(|error| JitError::CraneliftModuleError {
                message: format!("Failed to declare {name} function: {error}"),
            })?;
        let function_reference = self
            .module
            .declare_func_in_func(function_id, function_builder.func);

        Ok(function_reference)
    }
}

#[derive(Clone, Copy)]
struct FunctionIds {
    direct: FuncId,
    stackless: FuncId,
}

pub type JitLogic = fn(&mut ThreadContext) -> ThreadResult;

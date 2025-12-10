mod instruction_compiler;

use std::mem::{offset_of, transmute};

use cranelift::{
    codegen::ir::InstBuilder,
    prelude::{
        AbiParam, Configurable, FunctionBuilder, FunctionBuilderContext, MemFlags, Signature,
        isa::CallConv,
        settings::{self, Flags},
        types::I64,
    },
};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module};
use rustc_hash::FxHashSet;
use tracing::Level;

use crate::{
    dust_crate::Program,
    instruction::Operation,
    jit_vm::{
        JitError, Register, ffi_functions::*,
        jit_compiler::instruction_compiler::InstructionCompiler, thread::ThreadContext,
    },
    prototype::Prototype,
};

pub struct JitCompiler<'a> {
    module: JITModule,
    program: &'a Program,
    continuation_signature: Signature,
    function_ids: Vec<FuncId>,
}

impl<'a> JitCompiler<'a> {
    pub fn new(program: &'a Program) -> Self {
        let mut settings_builder = settings::builder();

        settings_builder
            .set("preserve_frame_pointers", "true")
            .expect("Failed to configure JIT frame pointers");

        let flags = Flags::new(settings_builder);
        let isa = cranelift_native::builder()
            .expect("Failed to create native Cranelift ISA builder")
            .finish(flags)
            .expect("Failed to finish Cranelift ISA");

        let mut builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());

        builder
            .symbol("allocate_list", allocate_list as *const u8)
            .symbol("insert_into_list", insert_into_list as *const u8)
            .symbol("get_from_list", get_from_list as *const u8)
            .symbol("compare_lists_equal", compare_lists_equal as *const u8)
            .symbol(
                "compare_lists_less_than",
                compare_lists_less_than as *const u8,
            )
            .symbol(
                "compare_lists_less_than_equal",
                compare_lists_less_than_equal as *const u8,
            )
            .symbol("allocate_string", allocate_string as *const u8)
            .symbol("concatenate_strings", concatenate_strings as *const u8)
            .symbol(
                "concatenate_character_string",
                concatenate_character_string as *const u8,
            )
            .symbol(
                "concatenate_string_character",
                concatenate_string_character as *const u8,
            )
            .symbol(
                "concatenate_characters",
                concatenate_characters as *const u8,
            )
            .symbol("compare_strings_equal", compare_strings_equal as *const u8)
            .symbol(
                "compare_strings_less_than",
                compare_strings_less_than as *const u8,
            )
            .symbol(
                "compare_strings_less_than_equal",
                compare_strings_less_than_equal as *const u8,
            )
            .symbol("integer_to_string", integer_to_string as *const u8)
            .symbol("read_line", read_line as *const u8)
            .symbol("write_line_integer", write_line_integer as *const u8)
            .symbol("write_line_string", write_line_string as *const u8)
            .symbol("byte_power", byte_power as *const u8)
            .symbol("integer_power", integer_power as *const u8)
            .symbol("float_power", float_power as *const u8);

        #[cfg(debug_assertions)]
        builder.symbol("log_operation_and_ip", log_operation_and_ip as *const u8);

        let module = JITModule::new(builder);

        let continuation_signature = {
            let pointer_type = module.isa().pointer_type();
            let mut signature = Signature::new(CallConv::Tail);

            signature.params.push(AbiParam::new(pointer_type)); // ThreadContext
            signature.params.push(AbiParam::new(I64)); // Return value
            signature.returns.push(AbiParam::new(I64)); // Return value

            signature
        };

        Self {
            module,
            program,
            continuation_signature,
            function_ids: vec![FuncId::from_u32(0); program.prototypes.len()],
        }
    }

    pub fn compile(&mut self) -> Result<JitLogic, JitError> {
        let span = tracing::span!(Level::INFO, "JIT_Compiler");
        let _enter = span.enter();

        self.compile_program()
    }

    fn compile_program(&mut self) -> Result<JitLogic, JitError> {
        let compile_order = get_compile_order(self.program);

        for index in compile_order {
            self.function_ids[index] = self.compile_prototype(index)?;
        }

        self.module
            .finalize_definitions()
            .map_err(|error| JitError::CraneliftModuleError {
                error: Box::new(error),
                cranelift_ir: String::new(),
            })?;

        let main_function_id = self.function_ids[0];
        let program_function_pointer = self.module.get_finalized_function(main_function_id);
        let jit_logic = unsafe { transmute::<*const u8, JitLogic>(program_function_pointer) };

        Ok(jit_logic)
    }

    fn compile_prototype(&mut self, prototype_index: usize) -> Result<FuncId, JitError> {
        let prototype =
            self.program
                .prototypes
                .get(prototype_index)
                .ok_or(JitError::MissingPrototype {
                    index: prototype_index,
                    total: self.program.prototypes.len(),
                })?;

        let segments = self.create_execution_segments(prototype)?;

        for segment in &segments {
            self.compile_segment(segment, prototype)?;
        }

        let entry_function_id = segments.first().unwrap().function_id;

        Ok(entry_function_id)
    }

    fn compile_segment(
        &mut self,
        segment: &ExecutionSegment,
        prototype: &Prototype,
    ) -> Result<(), JitError> {
        let mut context = self.module.make_context();
        let mut builder_context = FunctionBuilderContext::new();

        context.func.signature = if segment.is_entry {
            self.entry_signature(prototype)
        } else {
            self.continuation_signature.clone()
        };

        let mut builder = FunctionBuilder::new(&mut context.func, &mut builder_context);
        let entry_block = {
            let block = builder.create_block();

            builder.append_block_params_for_function_params(block);
            builder.switch_to_block(block);

            block
        };
        let instruction_blocks = {
            let block_count = (segment.start_ip..segment.end_ip).len();
            let mut blocks = Vec::with_capacity(block_count);

            for _ in 0..block_count {
                blocks.push(builder.create_block());
            }

            blocks
        };

        let pointer_type = self.module.isa().pointer_type();
        let parameters = builder.block_params(entry_block).to_vec();
        let thread_context = parameters[0];
        let register_buffer_pointer = builder.ins().load(
            pointer_type,
            MemFlags::new(),
            thread_context,
            offset_of!(ThreadContext, register_buffer_pointer) as i32,
        );
        let register_tag_buffer_pointer = builder.ins().load(
            pointer_type,
            MemFlags::new(),
            thread_context,
            offset_of!(ThreadContext, register_tag_buffer_pointer) as i32,
        );
        let continuation_buffer = builder.ins().load(
            pointer_type,
            MemFlags::new(),
            thread_context,
            offset_of!(ThreadContext, continuation_buffer_pointer) as i32,
        );
        let base_register_index = if prototype.index == 0 {
            builder.ins().iconst(pointer_type, 0)
        } else {
            let continuations_used = builder.ins().load(
                I64,
                MemFlags::new(),
                thread_context,
                offset_of!(ThreadContext, continuations_used) as i32,
            );
            let last_index = builder.ins().iadd_imm(continuations_used, -1);
            let last_continuation_offset = builder
                .ins()
                .imul_imm(last_index, size_of::<Continuation>() as i64);
            let last_continuation_address = builder
                .ins()
                .iadd(continuation_buffer, last_continuation_offset);

            builder.ins().load(
                I64,
                MemFlags::new(),
                last_continuation_address,
                offset_of!(Continuation, base_register_index) as i32,
            )
        };

        let mut ssa_registers = {
            let mut variables = Vec::with_capacity(prototype.register_count as usize);

            if segment.is_entry {
                let argument_count = prototype.function_type.value_parameters.len();

                for argument_value in parameters.into_iter().skip(1) {
                    let variable = builder.declare_var(I64);

                    builder.def_var(variable, argument_value);
                    variables.push(variable);
                }

                for _ in argument_count..prototype.register_count as usize {
                    variables.push(builder.declare_var(I64));
                }

                // TODO: Check the capacity of the register stack and grow if necessary
            } else {
                for register_index in 0..prototype.register_count {
                    let absolute_register_index = builder
                        .ins()
                        .iadd_imm(base_register_index, register_index as i64);
                    let offset = builder
                        .ins()
                        .imul_imm(absolute_register_index, size_of::<Register>() as i64);
                    let address = builder.ins().iadd(register_buffer_pointer, offset);
                    let value = builder.ins().load(I64, MemFlags::new(), address, 0);
                    let variable = builder.declare_var(I64);

                    builder.def_var(variable, value);
                    variables.push(variable);
                }

                if let Some(call_destination) = segment.call_destination {
                    let return_value = parameters[1];
                    let variable = variables[call_destination as usize];

                    builder.def_var(variable, return_value);
                }
            }

            variables
        };

        builder.ins().jump(instruction_blocks[0], &[]);

        let mut instruction_compiler = InstructionCompiler {
            prototype,
            segment,
            instruction_blocks: &instruction_blocks,
            constants: &self.program.constants,
            ssa_registers: &mut ssa_registers,
            register_buffer_pointer,
            register_tag_buffer_pointer,
            base_register_index,
            continuation_signature: &self.continuation_signature,
            function_ids: &self.function_ids,
            pointer_type,
            thread_context,
            module: &mut self.module,
        };

        for (block_index, ip) in (segment.start_ip..segment.end_ip).enumerate() {
            instruction_compiler.compile(ip, block_index, &mut builder)?;
        }

        builder.seal_all_blocks();
        builder.finalize();

        self.module
            .define_function(segment.function_id, &mut context)
            .map_err(|error| JitError::CraneliftModuleError {
                error: Box::new(error),
                cranelift_ir: context.func.display().to_string(),
            })?;
        self.module.clear_context(&mut context);

        Ok(())
    }

    fn create_execution_segments(
        &mut self,
        prototype: &Prototype,
    ) -> Result<Vec<ExecutionSegment>, JitError> {
        let mut segments = Vec::new();
        let mut current_start = 0;
        let mut continuation_function = None;
        let mut pending_call_destination: Option<u16> = None;
        let mut create_segment =
            |ip: usize, pending_call_destination: &mut Option<u16>| -> Result<(), JitError> {
                let is_entry = current_start == 0;
                let function_id = if is_entry {
                    let name = format!("proto_{}_entry", prototype.index);
                    let signature = self.entry_signature(prototype);

                    self.module
                        .declare_function(&name, Linkage::Local, &signature)
                        .map_err(|error| JitError::CraneliftModuleError {
                            error: Box::new(error),
                            cranelift_ir: String::new(),
                        })?
                } else {
                    let name = format!("proto_{}_ip_{}", prototype.index, ip);

                    self.module
                        .declare_function(&name, Linkage::Local, &self.continuation_signature)
                        .map_err(|error| JitError::CraneliftModuleError {
                            error: Box::new(error),
                            cranelift_ir: String::new(),
                        })?
                };

                let call_destination = pending_call_destination.take();

                segments.push(ExecutionSegment {
                    start_ip: current_start,
                    end_ip: ip + 1,
                    function_id,
                    is_entry,
                    continuation_function,
                    call_destination,
                });

                current_start = ip + 1;
                continuation_function = Some(function_id);

                Ok(())
            };

        for (ip, instruction) in prototype.instructions.iter().enumerate() {
            if instruction.operation() == Operation::CALL {
                create_segment(ip, &mut pending_call_destination)?;

                pending_call_destination = Some(instruction.a_field());
            }
        }

        create_segment(
            prototype.instructions.len() - 1,
            &mut pending_call_destination,
        )?;

        for index in 1..segments.len() {
            let right_function_id = segments[index].function_id;
            let left = &mut segments[index - 1];

            left.continuation_function = Some(right_function_id);
        }

        Ok(segments)
    }

    fn entry_signature(&self, prototype: &Prototype) -> Signature {
        let pointer_type = self.module.isa().pointer_type();
        let mut signature = Signature::new(CallConv::Tail);

        signature.params.push(AbiParam::new(pointer_type)); // ThreadContext

        for _ in 0..prototype.function_type.value_parameters.len() {
            signature.params.push(AbiParam::new(I64));
        }

        signature.returns.push(AbiParam::new(I64)); // Return value

        signature
    }
}

pub type JitLogic = extern "C" fn(&mut ThreadContext) -> i64;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Continuation {
    pub function: *const u8,
    pub base_register_index: usize,
}

#[derive(Clone, Copy, Debug)]
struct ExecutionSegment {
    function_id: FuncId,
    start_ip: usize,
    end_ip: usize,
    is_entry: bool,
    continuation_function: Option<FuncId>,
    call_destination: Option<u16>,
}

fn get_compile_order(program: &Program) -> Vec<usize> {
    fn depth_first_search(
        caller_index: usize,
        edges: &[FxHashSet<usize>],
        visited: &mut [bool],
        order: &mut Vec<usize>,
    ) {
        if visited[caller_index] {
            return;
        }

        visited[caller_index] = true;

        for callee_index in &edges[caller_index] {
            depth_first_search(*callee_index, edges, visited, order);
        }

        order.push(caller_index);
    }

    let prototype_count = program.prototypes.len();
    let mut edges = vec![FxHashSet::default(); prototype_count];

    for (caller_index, prototype) in program.prototypes.iter().enumerate() {
        for instruction in &prototype.instructions {
            if instruction.operation() == Operation::CALL {
                let callee_index = instruction.b_field() as usize;

                if callee_index < prototype_count {
                    edges[caller_index].insert(callee_index);
                }
            }
        }
    }

    let mut order = Vec::with_capacity(prototype_count);
    let mut visited = vec![false; prototype_count];

    depth_first_search(0, &edges, &mut visited, &mut order);

    order
}

mod instruction_compiler;

use std::mem::{offset_of, transmute};

use cranelift::{
    codegen::ir::InstBuilder,
    prelude::{
        AbiParam, FunctionBuilder, FunctionBuilderContext, MemFlags, Signature,
        Value as CraneliftValue, Variable, isa::CallConv, types::I64,
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
}

impl<'a> JitCompiler<'a> {
    pub fn new(program: &'a Program) -> Self {
        let mut builder = JITBuilder::new(cranelift_module::default_libcall_names()).unwrap();

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

            signature
        };

        Self {
            module,
            program,
            continuation_signature,
        }
    }

    pub fn compile(&mut self) -> Result<JitLogic, JitError> {
        let span = tracing::span!(Level::INFO, "JIT_Compiler");
        let _enter = span.enter();

        self.compile_program()
    }

    fn compile_program(&mut self) -> Result<JitLogic, JitError> {
        let compile_order = get_compile_order(self.program);
        let mut main_function_id = FuncId::from_u32(0);

        for index in compile_order {
            let function_id = self.compile_prototype(index)?;

            if index == 0 {
                main_function_id = function_id;
            }
        }

        self.module
            .finalize_definitions()
            .map_err(|error| JitError::CraneliftModuleError {
                error: Box::new(error),
                cranelift_ir: String::new(),
            })?;

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

        let entry_function_id = segments.iter().last().unwrap().function_id;

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
            let block_count = segment.end_ip - segment.start_ip;
            let mut blocks = Vec::with_capacity(block_count);

            for _ in 0..block_count {
                blocks.push(builder.create_block());
            }

            blocks
        };

        let pointer_type = self.module.isa().pointer_type();
        let parameters = builder.block_params(entry_block).to_vec();
        let thread_context = parameters[0];
        let register_tags_buffer_pointer = builder.ins().load(
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
        let (base_register_index, continuation_function) =
            if prototype.index == 0 && segment.is_entry {
                let zero = builder.ins().iconst(pointer_type, 0);

                (zero, None)
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
                let base_register_index = builder.ins().load(
                    I64,
                    MemFlags::new(),
                    last_continuation_address,
                    offset_of!(Continuation, base_register_index) as i32,
                );
                let continuation_function = builder.ins().load(
                    pointer_type,
                    MemFlags::new(),
                    last_continuation_address,
                    offset_of!(Continuation, function) as i32,
                );

                (base_register_index, Some(continuation_function))
            };

        let mut ssa_registers = {
            let mut variables = Vec::with_capacity(prototype.register_count as usize);

            if segment.is_entry {
                let argument_count = prototype.function_type.value_parameters.len();

                for index in 0..argument_count {
                    let parameter = parameters[2 + index];
                    let variable = builder.declare_var(I64);

                    builder.def_var(variable, parameter);
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
                    let address = builder.ins().iadd(thread_context, offset);
                    let value = builder.ins().load(I64, MemFlags::new(), address, 0);
                    let variable = builder.declare_var(I64);

                    builder.def_var(variable, value);
                    variables.push(variable);
                }
            }

            variables
        };

        builder.ins().jump(instruction_blocks[0], &[]);

        let mut instruction_compiler = InstructionCompiler {
            instruction_blocks: &instruction_blocks,
            constants: &self.program.constants,
            ssa_registers: &mut ssa_registers,
            register_tags_buffer_pointer,
            base_register_index,
            continuation_function,
            continuation_signature: &self.continuation_signature,
            thread_context,
            module: &mut self.module,
        };

        for (block_index, ip) in (segment.start_ip..segment.end_ip).enumerate() {
            let instruction =
                prototype
                    .instructions
                    .get(ip)
                    .ok_or(JitError::InstructionIndexOutOfBounds {
                        instruction_index: ip,
                        total_instruction_count: prototype.instructions.len(),
                    })?;

            instruction_compiler.compile(instruction, ip, block_index, &mut builder)?;
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

        {
            for (ip, instruction) in prototype.instructions.iter().enumerate() {
                if instruction.operation() == Operation::CALL {
                    let name = format!("proto_{}_ip_{ip}", prototype.index);
                    let function_id = self
                        .module
                        .declare_function(&name, Linkage::Local, &self.continuation_signature)
                        .map_err(|error| JitError::CraneliftModuleError {
                            error: Box::new(error),
                            cranelift_ir: String::new(),
                        })?;

                    segments.push(ExecutionSegment {
                        start_ip: current_start,
                        end_ip: ip,
                        function_id,
                        is_entry: false,
                    });

                    current_start = ip + 1;
                }
            }
        }

        let entry_id = {
            let name = format!("proto_{}_entry", prototype.index);
            let signature = self.entry_signature(prototype);

            self.module
                .declare_function(&name, Linkage::Local, &signature)
                .map_err(|error| JitError::CraneliftModuleError {
                    error: Box::new(error),
                    cranelift_ir: String::new(),
                })?
        };

        segments.push(ExecutionSegment {
            start_ip: current_start,
            end_ip: prototype.instructions.len(),
            function_id: entry_id,
            is_entry: true,
        });

        Ok(segments)
    }

    fn entry_signature(&self, prototype: &Prototype) -> Signature {
        let pointer_type = self.module.isa().pointer_type();
        let mut signature = Signature::new(CallConv::Tail);

        signature.params.push(AbiParam::new(pointer_type)); // ThreadContext
        signature.params.push(AbiParam::new(pointer_type)); // Continuation

        for _ in 0..prototype.function_type.value_parameters.len() {
            signature.params.push(AbiParam::new(I64));
        }

        signature.returns.push(AbiParam::new(I64)); // Return value

        signature
    }

    fn save_ssa_registers(
        builder: &mut FunctionBuilder,
        ssa_variables: &[Variable],
        register_buffer: CraneliftValue,
        frame_base: CraneliftValue,
    ) {
        for (i, var) in ssa_variables.iter().enumerate() {
            let value = builder.use_var(*var);
            let index = builder.ins().iadd_imm(frame_base, i as i64);
            let offset = builder.ins().imul_imm(index, size_of::<Register>() as i64);
            let addr = builder.ins().iadd(register_buffer, offset);
            builder.ins().store(MemFlags::new(), value, addr, 0);
        }
    }
}

pub type JitLogic = extern "C" fn(&mut ThreadContext) -> i64;

#[derive(Clone, Copy)]
pub struct Continuation {
    pub function: *const u8,
    pub base_register_index: usize,
}

#[derive(Clone, Copy)]
struct ExecutionSegment {
    function_id: FuncId,
    start_ip: usize,
    end_ip: usize,
    is_entry: bool,
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

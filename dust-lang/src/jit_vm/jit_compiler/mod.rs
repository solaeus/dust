mod instruction_compiler;

use std::{
    array,
    mem::{offset_of, transmute},
};

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
use cranelift_module::{FuncId, Module};
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

        Self {
            module,
            program,
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

        let mut context = self.module.make_context();

        context.func.signature = self.prototype_signature(prototype);

        let function_id = self
            .module
            .declare_function(
                &format!("proto_{}", prototype_index),
                cranelift_module::Linkage::Local,
                &context.func.signature,
            )
            .map_err(|error| JitError::CraneliftModuleError {
                error: Box::new(error),
                cranelift_ir: context.func.display().to_string(),
            })?;
        self.function_ids[prototype_index] = function_id;

        let mut builder_context = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut context.func, &mut builder_context);

        let entry_block = {
            let block = builder.create_block();

            builder.append_block_params_for_function_params(block);

            block
        };
        let instruction_blocks = {
            let mut blocks = Vec::with_capacity(prototype.instructions.len());

            for _ in 0..prototype.instructions.len() {
                blocks.push(builder.create_block());
            }

            blocks
        };

        let parameters = builder.block_params(entry_block).to_vec();
        let thread_context = parameters[0];
        let base_register_index = parameters[1];
        let function_parameters = parameters[2..].to_vec();

        debug_assert_eq!(
            function_parameters.len(),
            prototype.function_type.value_parameters.len()
        );

        builder.switch_to_block(entry_block);

        let mut ssa_registers = {
            let function_parameter_count = function_parameters.len();
            let mut variables = Vec::with_capacity(prototype.register_count as usize);

            for argument_value in function_parameters {
                let variable = builder.declare_var(I64);

                builder.def_var(variable, argument_value);
                variables.push(variable);
            }

            for _ in function_parameter_count..prototype.register_count as usize {
                variables.push(builder.declare_var(I64));
            }

            variables
        };

        let pointer_type = self.module.isa().pointer_type();
        let thread_context_fields =
            ThreadContext::get_fields(thread_context, pointer_type, &mut builder);

        // TODO: Check the current ThreadStatus and call an error function if necessary
        // TODO: Check the capacity of the register stack and grow if necessary

        builder.ins().jump(instruction_blocks[0], &[]);

        let mut instruction_compiler = InstructionCompiler {
            prototype,
            instruction_blocks: &instruction_blocks,
            function_ids: &self.function_ids,
            constants: &self.program.constants,
            ssa_registers: &mut ssa_registers,
            thread_context,
            thread_context_fields,
            base_register_index,
            pointer_type,
            module: &mut self.module,
        };

        for ip in 0..prototype.instructions.len() {
            instruction_compiler.compile(ip, &mut builder)?;
        }

        builder.seal_all_blocks();
        builder.finalize();
        self.module
            .define_function(function_id, &mut context)
            .map_err(|error| JitError::CraneliftModuleError {
                error: Box::new(error),
                cranelift_ir: context.func.display().to_string(),
            })?;
        self.module.clear_context(&mut context);

        Ok(function_id)
    }

    fn prototype_signature(&self, prototype: &Prototype) -> Signature {
        let pointer_type = self.module.isa().pointer_type();
        let mut signature = Signature::new(CallConv::Tail);

        signature.params.push(AbiParam::new(pointer_type)); // ThreadContext
        signature.params.push(AbiParam::new(I64)); // Base register index

        for _ in 0..prototype.function_type.value_parameters.len() {
            signature.params.push(AbiParam::new(I64));
        }

        signature.returns.push(AbiParam::new(I64)); // Return value

        signature
    }
}

pub type JitLogic = extern "C" fn(&mut ThreadContext, usize) -> i64;

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

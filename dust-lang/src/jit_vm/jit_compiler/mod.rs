mod instruction_compiler;

use std::{collections::HashSet, mem::transmute};

use super::thread_pool::JitPrototype;
use crate::r#type::Type;
use crate::{jit_vm::RegisterTag, prototype::Prototype};

use cranelift::{
    codegen::ir::{ArgumentPurpose, InstBuilder},
    prelude::{
        AbiParam, Configurable, FunctionBuilder, FunctionBuilderContext, MemFlags, Signature,
        settings::{self, Flags},
        types::I64,
    },
};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Module};
use rustc_hash::{FxBuildHasher, FxHashSet};

use crate::{
    dust_crate::Program,
    instruction::Operation,
    jit_vm::{
        JitError, ffi_functions::*, jit_compiler::instruction_compiler::InstructionCompiler,
        thread_pool::ThreadContext,
    },
};

pub struct JitCompiler<'a> {
    module: JITModule,
    program: &'a Program,
    main_prototype_index: u16,
    abi_function_ids: Vec<FuncId>,
}

impl<'a> JitCompiler<'a> {
    pub fn new(program: &'a Program, main_prototype_index: u16) -> Result<Self, JitError> {
        let mut settings_builder = settings::builder();

        settings_builder
            .set("opt_level", "speed")
            .expect("Failed to set Cranelift optimization level");
        settings_builder
            .set("is_pic", "false")
            .expect("Failed to set Cranelift PIC setting");

        #[cfg(debug_assertions)]
        {
            settings_builder
                .set("enable_verifier", "true")
                .expect("Failed to set Cranelift verifier setting");
            settings_builder
                .set("regalloc_checker", "true")
                .expect("Failed to set Cranelift register allocator checker setting");
        }

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
            .symbol("write_line", write_line as *const u8)
            .symbol("spawn", spawn as *const u8)
            .symbol("byte_power", byte_power as *const u8)
            .symbol("integer_power", integer_power as *const u8)
            .symbol("float_power", float_power as *const u8);

        #[cfg(debug_assertions)]
        builder.symbol("log_operation_and_ip", log_operation_and_ip as *const u8);

        let module = JITModule::new(builder);

        Ok(Self {
            module,
            program,
            main_prototype_index,
            abi_function_ids: vec![FuncId::from_u32(0); program.prototypes.len()],
        })
    }

    pub fn compile(&mut self) -> Result<(JitEntry, Vec<JitPrototype>), JitError> {
        let (compile_order, recursive_calls) = get_compile_order_and_recursive_calls(self.program);

        let mut compiled = FxHashSet::default();

        for index in compile_order {
            self.abi_function_ids[index] = self.compile_prototype(index, &recursive_calls)?;
            compiled.insert(index);
        }

        for index in 0..self.program.prototypes.len() {
            if !compiled.contains(&index) {
                self.abi_function_ids[index] = self.compile_prototype(index, &recursive_calls)?;
            }
        }

        self.module
            .finalize_definitions()
            .map_err(|error| JitError::CraneliftModuleError {
                error: Box::new(error),
                cranelift_ir: None,
            })?;

        let mut jit_prototypes = Vec::with_capacity(self.program.prototypes.len());

        for (index, func_id) in self.abi_function_ids.iter().enumerate() {
            let function_pointer = self.module.get_finalized_function(*func_id);
            let return_type = &self.program.prototypes[index].function_type.return_type;
            let return_value_tags_vec = value_tags_for_type(return_type);

            let return_kind = match return_type {
                Type::None => 0,
                Type::Struct { .. } => 2,
                _ => 1,
            };

            jit_prototypes.push(JitPrototype {
                function_pointer: function_pointer as *mut u8,
                return_value_tags_buffer: return_value_tags_vec.as_ptr(),
                return_value_count: return_value_tags_vec.len(),
                return_kind,
                return_value_tags_vec,
                is_recursive: recursive_calls.contains(&(index as u16, index as u16)),
            });
        }

        let main_function_id = self.abi_function_ids[self.main_prototype_index as usize];
        let program_function_pointer = self.module.get_finalized_function(main_function_id);
        let main_return_type = &self.program.prototypes[self.main_prototype_index as usize]
            .function_type
            .return_type;

        let entry = match main_return_type {
            Type::None => {
                let f = unsafe { transmute::<*const u8, JitLogicNone>(program_function_pointer) };
                JitEntry::None(f)
            }
            Type::Struct { .. } => {
                let f = unsafe { transmute::<*const u8, JitLogicStruct>(program_function_pointer) };
                JitEntry::Struct(f)
            }
            _ => {
                let f = unsafe { transmute::<*const u8, JitLogicScalar>(program_function_pointer) };
                JitEntry::Scalar(f)
            }
        };

        Ok((entry, jit_prototypes))
    }

    fn compile_prototype(
        &mut self,
        prototype_index: usize,
        recursive_calls: &FxHashSet<(u16, u16)>,
    ) -> Result<FuncId, JitError> {
        let prototype =
            self.program
                .prototypes
                .get(prototype_index)
                .ok_or(JitError::MissingPrototype {
                    index: prototype_index,
                    total: self.program.prototypes.len(),
                })?;

        let mut context = self.module.make_context();
        let abi_signature = self.prototype_signature(prototype);
        let indirect_sigs = self.indirect_signatures();

        context.func.signature = abi_signature.clone();

        let abi_function_id = self
            .module
            .declare_function(
                &format!("proto_{}", prototype_index),
                cranelift_module::Linkage::Local,
                &context.func.signature,
            )
            .map_err(|error| JitError::CraneliftModuleError {
                error: Box::new(error),
                cranelift_ir: Some(context.func.display().to_string()),
            })?;
        self.abi_function_ids[prototype_index] = abi_function_id;

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
        let (struct_return_ptr, thread_context, base_register_index) =
            if matches!(prototype.function_type.return_type, Type::Struct { .. }) {
                (Some(parameters[0]), parameters[1], parameters[2])
            } else {
                (None, parameters[0], parameters[1])
            };

        builder.switch_to_block(entry_block);

        let pointer_type = self.module.isa().pointer_type();
        let thread_context_fields =
            ThreadContext::get_fields(thread_context, pointer_type, &mut builder);

        let mut ssa_registers = {
            let function_parameter_count = prototype.function_type.value_parameters.len();
            let mut variables = Vec::with_capacity(prototype.register_count as usize);

            for argument_index in 0..function_parameter_count {
                let variable = builder.declare_var(I64);
                let argument_value = builder.ins().load(
                    I64,
                    MemFlags::new(),
                    thread_context_fields.function_arguments,
                    (argument_index * 8) as i32,
                );

                builder.def_var(variable, argument_value);
                variables.push(variable);
            }

            for _ in function_parameter_count..prototype.register_count as usize {
                variables.push(builder.declare_var(I64));
            }

            variables
        };

        // TODO: Check the current ThreadStatus and call an error function if necessary
        // TODO: Check the capacity of the register stack and grow if necessary

        builder.ins().jump(instruction_blocks[0], &[]);

        let mut instruction_compiler = InstructionCompiler {
            program: self.program,
            prototype,
            instruction_blocks: &instruction_blocks,
            function_ids: &self.abi_function_ids,
            constants: &self.program.constants,
            ssa_registers: &mut ssa_registers,
            thread_context,
            thread_context_fields,
            recursive_calls,
            base_register_index,
            struct_return_ptr,
            module: &mut self.module,
            indirect_signatures: indirect_sigs,
        };

        for ip in 0..prototype.instructions.len() {
            instruction_compiler.compile(ip, &mut builder)?;
        }

        drop(instruction_compiler);

        builder.seal_all_blocks();
        builder.finalize();
        self.module
            .define_function(abi_function_id, &mut context)
            .map_err(|error| JitError::CraneliftModuleError {
                error: Box::new(error),
                cranelift_ir: Some(context.func.display().to_string()),
            })?;
        self.module.clear_context(&mut context);

        Ok(abi_function_id)
    }

    fn prototype_signature(&self, prototype: &Prototype) -> Signature {
        let pointer_type = self.module.isa().pointer_type();
        let mut signature = Signature::new(self.module.isa().default_call_conv());

        match prototype.function_type.return_type {
            Type::Struct { .. } => {
                signature.params.push(AbiParam::special(
                    pointer_type,
                    ArgumentPurpose::StructReturn,
                ));
                signature.params.push(AbiParam::new(pointer_type)); // ThreadContext
                signature.params.push(AbiParam::new(I64)); // Base register index
            }
            _ => {
                signature.params.push(AbiParam::new(pointer_type)); // ThreadContext
                signature.params.push(AbiParam::new(I64)); // Base register index

                if !matches!(prototype.function_type.return_type, Type::None) {
                    signature.returns.push(AbiParam::new(I64));
                }
            }
        }

        signature
    }

    fn indirect_signatures(&self) -> (Signature, Signature, Signature) {
        let pointer_type = self.module.isa().pointer_type();
        let cc = self.module.isa().default_call_conv();

        let mut none_sig = Signature::new(cc);
        none_sig.params.push(AbiParam::new(pointer_type));
        none_sig.params.push(AbiParam::new(I64));

        let mut scalar_sig = Signature::new(cc);
        scalar_sig.params.push(AbiParam::new(pointer_type));
        scalar_sig.params.push(AbiParam::new(I64));
        scalar_sig.returns.push(AbiParam::new(I64));

        let mut struct_sig = Signature::new(cc);
        struct_sig.params.push(AbiParam::special(pointer_type, ArgumentPurpose::StructReturn));
        struct_sig.params.push(AbiParam::new(pointer_type));
        struct_sig.params.push(AbiParam::new(I64));

        (none_sig, scalar_sig, struct_sig)
    }
}

pub type JitLogicNone = extern "C" fn(&mut ThreadContext, usize);
pub type JitLogicScalar = extern "C" fn(&mut ThreadContext, usize) -> i64;
pub type JitLogicStruct = extern "C" fn(*mut i64, &mut ThreadContext, usize);

pub enum JitEntry {
    None(JitLogicNone),
    Scalar(JitLogicScalar),
    Struct(JitLogicStruct),
}

// https://en.wikipedia.org/wiki/Tarjan%27s_strongly_connected_components_algorithm
fn get_compile_order_and_recursive_calls(program: &Program) -> (Vec<usize>, FxHashSet<(u16, u16)>) {
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

    struct Tarjan<'a> {
        edges: &'a [HashSet<usize, FxBuildHasher>],
        index_counter: usize,
        call_stack: Vec<usize>,
        on_stack: Vec<bool>,
        indices: Vec<usize>,
        lowlinks: Vec<usize>,
        scc_id: Vec<usize>,
        scc_count: usize,
        order: Vec<usize>,
    }

    impl Tarjan<'_> {
        fn visit(&mut self, node: usize) {
            self.indices[node] = self.index_counter;
            self.lowlinks[node] = self.index_counter;
            self.index_counter += 1;
            self.call_stack.push(node);
            self.on_stack[node] = true;

            for &neighbor in &self.edges[node] {
                if self.indices[neighbor] == usize::MAX {
                    self.visit(neighbor);
                    self.lowlinks[node] = self.lowlinks[node].min(self.lowlinks[neighbor]);
                } else if self.on_stack[neighbor] {
                    self.lowlinks[node] = self.lowlinks[node].min(self.indices[neighbor]);
                }
            }

            if self.lowlinks[node] == self.indices[node] {
                while let Some(top) = self.call_stack.pop() {
                    self.on_stack[top] = false;
                    self.scc_id[top] = self.scc_count;
                    self.order.push(top);
                    if top == node {
                        break;
                    }
                }
                self.scc_count += 1;
            }
        }
    }

    let mut tarjan = Tarjan {
        edges: &edges,
        index_counter: 0,
        call_stack: Vec::new(),
        on_stack: vec![false; prototype_count],
        indices: vec![usize::MAX; prototype_count],
        lowlinks: vec![usize::MAX; prototype_count],
        scc_id: vec![usize::MAX; prototype_count],
        scc_count: 0,
        order: Vec::with_capacity(prototype_count),
    };

    tarjan.visit(0);

    let mut recursive_calls = HashSet::default();

    for (caller, callees) in edges.iter().enumerate() {
        for &callee in callees {
            if tarjan.scc_id[caller] == tarjan.scc_id[callee] {
                recursive_calls.insert((caller as u16, callee as u16));
            }
        }
    }

    (tarjan.order, recursive_calls)
}

fn value_tags_for_type(r#type: &Type) -> Vec<RegisterTag> {
    match r#type {
        Type::None => vec![RegisterTag::EMPTY],
        Type::Boolean
        | Type::Byte
        | Type::Character
        | Type::Float
        | Type::Integer
        | Type::Function(_) => vec![RegisterTag::SCALAR],
        Type::String | Type::List(_) => vec![RegisterTag::OBJECT],
        Type::Struct { fields, .. } => {
            let mut tags = Vec::new();

            for (_, field_type) in fields {
                tags.extend(value_tags_for_type(field_type));
            }

            tags
        }
    }
}

fn return_word_count_for_prototype(return_type: &Type) -> usize {
    value_tags_for_type(return_type).len()
}

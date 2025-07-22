use crate::{
    Chunk, Instruction, MemoryKind, OperandType, Operation, StrippedChunk,
    instruction::{Add, Jump, Less, Load, Return},
    vm::{CallFrame, Register, thread::ThreadRunner},
};
use cranelift::{codegen::ir::Function, prelude::*};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};
use std::mem::{offset_of, transmute};

#[unsafe(no_mangle)]
pub extern "C" fn load_constant(thread: &mut ThreadRunner, call: &mut CallFrame, instruction: i64) {
}

pub struct Jit {
    module: JITModule,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn set_return_value_integer(runner: *mut ThreadRunner, val: i64) {
    (*runner).return_value = Some(crate::Value::Integer(val));
}

impl Jit {
    pub fn new() -> Self {
        let mut builder = JITBuilder::new(cranelift_module::default_libcall_names()).unwrap();
        builder.symbol(
            "set_return_value_integer",
            set_return_value_integer as *const u8,
        );
        let module = JITModule::new(builder);
        Self { module }
    }

    pub fn compile(
        &mut self,
        chunk: &StrippedChunk,
    ) -> extern "C" fn(*mut ThreadRunner, *mut CallFrame) {
        let mut builder_context = FunctionBuilderContext::new();
        let mut context = self.module.make_context();
        let ptr_type = self.module.isa().pointer_type();

        context.func.signature.params.push(AbiParam::new(ptr_type));
        context.func.signature.params.push(AbiParam::new(ptr_type));

        let mut ret_sig = context.func.signature.clone();
        let mut builder = FunctionBuilder::new(&mut context.func, &mut builder_context);

        let instructions = chunk.instructions();
        let constants = chunk.constants();
        let instruction_count = instructions.len();

        let mut blocks = Vec::with_capacity(instruction_count);
        for _ in 0..instruction_count {
            blocks.push(builder.create_block());
        }
        let dummy_final_block = builder.create_block();

        ret_sig.params = vec![AbiParam::new(ptr_type), AbiParam::new(types::I64)];
        ret_sig.returns = vec![];
        let mut ret_sig = Signature::new(self.module.isa().default_call_conv());
        ret_sig.params.push(AbiParam::new(ptr_type)); // ThreadRunner*
        ret_sig.params.push(AbiParam::new(types::I64)); // integer value
        ret_sig.returns = vec![];
        let set_ret_func_id = self
            .module
            .declare_function("set_return_value_integer", Linkage::Import, &ret_sig)
            .unwrap();
        let set_ret_func_ref = self
            .module
            .declare_func_in_func(set_ret_func_id, &mut builder.func);

        let entry_block = builder.create_block();
        builder.switch_to_block(entry_block);
        builder.append_block_params_for_function_params(entry_block);
        builder.declare_var(Variable::new(0), ptr_type);
        builder.declare_var(Variable::new(1), ptr_type);
        let runner_ptr = builder.block_params(entry_block)[0];
        let frame_ptr = builder.block_params(entry_block)[1];
        builder.def_var(Variable::new(0), runner_ptr);
        builder.def_var(Variable::new(1), frame_ptr);

        builder.ins().jump(blocks[0], &[]);

        let mut processed = vec![false; instruction_count];
        let mut worklist = std::collections::VecDeque::new();
        worklist.push_back(0);
        worklist.push_back(instructions.len() - 1);

        while let Some(ip) = worklist.pop_front() {
            if processed[ip] {
                continue;
            }
            builder.switch_to_block(blocks[ip]);
            let runner_ptr = builder.use_var(Variable::new(0));
            let frame_ptr = builder.use_var(Variable::new(1));
            let instr = &instructions[ip];

            match instr.operation() {
                Operation::LOAD => {
                    let Load {
                        destination,
                        operand,
                        r#type,
                        ..
                    } = Load::from(*instr);
                    match operand.memory {
                        MemoryKind::REGISTER => {
                            let registers_field_offset = offset_of!(CallFrame, registers) as i32;
                            let registers_pointer = builder.ins().load(
                                ptr_type,
                                MemFlags::new(),
                                frame_ptr,
                                registers_field_offset,
                            );
                            let operand_index =
                                builder.ins().iconst(types::I64, operand.index as i64);
                            let source_offset = builder
                                .ins()
                                .imul_imm(operand_index, std::mem::size_of::<Register>() as i64);
                            let destination_index =
                                builder.ins().iconst(types::I64, destination.index as i64);
                            let destination_offset = builder.ins().imul_imm(
                                destination_index,
                                std::mem::size_of::<Register>() as i64,
                            );
                            let source_address =
                                builder.ins().iadd(registers_pointer, source_offset);
                            let destination_address =
                                builder.ins().iadd(registers_pointer, destination_offset);
                            let source_value =
                                builder
                                    .ins()
                                    .load(types::I64, MemFlags::new(), source_address, 0);
                            builder.ins().store(
                                MemFlags::new(),
                                source_value,
                                destination_address,
                                0,
                            );
                        }
                        MemoryKind::CONSTANT => match r#type {
                            OperandType::INTEGER => {
                                let integer_constant = match constants[operand.index].as_integer() {
                                    Some(value) => value,
                                    None => panic!(
                                        "Attempted to load a non-integer constant as integer."
                                    ),
                                };
                                let constant_value =
                                    builder.ins().iconst(types::I64, integer_constant);
                                let registers_field_offset =
                                    offset_of!(CallFrame, registers) as i32;
                                let registers_pointer = builder.ins().load(
                                    ptr_type,
                                    MemFlags::new(),
                                    frame_ptr,
                                    registers_field_offset,
                                );
                                let destination_index =
                                    builder.ins().iconst(types::I64, destination.index as i64);
                                let destination_offset = builder.ins().imul_imm(
                                    destination_index,
                                    std::mem::size_of::<Register>() as i64,
                                );
                                let destination_address =
                                    builder.ins().iadd(registers_pointer, destination_offset);
                                builder.ins().store(
                                    MemFlags::new(),
                                    constant_value,
                                    destination_address,
                                    0,
                                );
                            }
                            _ => todo!("Unhandled operand type for LOAD"),
                        },
                        _ => todo!("Unhandled memory kind for LOAD"),
                    }
                }
                Operation::ADD => {
                    let Add {
                        destination,
                        left,
                        right,
                        r#type,
                    } = Add::from(*instr);
                    match r#type {
                        OperandType::INTEGER => {
                            let registers_field_offset = offset_of!(CallFrame, registers) as i32;
                            let registers_pointer = builder.ins().load(
                                ptr_type,
                                MemFlags::new(),
                                frame_ptr,
                                registers_field_offset,
                            );
                            let left_index = builder.ins().iconst(types::I64, left.index as i64);
                            let left_offset = builder
                                .ins()
                                .imul_imm(left_index, std::mem::size_of::<Register>() as i64);
                            let left_address = builder.ins().iadd(registers_pointer, left_offset);
                            let left_value =
                                builder
                                    .ins()
                                    .load(types::I64, MemFlags::new(), left_address, 0);
                            let right_value = match right.memory {
                                MemoryKind::REGISTER => {
                                    let right_index =
                                        builder.ins().iconst(types::I64, right.index as i64);
                                    let right_offset = builder.ins().imul_imm(
                                        right_index,
                                        std::mem::size_of::<Register>() as i64,
                                    );
                                    let right_address =
                                        builder.ins().iadd(registers_pointer, right_offset);
                                    builder.ins().load(
                                        types::I64,
                                        MemFlags::new(),
                                        right_address,
                                        0,
                                    )
                                }
                                MemoryKind::CONSTANT => {
                                    let constant_value = match constants[right.index].as_integer() {
                                        Some(val) => val,
                                        None => {
                                            panic!("Attempted to use non-integer constant in ADD")
                                        }
                                    };
                                    builder.ins().iconst(types::I64, constant_value)
                                }
                                _ => todo!("Unsupported right operand for ADD"),
                            };
                            let result_value = builder.ins().iadd(left_value, right_value);
                            let destination_index =
                                builder.ins().iconst(types::I64, destination.index as i64);
                            let destination_offset = builder.ins().imul_imm(
                                destination_index,
                                std::mem::size_of::<Register>() as i64,
                            );
                            let destination_address =
                                builder.ins().iadd(registers_pointer, destination_offset);
                            builder.ins().store(
                                MemFlags::new(),
                                result_value,
                                destination_address,
                                0,
                            );
                        }
                        _ => todo!("Unhandled operand type for ADD"),
                    }
                }
                Operation::LESS => {
                    let Less {
                        comparator,
                        left,
                        right,
                        r#type,
                    } = Less::from(*instr);
                    match r#type {
                        OperandType::INTEGER => match (left.memory, right.memory) {
                            (MemoryKind::REGISTER, MemoryKind::CONSTANT) => {
                                let constant_value = match constants[right.index].as_integer() {
                                    Some(val) => val,
                                    None => panic!("Attempted to use non-integer constant in LESS"),
                                };
                                let registers_field_offset =
                                    offset_of!(CallFrame, registers) as i32;
                                let registers_pointer = builder.ins().load(
                                    ptr_type,
                                    MemFlags::new(),
                                    frame_ptr,
                                    registers_field_offset,
                                );
                                let left_offset = builder.ins().iconst(
                                    ptr_type,
                                    (left.index * std::mem::size_of::<Register>()) as i64,
                                );
                                let left_addr = builder.ins().iadd(registers_pointer, left_offset);
                                let left_value =
                                    builder
                                        .ins()
                                        .load(types::I64, MemFlags::new(), left_addr, 0);
                                let const_value = builder.ins().iconst(types::I64, constant_value);
                                let cmp_result = if comparator != 0 {
                                    builder.ins().icmp(
                                        IntCC::SignedLessThan,
                                        left_value,
                                        const_value,
                                    )
                                } else {
                                    builder.ins().icmp(
                                        IntCC::SignedGreaterThanOrEqual,
                                        left_value,
                                        const_value,
                                    )
                                };

                                let skip_ip = ip + 2; // If true, skip next instruction
                                let next_ip = ip + 1; // If false, proceed to next instruction
                                let skip_block = if skip_ip < blocks.len() {
                                    blocks[skip_ip]
                                } else {
                                    panic!(
                                        "JIT: Branch target out of bounds (skip_block) at ip={ip}"
                                    );
                                };
                                let next_block = if next_ip < blocks.len() {
                                    blocks[next_ip]
                                } else {
                                    panic!(
                                        "JIT: Branch target out of bounds (next_block) at ip={ip}",
                                    );
                                };
                                builder
                                    .ins()
                                    .brif(cmp_result, skip_block, &[], next_block, &[]);
                                if skip_ip < blocks.len() && !processed[skip_ip] {
                                    worklist.push_back(skip_ip);
                                }
                                if next_ip < blocks.len() && !processed[next_ip] {
                                    worklist.push_back(next_ip);
                                }
                            }
                            _ => todo!("Unhandled operand types for LESS"),
                        },
                        _ => todo!("Unhandled LESS type"),
                    }
                }
                Operation::JUMP => {
                    let Jump {
                        offset,
                        is_positive,
                    } = Jump::from(*instr);
                    let target_ip = if is_positive != 0 {
                        ip + offset + 1
                    } else {
                        ip - offset
                    };
                    if target_ip < blocks.len() {
                        if target_ip == ip {
                            panic!("JIT: Jump to self detected at ip={}", ip);
                        }
                        builder.ins().jump(blocks[target_ip], &[]);
                        if !processed[target_ip] {
                            worklist.push_back(target_ip);
                        }
                    } else {
                        panic!("JIT: Jump target out of bounds at ip={}", ip);
                    }
                }
                Operation::RETURN => {
                    let Return {
                        should_return_value,
                        return_value_address,
                        r#type,
                    } = Return::from(*instr);

                    if should_return_value != 0 {
                        match r#type {
                            OperandType::INTEGER => {
                                let runner_ptr = builder.use_var(Variable::new(0));
                                let frame_ptr = builder.use_var(Variable::new(1));
                                let registers_field_offset =
                                    offset_of!(CallFrame, registers) as i32;
                                let registers_pointer = builder.ins().load(
                                    ptr_type,
                                    MemFlags::new(),
                                    frame_ptr,
                                    registers_field_offset,
                                );
                                let ret_index = builder
                                    .ins()
                                    .iconst(types::I64, return_value_address.index as i64);
                                let ret_offset = builder
                                    .ins()
                                    .imul_imm(ret_index, std::mem::size_of::<Register>() as i64);
                                let ret_addr = builder.ins().iadd(registers_pointer, ret_offset);
                                let ret_value =
                                    builder.ins().load(types::I64, MemFlags::new(), ret_addr, 0);
                                builder
                                    .ins()
                                    .call(set_ret_func_ref, &[runner_ptr, ret_value]);
                                builder.ins().return_(&[]);
                            }
                            _ => {
                                todo!("RETURN with non-integer value not yet implemented");
                            }
                        }
                    } else {
                        builder.ins().return_(&[]);
                    }
                }
                _ => {
                    todo!("Unhandled operation: {:?}", instr.operation());
                }
            }
            if !matches!(
                instr.operation(),
                Operation::JUMP | Operation::RETURN | Operation::LESS
            ) {
                let next_ip = ip + 1;
                if next_ip < blocks.len() && instructions[next_ip].operation() != Operation::RETURN
                {
                    builder.ins().jump(blocks[next_ip], &[]);
                    if !processed[next_ip] {
                        worklist.push_back(next_ip);
                    }
                }
            }
            processed[ip] = true;
        }

        builder.switch_to_block(dummy_final_block);
        builder.ins().return_(&[]);
        builder.seal_all_blocks();

        let function_id = self
            .module
            .declare_anonymous_function(&context.func.signature)
            .unwrap();
        self.module
            .define_function(function_id, &mut context)
            .unwrap();
        self.module.clear_context(&mut context);
        self.module.finalize_definitions().unwrap();

        let pointer = self.module.get_finalized_function(function_id);
        unsafe {
            std::mem::transmute::<*const u8, extern "C" fn(*mut ThreadRunner, *mut CallFrame)>(
                pointer,
            )
        }
    }
}

impl Default for Jit {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct JitInstruction {
    pub logic: extern "C" fn(*mut ThreadRunner, *mut CallFrame),
}

impl JitInstruction {
    pub fn no_op() -> Self {
        extern "C" fn no_op_logic(_: *mut ThreadRunner, _: *mut CallFrame) {}
        JitInstruction { logic: no_op_logic }
    }
}

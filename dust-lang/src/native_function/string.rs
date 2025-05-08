use crate::{DustString, Instruction, TypeCode, instruction::CallNative, risky_vm::Thread};

pub fn to_string(instruction: Instruction, thread: &mut Thread) {
    let CallNative {
        destination,
        function: _,
        argument_list_index,
    } = CallNative::from(instruction);

    let current_frame = thread.current_frame();
    let current_registers = thread.current_memory();
    let (argument_index, argument_type) = current_frame
        .get_argument_list(argument_list_index)
        .0
        .first()
        .unwrap();

    let string = match *argument_type {
        TypeCode::INTEGER => {
            let integer = current_registers
                .integers
                .get(*argument_index as usize)
                .as_value();

            DustString::from(integer.to_string())
        }
        _ => unreachable!(),
    };

    *thread
        .current_memory_mut()
        .strings
        .get_mut(destination.index as usize)
        .as_value_mut() = DustString::from(string);
}

use crate::{DustString, Instruction, TypeCode, instruction::CallNative, risky_vm::Thread};

pub fn to_string(instruction: Instruction, thread: &mut Thread) {
    let CallNative {
        destination,
        function: _,
        argument_list_index,
    } = CallNative::from(instruction);

    let (argument_index, argument_type) = thread
        .current_call
        .chunk
        .argument_lists
        .get(argument_list_index as usize)
        .unwrap()
        .0
        .first()
        .unwrap();

    let string = match *argument_type {
        TypeCode::INTEGER => {
            let integer = thread
                .current_memory
                .integers
                .get(*argument_index as usize)
                .unwrap()
                .as_value();

            DustString::from(integer.to_string())
        }
        _ => unreachable!(),
    };

    *thread
        .current_memory
        .strings
        .get_mut(destination.index as usize)
        .unwrap()
        .as_value_mut() = DustString::from(string);
}

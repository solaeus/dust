use std::io::{Write, stdin, stdout};

use crate::{
    Arguments, Destination, DustString,
    instruction::AddressKind,
    panic_vm::{CallFrame, Memory},
};

pub fn read_line<const REGISTER_COUNT: usize>(
    destination: Destination,
    _: &Arguments,
    _: &mut CallFrame,
    memory: &mut Memory<REGISTER_COUNT>,
) {
    let mut buffer = String::new();
    let _ = stdin().read_line(&mut buffer);

    if destination.is_register {
        memory.registers.strings[destination.index as usize] =
            DustString::from(buffer.trim_end_matches('\n'));
    } else {
        memory.strings[destination.index as usize] =
            DustString::from(buffer.trim_end_matches('\n'));
    }
}

pub fn write_line<const REGISTER_COUNT: usize>(
    _: Destination,
    arguments: &Arguments,
    _: &mut CallFrame,
    memory: &mut Memory<REGISTER_COUNT>,
) {
    let mut stdout = stdout();

    for address in &arguments.values {
        match address.kind {
            AddressKind::STRING_REGISTER => {
                let string = &memory.registers.strings[address.index as usize];

                stdout.write_all(string.as_bytes()).unwrap();
            }
            AddressKind::STRING_MEMORY => {
                let string = &memory.strings[address.index as usize];

                stdout.write_all(string.as_bytes()).unwrap();
            }
            _ => unreachable!(),
        }
    }

    let _ = stdout.write_all(b"\n");
}

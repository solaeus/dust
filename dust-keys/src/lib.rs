use proc_macro::{TokenStream, TokenTree};
use std::fmt::Write;

use dust_compiler::instruction::{InstructionBuilder, MemoryKind, OperandType, Operation};

const OPERATIONS: [(&str, Operation); 24] = [
    ("MOVE", Operation::MOVE),
    ("REFERENCE", Operation::REFERENCE),
    ("GET", Operation::GET),
    ("SET", Operation::SET),
    ("DROP", Operation::DROP),
    ("EQUAL", Operation::EQUAL),
    ("NOT_EQUAL", Operation::NOT_EQUAL),
    ("GREATER", Operation::GREATER),
    ("GREATER_EQUAL", Operation::GREATER_EQUAL),
    ("LESS", Operation::LESS),
    ("LESS_EQUAL", Operation::LESS_EQUAL),
    ("TEST", Operation::TEST),
    ("NEGATE", Operation::NEGATE),
    ("ADD", Operation::ADD),
    ("SUBTRACT", Operation::SUBTRACT),
    ("MULTIPLY", Operation::MULTIPLY),
    ("DIVIDE", Operation::DIVIDE),
    ("MODULO", Operation::MODULO),
    ("EXPONENT", Operation::EXPONENT),
    ("CALL", Operation::CALL),
    ("CALL_NATIVE", Operation::CALL_NATIVE),
    ("JUMP", Operation::JUMP),
    ("RETURN", Operation::RETURN),
    ("CONVERT", Operation::CONVERT),
];

const OPERAND_TYPES: [(&str, OperandType); 16] = [
    ("BOOLEAN", OperandType::BOOLEAN),
    ("I_8", OperandType::I_8),
    ("I_16", OperandType::I_16),
    ("I_32", OperandType::I_32),
    ("I_64", OperandType::I_64),
    ("I_128", OperandType::I_128),
    ("U_8", OperandType::U_8),
    ("U_16", OperandType::U_16),
    ("U_32", OperandType::U_32),
    ("U_64", OperandType::U_64),
    ("U_128", OperandType::U_128),
    ("F_32", OperandType::F_32),
    ("F_64", OperandType::F_64),
    ("CHARACTER", OperandType::CHARACTER),
    ("FUNCTION", OperandType::FUNCTION),
    ("HEAP_POINTER", OperandType::HEAP_POINTER),
];

const MEMORY_KINDS: [(&str, MemoryKind); 7] = [
    ("EMPTY", MemoryKind::EMPTY),
    ("REGISTER", MemoryKind::REGISTER),
    ("DOUBLE_REGISTER", MemoryKind::DOUBLE_REGISTER),
    ("QUAD_REGISTER", MemoryKind::QUAD_REGISTER),
    ("REFERENCE", MemoryKind::REFERENCE),
    ("ENCODED", MemoryKind::ENCODED),
    ("CONSTANT", MemoryKind::CONSTANT),
];

#[proc_macro]
pub fn create_operation_dispatch(input: TokenStream) -> TokenStream {
    let mut trees = input.into_iter();
    let scrutinee = trees
        .next()
        .map(|tree| tree.to_string())
        .unwrap_or_default();

    trees.next();

    let mut rules: Vec<(String, String)> = Vec::new();

    'rules: loop {
        let mut patterns = String::new();
        let mut filled = 0;

        while filled < 2 {
            match trees.next() {
                Some(TokenTree::Ident(ident)) => {
                    patterns.push_str(&ident.to_string());
                    patterns.push(' ');
                    filled += 1;
                }
                Some(TokenTree::Punct(punct)) if punct.as_char() == '*' => {
                    patterns.push_str("* ");
                    filled += 1;
                }
                Some(_) => {}
                None => break 'rules,
            }
        }

        trees.next();
        trees.next();

        let mut path = String::new();

        for tree in trees.by_ref() {
            match tree {
                TokenTree::Punct(punct) if punct.as_char() == ',' => break,
                tree => path.push_str(&tree.to_string()),
            }
        }

        rules.push((patterns, path));
    }

    let mut arms = vec![String::new(); rules.len()];

    for (operation_name, operation) in OPERATIONS {
        for (operand_type_name, operand_type) in OPERAND_TYPES {
            let key = InstructionBuilder::new(operation)
                .operand_type(operand_type)
                .build()
                .operation_key();

            for (index, (patterns, _)) in rules.iter().enumerate() {
                let matched = patterns
                    .split_whitespace()
                    .zip([operation_name, operand_type_name])
                    .all(|(pattern, name)| pattern == "*" || pattern == name);

                if matched {
                    if !arms[index].is_empty() {
                        arms[index].push_str(" | ");
                    }

                    write!(arms[index], "{key}").unwrap();

                    break;
                }
            }
        }
    }

    let mut source = format!("match {scrutinee} {{");

    for (index, (patterns, path)) in rules.iter().enumerate() {
        if patterns.split_whitespace().all(|pattern| pattern == "*") {
            write!(source, "\n    _ => {path},").unwrap();
        } else if arms[index].is_empty() {
            return format!(
                "compile_error!(\"rule `{} => {path}` matches no operation key\")",
                patterns.trim()
            )
            .parse()
            .unwrap();
        } else {
            write!(source, "\n    {} => {path},", arms[index]).unwrap();
        }
    }

    source.push_str("\n}");

    source.parse().unwrap()
}

#[proc_macro]
pub fn create_operand_dispatch(input: TokenStream) -> TokenStream {
    let mut trees = input.into_iter();
    let scrutinee = trees
        .next()
        .map(|tree| tree.to_string())
        .unwrap_or_default();

    trees.next();

    let mut rules: Vec<(String, String)> = Vec::new();

    'rules: loop {
        let mut patterns = String::new();
        let mut filled = 0;

        while filled < 3 {
            match trees.next() {
                Some(TokenTree::Ident(ident)) => {
                    patterns.push_str(&ident.to_string());
                    patterns.push(' ');
                    filled += 1;
                }
                Some(TokenTree::Punct(punct)) if punct.as_char() == '*' => {
                    patterns.push_str("* ");
                    filled += 1;
                }
                Some(_) => {}
                None => break 'rules,
            }
        }

        trees.next();
        trees.next();

        let mut path = String::new();

        for tree in trees.by_ref() {
            match tree {
                TokenTree::Punct(punct) if punct.as_char() == ',' => break,
                tree => path.push_str(&tree.to_string()),
            }
        }

        rules.push((patterns, path));
    }

    let mut arms = vec![String::new(); rules.len()];

    for (operand_type_name, operand_type) in OPERAND_TYPES {
        for (b_memory_name, b_memory) in MEMORY_KINDS {
            for (c_memory_name, c_memory) in MEMORY_KINDS {
                let key = InstructionBuilder::new(Operation::NO_OP)
                    .operand_type(operand_type)
                    .b_memory(b_memory)
                    .c_memory(c_memory)
                    .build()
                    .operand_key();

                for (index, (patterns, _)) in rules.iter().enumerate() {
                    let matched = patterns
                        .split_whitespace()
                        .zip([operand_type_name, b_memory_name, c_memory_name])
                        .all(|(pattern, name)| pattern == "*" || pattern == name);

                    if matched {
                        if !arms[index].is_empty() {
                            arms[index].push_str(" | ");
                        }

                        write!(arms[index], "{key}").unwrap();

                        break;
                    }
                }
            }
        }
    }

    let mut source = format!("match {scrutinee} {{");

    for (index, (patterns, path)) in rules.iter().enumerate() {
        if patterns.split_whitespace().all(|pattern| pattern == "*") {
            write!(source, "\n    _ => {path},").unwrap();
        } else if arms[index].is_empty() {
            return format!(
                "compile_error!(\"rule `{} => {path}` matches no address key\")",
                patterns.trim()
            )
            .parse()
            .unwrap();
        } else {
            write!(source, "\n    {} => {path},", arms[index]).unwrap();
        }
    }

    source.push_str("\n}");

    source.parse().unwrap()
}

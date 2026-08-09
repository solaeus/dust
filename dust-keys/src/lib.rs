use proc_macro::{Delimiter, Spacing, TokenStream, TokenTree};
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

fn parse_rules(
    input: TokenStream,
    arity: usize,
) -> Result<(String, Vec<(String, String)>), String> {
    let mut trees = input.into_iter().peekable();
    let mut scrutinee = String::new();

    loop {
        match trees.next() {
            Some(TokenTree::Punct(punct)) if punct.as_char() == ',' => break,
            Some(tree) => scrutinee.push_str(&tree.to_string()),
            None => return Err("expected a scrutinee expression followed by `,`".to_string()),
        }
    }

    if scrutinee.is_empty() {
        return Err("expected a scrutinee expression before `,`".to_string());
    }

    let mut rules: Vec<(String, String)> = Vec::new();

    loop {
        let mut patterns = String::new();
        let mut filled = 0;

        while filled < arity {
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
                Some(TokenTree::Punct(punct)) if punct.as_char() == ',' => {}
                Some(tree) => {
                    return Err(format!("expected a pattern name or `*`, found `{tree}`"));
                }
                None => {
                    if filled == 0 {
                        return Ok((scrutinee, rules));
                    }

                    return Err(format!(
                        "rule `{}` needs {arity} patterns but has {filled}",
                        patterns.trim()
                    ));
                }
            }
        }

        match (trees.next(), trees.next()) {
            (Some(TokenTree::Punct(equals)), Some(TokenTree::Punct(greater)))
                if equals.as_char() == '='
                    && equals.spacing() == Spacing::Joint
                    && greater.as_char() == '>' => {}
            _ => {
                return Err(format!(
                    "expected `=>` after rule pattern `{}`",
                    patterns.trim()
                ));
            }
        }

        let path = match trees.next() {
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                if matches!(trees.peek(), Some(TokenTree::Punct(punct)) if punct.as_char() == ',') {
                    trees.next();
                }

                group.to_string()
            }
            Some(first) => {
                let mut path = first.to_string();

                for tree in trees.by_ref() {
                    match tree {
                        TokenTree::Punct(punct) if punct.as_char() == ',' => break,
                        tree => path.push_str(&tree.to_string()),
                    }
                }

                path
            }
            None => {
                return Err(format!(
                    "expected a handler after `=>` in rule `{}`",
                    patterns.trim()
                ));
            }
        };

        rules.push((patterns, path));
    }
}

fn get_pattern_range<T>(table: &[(&str, T)], pattern: &str) -> (usize, usize) {
    if pattern == "*" {
        return (0, table.len());
    }

    match table.iter().position(|(name, _)| *name == pattern) {
        Some(index) => (index, index + 1),
        None => (0, 0),
    }
}

fn assign_rule(match_arms: &mut [String], assigned_rules: &mut [bool], arm_index: usize, key: u64) {
    if assigned_rules[key as usize] {
        return;
    }

    assigned_rules[key as usize] = true;

    if !match_arms[arm_index].is_empty() {
        match_arms[arm_index].push_str(" | ");
    }

    write!(match_arms[arm_index], "{key}").unwrap();
}

fn expand(scrutinee: &str, rules: &[(String, String)], arms: &[String], key_kind: &str) -> String {
    let mut source = format!("match {scrutinee} {{");

    for (index, (patterns, path)) in rules.iter().enumerate() {
        if patterns.split_whitespace().all(|pattern| pattern == "*") {
            write!(source, "\n    _ => {{ {path} }},").unwrap();
        } else if arms[index].is_empty() {
            return format!(
                "compile_error!(\"rule `{} => {path}` matches no {key_kind}\")",
                patterns.trim()
            );
        } else {
            write!(source, "\n    {} => {{ {path} }},", arms[index]).unwrap();
        }
    }

    source.push_str("\n}");

    source
}

#[proc_macro]
pub fn operation_handler_dispatch(input: TokenStream) -> TokenStream {
    let (scrutinee, rules) = match parse_rules(input, 2) {
        Ok(parsed) => parsed,
        Err(message) => return format!("compile_error!(\"{message}\")").parse().unwrap(),
    };
    let mut match_arms = vec![String::new(); rules.len()];
    let mut assigned_rules = [false; 512];

    for (rule_index, (patterns, _)) in rules.iter().enumerate() {
        if patterns.split_whitespace().all(|pattern| pattern == "*") {
            break;
        }

        let mut words = patterns.split_whitespace();
        let (operation_start, operation_end) =
            get_pattern_range(&OPERATIONS, words.next().unwrap());
        let (operand_type_start, operand_type_end) =
            get_pattern_range(&OPERAND_TYPES, words.next().unwrap());

        for (_, operation) in &OPERATIONS[operation_start..operation_end] {
            for (_, operand_type) in &OPERAND_TYPES[operand_type_start..operand_type_end] {
                let key = InstructionBuilder::new(*operation)
                    .operand_type(*operand_type)
                    .build()
                    .operation_key();

                assign_rule(&mut match_arms, &mut assigned_rules, rule_index, key);
            }
        }
    }

    expand(&scrutinee, &rules, &match_arms, "operation key")
        .parse()
        .unwrap()
}

#[proc_macro]
pub fn operand_handler_dispatch(input: TokenStream) -> TokenStream {
    let (scrutinee, rules) = match parse_rules(input, 3) {
        Ok(parsed) => parsed,
        Err(message) => return format!("compile_error!(\"{message}\")").parse().unwrap(),
    };
    let mut match_arms = vec![String::new(); rules.len()];
    let mut assigned_rules = [false; 1024];

    for (rule_index, (patterns, _)) in rules.iter().enumerate() {
        if patterns.split_whitespace().all(|pattern| pattern == "*") {
            break;
        }

        let mut words = patterns.split_whitespace();
        let (operand_type_start, operand_type_end) =
            get_pattern_range(&OPERAND_TYPES, words.next().unwrap());
        let (b_memory_start, b_memory_end) =
            get_pattern_range(&MEMORY_KINDS, words.next().unwrap());
        let (c_memory_start, c_memory_end) =
            get_pattern_range(&MEMORY_KINDS, words.next().unwrap());

        for (_, operand_type) in &OPERAND_TYPES[operand_type_start..operand_type_end] {
            for (_, b_memory) in &MEMORY_KINDS[b_memory_start..b_memory_end] {
                for (_, c_memory) in &MEMORY_KINDS[c_memory_start..c_memory_end] {
                    let key = InstructionBuilder::new(Operation::NO_OP)
                        .operand_type(*operand_type)
                        .b_memory(*b_memory)
                        .c_memory(*c_memory)
                        .build()
                        .operand_key();

                    assign_rule(&mut match_arms, &mut assigned_rules, rule_index, key);
                }
            }
        }
    }

    expand(&scrutinee, &rules, &match_arms, "address key")
        .parse()
        .unwrap()
}

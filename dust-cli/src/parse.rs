use std::{
    fs::File,
    io::{Write, stdout},
    time::Instant,
};

use dust_lang::{
    dust_error::DustError,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    source::SourceFile,
};
use memmap2::Mmap;
use ron::ser::PrettyConfig;

use crate::{
    cli::{GlobalOptions, InputOptions, OutputOptions, ParseCommand},
    handle_source, print_times,
};

fn handle_output(
    syntax_tree: &dust_lang::syntax::SyntaxTree,
    no_output: bool,
    ron: bool,
    pretty_ron: bool,
    postcard: bool,
    trees: bool,
) {
    if !no_output {
        if ron {
            println!("{}", ron::to_string(syntax_tree).unwrap());
        } else if pretty_ron {
            println!(
                "{}",
                ron::ser::to_string_pretty(syntax_tree, PrettyConfig::new().struct_names(true))
                    .unwrap()
            );
        } else if postcard {
            let postcard = postcard::to_extend(syntax_tree, Vec::new()).unwrap();

            stdout().write_all(&postcard).unwrap();
        } else if trees {
            println!("{syntax_tree}");
        }
    }
}

pub fn handle_parse_command(command: ParseCommand, start_time: Instant) {
    let ParseCommand {
        global: GlobalOptions { log, time, name },
        input: InputOptions {
            mut eval,
            stdin,
            path,
        },
        output:
            OutputOptions {
                no_output,
                ron,
                pretty_ron,
                postcard,
            },
        trees,
    } = command;

    let mut source = handle_source(&eval, path, stdin);

    let mut errors = Vec::new();

    for (file_id, file) in source.files_iter() {
        let lexer = if file.is_utf8_validated() {
            Lexer::from_utf8(file.content_as_str())
        } else {
            Lexer::from_bytes(file.content_as_bytes())
        };
        let parser = Parser::new(file_id, lexer);
        let ParseResult {
            syntax_tree,
            errors: parse_errors,
        } = parser.parse();

        handle_output(&syntax_tree, no_output, ron, pretty_ron, postcard, trees);
        errors.extend(parse_errors);
    }

    if !errors.is_empty() {
        eprintln!("{}", DustError::parse(errors, source).report());
    }

    if time {
        let parse_time = start_time.elapsed();

        print_times(&[("Parse Time", parse_time, None)]);
    }
}

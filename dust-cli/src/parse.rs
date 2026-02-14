use std::{
    io::{Write, stdout},
    time::Instant,
};

use dust_lang::{
    dust_error::DustError,
    lexer::Lexer,
    parser::{ParseResult, Parser},
};
use ron::ser::PrettyConfig;

use crate::{
    cli::{FormatOptions, InputOptions, OutputOptions, ParseCommand},
    handle_eval, handle_source, print_times,
};

pub fn handle_parse_command(command: ParseCommand, start_time: Instant) {
    let ParseCommand {
        input: InputOptions {
            mut eval,
            stdin,
            path,
        },
        output: OutputOptions { no_output, time },
        trees,
        format:
            FormatOptions {
                debug,
                pretty_debug,
                ron,
                pretty_ron,
                json,
                pretty_json,
                postcard,
            },
    } = command;
    handle_eval(&mut eval);

    let source = handle_source(&eval, path, stdin);
    let mut errors = Vec::new();

    for (file_id, file) in source.iter() {
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

        if !parse_errors.is_empty() {
            errors.extend(parse_errors.into_iter());

            continue;
        }

        if !no_output {
            if debug {
                println!("{syntax_tree:?}");
            } else if pretty_debug {
                println!("{:#?}", syntax_tree);
            } else if ron {
                println!("{}", ron::to_string(&syntax_tree).unwrap());
            } else if pretty_ron {
                println!(
                    "{}",
                    ron::ser::to_string_pretty(
                        &syntax_tree,
                        PrettyConfig::new().struct_names(true)
                    )
                    .unwrap()
                );
            } else if json {
                println!("{}", serde_json::to_string(&syntax_tree).unwrap());
            } else if pretty_json {
                println!("{}", serde_json::to_string_pretty(&syntax_tree).unwrap());
            } else if postcard {
                let postcard = postcard::to_extend(&syntax_tree, Vec::new()).unwrap();

                stdout().write_all(&postcard).unwrap();
            } else if trees {
                println!("{syntax_tree}");
            }
        }
    }

    if !errors.is_empty() {
        eprintln!("{}", DustError::parse(errors, source).report());
    }

    if time {
        let parse_time = start_time.elapsed();

        print_times(&[("Parse Time", parse_time, None)]);
    }
}

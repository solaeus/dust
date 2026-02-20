use std::{
    io::{Write, stdout},
    path::Path,
    time::Instant,
};

use dust_lang::{
    dust_error::DustError,
    lexer::Lexer,
    parser::{ParseError, ParseResult, Parser},
    source::{Position, SourceFile},
};
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

    let mut parse_errors = Vec::new();
    let mut files_parsed = 0;

    while files_parsed < source.file_count() {
        let (file_id, file) = source.files_iter().nth(files_parsed).unwrap();

        let lexer = if file.is_utf8_validated() {
            Lexer::from_utf8(file.content_as_str())
        } else {
            Lexer::from_bytes(file.content_as_bytes())
        };
        let parser = Parser::new(file_id, lexer);
        let ParseResult {
            syntax_tree,
            errors,
            file_module_names,
        } = parser.parse();

        handle_output(&syntax_tree, no_output, ron, pretty_ron, postcard, trees);
        parse_errors.extend(errors);

        files_parsed += 1;

        for span in file_module_names {
            let parent_file = source.get_file(file_id);
            let module_name_str = parent_file.content_str(span);
            let parent_path = Path::new(parent_file.full_path())
                .parent()
                .unwrap_or_else(|| Path::new("/"));
            let module_path = parent_path.join(module_name_str).with_added_extension("ds");
            let module_file = {
                match SourceFile::base_file(module_path) {
                    Ok(file) => file,
                    Err(error) => {
                        parse_errors.push(ParseError::CannotResolveModule {
                            error,
                            position: Position::new(file_id, span),
                        });

                        continue;
                    }
                }
            };

            source.add_file(module_file);
        }
    }

    if !parse_errors.is_empty() {
        eprintln!("{}", DustError::parse(parse_errors, source).report());
    }

    if time {
        let parse_time = start_time.elapsed();

        print_times(&[("Parse Time", parse_time, None)]);
    }
}

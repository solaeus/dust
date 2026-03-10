use std::io::{Write, stdout};

use dust_lang::{
    error::Error,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    syntax::tree::SyntaxTree,
};
use ron::ser::PrettyConfig;

use crate::{
    build_source,
    cli::{GlobalOptions, InputOptions, OutputOptions, ParseCommand},
};

pub fn handle_parse_command(command: ParseCommand) {
    let ParseCommand {
        global: GlobalOptions { log: _, name: _ },
        input: InputOptions { eval, stdin, path },
        output:
            OutputOptions {
                ron,
                pretty_ron,
                postcard,
            },
        trees,
    } = command;

    let source = build_source(&eval, path, stdin);

    let mut parse_errors = Vec::new();

    for (file_id, file) in source.iter() {
        let lexer = if file.utf8_validated() {
            Lexer::from_utf8(file.content_as_str())
        } else {
            Lexer::from_bytes(file.content_as_bytes())
        };
        let parser = Parser::new(file_id, lexer);
        let ParseResult {
            syntax_tree,
            errors,
            ..
        } = parser.parse();

        handle_output(&syntax_tree, ron, pretty_ron, postcard, trees);
        parse_errors.extend(errors);
    }

    if !parse_errors.is_empty() {
        Error::with_source(parse_errors, source).print_and_exit();
    }
}

fn handle_output(
    syntax_tree: &SyntaxTree,
    ron: bool,
    pretty_ron: bool,
    postcard: bool,
    trees: bool,
) {
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

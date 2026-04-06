use std::io::{Write, stdout};

use dust_lang::{
    error::{Error, ErrorKind},
    lexer::Lexer,
    parser::{ParseResult, Parser},
    syntax::tree::SyntaxTree,
};
use ron::ser::PrettyConfig;

use crate::{
    build_source,
    cli::{InputOptions, OutputOptions, ParseCommand},
};

pub fn handle_parse_command(command: ParseCommand) {
    let ParseCommand {
        global: _,
        input: InputOptions { eval, stdin, path },
        output:
            OutputOptions {
                debug,
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

        handle_output(&syntax_tree, debug, ron, pretty_ron, postcard, trees);
        parse_errors.extend(errors.into_iter().map(ErrorKind::Parse));
    }

    if !parse_errors.is_empty() {
        Error::with_source(parse_errors, source).print_and_exit();
    }
}

fn handle_output(
    syntax_tree: &SyntaxTree,
    debug: bool,
    ron: bool,
    pretty_ron: bool,
    postcard: bool,
    trees: bool,
) {
    if debug {
        println!("{syntax_tree:#?}");
    } else if ron {
        let ron_string = ron::to_string(syntax_tree).expect("Failed to serialize program to RON");

        stdout()
            .write_all(ron_string.as_bytes())
            .expect("Failed to write RON output to stdout");
    } else if pretty_ron {
        let ron_string =
            ron::ser::to_string_pretty(syntax_tree, PrettyConfig::default().struct_names(true))
                .expect("Failed to serialize program to pretty RON");

        stdout()
            .write_all(ron_string.as_bytes())
            .expect("Failed to write pretty RON output to stdout");
    } else if postcard {
        let bytes = postcard::to_extend(syntax_tree, Vec::new())
            .expect("Failed to serialize program to Postcard");

        stdout()
            .write_all(&bytes)
            .expect("Failed to write Postcard output to stdout");
    } else if trees {
        print!("{syntax_tree}");
    }
}

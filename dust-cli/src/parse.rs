use dust_lang::{
    error::{Error, ErrorContext, ErrorKind},
    lexer::Lexer,
    parser::{ParseResult, Parser},
    syntax::SyntaxId,
};
use ron::ser::{PrettyConfig, to_string_pretty};

use crate::{build_source, cli::ParseCommand};

pub fn handle_parse_command(command: ParseCommand) {
    let ParseCommand {
        global: _,
        input,
        ron,
    } = command;

    let source = build_source(input);

    let mut syntax_trees = Vec::new();
    let mut errors = Vec::new();
    let mut next_syntax_id = SyntaxId::ROOT;

    for (source_id, file) in source.iter() {
        let lexer = if file.utf8_validated() {
            Lexer::with_validated_source(file.content_as_str())
        } else {
            Lexer::with_unvalidated_source(file.content_as_bytes())
        };
        let parser = Parser::new(source_id, next_syntax_id, lexer);
        let ParseResult {
            syntax_tree,
            errors: parse_errors,
            ..
        } = parser.parse();
        next_syntax_id = syntax_tree.next_syntax_id();

        syntax_trees.push(syntax_tree);
        errors.extend(parse_errors.into_iter().map(ErrorKind::Parse));
    }

    if !errors.is_empty() {
        Error::new(errors, ErrorContext::Source(source)).print_and_exit();
    }

    if ron {
        let config = PrettyConfig::new().compact_arrays(false).struct_names(true);

        for tree in syntax_trees {
            println!("{}", to_string_pretty(&tree, config.clone()).unwrap());
        }
    } else {
        for tree in syntax_trees {
            println!("{tree}");
        }
    }
}

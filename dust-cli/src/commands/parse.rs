use std::io::{Write, stderr, stdout};

use dust_compiler::{
    error::{Error as DustError, ErrorContext, ErrorKind},
    lexer::Lexer,
    parser::Parser,
    syntax::SyntaxId,
};
use ron::ser::{PrettyConfig, to_string_pretty};

use crate::{build_source, cli::ParseCommand, error::Error};

pub fn parse<'src>(command: ParseCommand) -> Result<(), Error<'src>> {
    let ParseCommand {
        global: _,
        input,
        ron,
    } = command;

    let source = build_source(input)?;

    let mut syntax_trees = Vec::with_capacity(source.file_count());
    let mut errors = Vec::new();

    for (code_id, file) in source.iter() {
        let lexer = if file.utf8_validated() {
            Lexer::validated(file.content_as_str())
        } else {
            Lexer::unvalidated(file.content_as_bytes())
        };
        let parser = Parser::new(code_id, lexer, &mut errors);
        let syntax_tree = parser.parse();

        syntax_trees.push(syntax_tree);
    }

    if !errors.is_empty() {
        let error_string = DustError::new(errors, ErrorContext::Source(source)).to_string();

        stderr().write_all(error_string.as_bytes())?;
    }

    if ron {
        let config = PrettyConfig::new().compact_arrays(false).struct_names(true);

        for tree in syntax_trees {
            stdout().write_all(to_string_pretty(&tree, config.clone())?.as_bytes())?;
        }
    } else {
        for tree in syntax_trees {
            stdout().write_all(tree.to_string().as_bytes())?;
        }
    }

    Ok(())
}

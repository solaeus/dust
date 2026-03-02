use crate::source::SourceFile;

pub const CORE: SourceFile = SourceFile::Embedded {
    path: "core.ds",
    content: include_bytes!("../../std/core.ds"),
    utf8_validated: true,
};

#[cfg(test)]
mod tests {
    use crate::{
        compiler::Compiler,
        lexer::Lexer,
        parser::{ParseResult, Parser},
        source::{Source, SourceFileId},
    };

    use super::*;

    #[test]
    fn core_is_valid_utf8() {
        assert!(str::from_utf8(CORE.content_as_bytes()).is_ok());
    }

    #[test]
    fn parse_core() {
        let ParseResult { errors, .. } =
            Parser::new(SourceFileId::MAIN, Lexer::from_utf8(CORE.content_as_str())).parse();

        assert!(errors.is_empty(), "{errors:#?}");
    }

    #[test]
    fn compile_core() {
        const CORE_CONSUMER: SourceFile = SourceFile::Embedded {
            path: "main.ds",
            content: b"use core; fn main() {}",
            utf8_validated: true,
        };

        let mut source = Source::new();

        source.add_file(CORE_CONSUMER);
        source.add_file(CORE);

        let compiler = Compiler::new(source);
        let _program = compiler
            .compile(None)
            .inspect_err(|error| {
                eprintln!("{error}");
            })
            .unwrap();
    }
}

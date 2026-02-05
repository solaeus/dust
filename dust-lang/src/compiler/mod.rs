mod declaration_binder;
mod emitter;
mod error;
mod resolver;
mod type_binder;

#[cfg(test)]
mod tests;

pub use emitter::Emitter;
pub use error::CompileError;
pub use resolver::{
    Declaration, DeclarationKind, ModuleKind, Resolver, Scope, ScopeId, ScopeKind, TypeId, TypeNode,
};

use tracing::{Level, span};

use crate::{
    compiler::{
        declaration_binder::DeclarationBinder, resolver::DeclarationId, type_binder::TypeBinder,
    },
    dust_crate::Program,
    dust_error::DustError,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    prototype::Prototype,
    source::{Source, SourceCode, SourceFile, SourceFileId},
    syntax::{Syntax, SyntaxReaderIterator},
};

pub const DEFAULT_PROGRAM_NAME: &str = "dust_program";

pub fn compile_main_prototype(source_code: String) -> Result<Prototype, DustError> {
    let mut source = Source::new();
    source.add_file(SourceFile {
        name: "main".to_string(),
        source_code: SourceCode::String(source_code),
    });

    let compiler = Compiler::new(source);
    let mut program = compiler.compile(None)?;

    Ok(program.prototypes.remove(0))
}

pub fn compile_prototypes(source_code: String) -> Result<Vec<Prototype>, DustError> {
    let mut source = Source::new();

    source.add_file(SourceFile {
        name: "main".to_string(),
        source_code: SourceCode::String(source_code),
    });

    let compiler = Compiler::new(source);
    let program = compiler.compile(None)?;

    Ok(program.prototypes)
}

pub struct Compiler {
    context: Resolver,
    source: Source,
    syntax: Syntax,
}

impl Compiler {
    pub fn new(source: Source) -> Self {
        Self {
            syntax: Syntax::with_capacity(source.file_count()),
            source,
            context: Resolver::new(),
        }
    }

    pub fn context(&self) -> &Resolver {
        &self.context
    }

    pub fn compile(self, name: Option<String>) -> Result<Program, DustError> {
        self.compile_with_extras(name)
            .map(|(program, _, _)| program)
    }

    pub fn compile_with_extras(
        self,
        name: Option<String>,
    ) -> Result<(Program, Source, Syntax), DustError> {
        let (
            Resolver {
                mut constants,
                prototypes,
                ..
            },
            source,
            syntax,
        ) = self.compile_inner()?;
        let name_index =
            constants.add_string(name.as_deref().unwrap_or(DEFAULT_PROGRAM_NAME).as_bytes());

        Ok((
            Program {
                name_index,
                constants,
                prototypes,
            },
            source,
            syntax,
        ))
    }

    fn compile_inner(mut self) -> Result<(Resolver, Source, Syntax), DustError> {
        let span = span!(Level::INFO, "compile");
        let _enter = span.enter();

        // Parsing phase
        {
            let span = span!(Level::INFO, "parse");
            let _enter = span.enter();

            let mut parse_errors = Vec::new();

            for (index, file) in self.source.files().iter().enumerate() {
                let file_id = SourceFileId(index as u32);
                let lexer = Lexer::new(file.source_code.as_ref());
                let parser = Parser::new(file_id, lexer);
                let ParseResult {
                    syntax_tree,
                    errors,
                } = if file_id == SourceFileId::MAIN {
                    parser.parse_main()
                } else {
                    parser.parse_file_module()
                };

                self.syntax.add_tree(syntax_tree);
                parse_errors.extend(errors);
            }

            if !parse_errors.is_empty() {
                return Err(DustError::parse(parse_errors, self.source));
            }
        }

        // Declaration binding phase
        {
            let span = span!(Level::INFO, "declare");
            let _enter = span.enter();

            let main_declaration_binder = DeclarationBinder::new(
                SourceFileId::MAIN,
                ScopeId::PROJECT,
                &self.source,
                &self.syntax,
                &mut self.context,
            );

            match main_declaration_binder.bind_main() {
                Ok(()) => (),
                Err(error) => return Err(DustError::compile(error, self.source)),
            }
        }

        // Type binding phase
        let main_function_type = {
            let span = span!(Level::INFO, "type");
            let _enter = span.enter();

            let main_type_binder = TypeBinder::new(
                SourceFileId::MAIN,
                &self.source,
                &self.syntax,
                &mut self.context,
            );

            match main_type_binder.resolve_main() {
                Ok(main_type) => main_type,
                Err(error) => return Err(DustError::compile(error, self.source)),
            }
        };

        // Emission phase
        {
            let span = span!(Level::INFO, "emit");
            let _enter = span.enter();

            self.context.prototypes.push(Prototype::default()); // Placeholder for main prototype

            let main_emitter = match Emitter::new(
                DeclarationId::MAIN,
                0,
                SourceFileId::MAIN,
                main_function_type,
                ScopeId::PROJECT,
                None,
                (&self.source, &self.syntax, &mut self.context),
            ) {
                Ok(emitter) => emitter,
                Err(error) => return Err(DustError::compile(error, self.source)),
            };

            self.context.prototypes[0] = match main_emitter.emit_main() {
                Ok(prototype) => prototype,
                Err(error) => return Err(DustError::compile(error, self.source)),
            };
        }

        Ok((self.context, self.source, self.syntax))
    }
}

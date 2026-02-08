mod declaration_binder;
mod emitter;
mod error;
mod resolver;
mod type_binder;

#[cfg(test)]
mod tests;

pub use emitter::Emitter;
pub use error::{CompileError, InternalError};
pub use resolver::{
    Declaration, DeclarationKind, DeclarationMembers, ModuleKind, Resolver, Scope, ScopeId,
    ScopeKind, Symbol, TypeId, TypeMembers, TypeNode,
};

use tracing::{Level, span};

use crate::{
    compiler::{declaration_binder::DeclarationBinder, type_binder::TypeBinder},
    dust_crate::Program,
    dust_error::DustError,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    prototype::Prototype,
    source::{Source, SourceFile, SourceFileId},
    syntax::{Syntax, SyntaxId},
};

pub const DEFAULT_PROGRAM_NAME: &str = "dust_program";

pub fn compile_main_prototype(source_code: String) -> Result<Prototype, DustError> {
    let mut source = Source::new();
    source.add_file(SourceFile::embedded("eval".to_string(), source_code));

    let compiler = Compiler::new(source);
    let mut program = compiler.compile(None)?;

    Ok(program.prototypes.remove(0))
}

pub fn compile_prototypes(source_code: String) -> Result<Vec<Prototype>, DustError> {
    let mut source = Source::new();

    source.add_file(SourceFile::embedded("eval".to_string(), source_code));

    let compiler = Compiler::new(source);
    let program = compiler.compile(None)?;

    Ok(program.prototypes)
}

pub struct Compiler {
    syntax: Syntax,
    source: Source,
    resolver: Resolver,
}

impl Compiler {
    pub fn new(source: Source) -> Self {
        Self {
            syntax: Syntax::with_capacity(source.file_count()),
            source,
            resolver: Resolver::new(),
        }
    }

    pub fn context(&self) -> &Resolver {
        &self.resolver
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
                name_id: name_index,
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
                let lexer = Lexer::new(file.full_source_bytes());
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
        let main_function_declaration_id = {
            let span = span!(Level::INFO, "declare");
            let _enter = span.enter();

            let main_declaration_binder = DeclarationBinder::new(
                ScopeId::PROJECT,
                &self.source,
                &self.syntax,
                &mut self.resolver,
            );

            match main_declaration_binder.bind_main() {
                Ok(main_declaration_id) => main_declaration_id,
                Err(error) => return Err(DustError::compile(error, self.source, self.resolver)),
            }
        };

        // Type binding phase
        let _main_function_type = {
            let span = span!(Level::INFO, "type");
            let _enter = span.enter();

            let main_type_binder = TypeBinder::new(
                SourceFileId::MAIN,
                &self.source,
                &self.syntax,
                &mut self.resolver,
            );

            match main_type_binder.bind_main() {
                Ok(main_type) => main_type,
                Err(error) => return Err(DustError::compile(error, self.source, self.resolver)),
            }
        };

        self.resolver.prototypes.push(Prototype::default()); // Placeholder for main prototype

        // Emission phase
        let main_prototype = {
            let span = span!(Level::INFO, "emit");
            let _enter = span.enter();

            let main_syntax_tree = if let Some(tree) = self.syntax.get_tree(SourceFileId::MAIN) {
                tree
            } else {
                return Err(DustError::compile(
                    CompileError::Internal(InternalError::MissingSyntaxTree(SourceFileId::MAIN)),
                    self.source,
                    self.resolver,
                ));
            };
            let main_function = if let Some(syntax_node) = main_syntax_tree.root() {
                syntax_node
            } else {
                return Err(DustError::compile(
                    CompileError::Internal(InternalError::MissingSyntaxNode(SyntaxId::ROOT)),
                    self.source,
                    self.resolver,
                ));
            };
            let main_declaration = if let Some(declaration) =
                self.resolver.get_declaration(main_function_declaration_id)
            {
                declaration
            } else {
                return Err(DustError::compile(
                    CompileError::Internal(InternalError::MissingDeclaration(
                        main_function_declaration_id,
                    )),
                    self.source,
                    self.resolver,
                ));
            };

            let main_emitter = match Emitter::new(
                main_function,
                main_function_declaration_id,
                main_declaration.scope_id,
                0,
                None,
                (&self.source, &self.syntax, &mut self.resolver),
            ) {
                Ok(emitter) => emitter,
                Err(error) => return Err(DustError::compile(error, self.source, self.resolver)),
            };

            match main_emitter.emit_main() {
                Ok(prototype) => prototype,
                Err(error) => return Err(DustError::compile(error, self.source, self.resolver)),
            }
        };

        self.resolver.prototypes[0] = main_prototype;

        Ok((self.resolver, self.source, self.syntax))
    }
}

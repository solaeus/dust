mod declaration_binder;
mod emitter;
pub mod error;
mod type_binder;

use std::path::Path;

use smallvec::SmallVec;
use tracing::{Level, span};

use crate::{
    compiler::{
        declaration_binder::DeclarationBinder, emitter::Emitter, error::CompileError,
        type_binder::TypeBinder,
    },
    constant_list::ConstantListBuilder,
    dust_error::{Error, ErrorKind},
    instruction::OperandType,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    program::Program,
    prototype::{PrototypeId, PrototypeList},
    resolver::{
        Resolver,
        declaration_graph::Visibility,
        scope_graph::{Scope, ScopeId, ScopeKind},
    },
    source::{Source, SourceFile, SourceFileId},
    syntax::{Syntax, SyntaxVisitor},
};

pub fn compile<'src>(source_files: &[(&'src str, &'src str)]) -> Result<Program, Error<'src>> {
    let mut source = Source::new();

    for (name, source_code) in source_files {
        let file = SourceFile::validated(name, source_code);

        source.add_file(file);
    }

    let compiler = Compiler::new(source);
    let program = compiler.compile(None)?;

    Ok(program)
}

pub struct Compiler<'src> {
    syntax: Syntax,
    source: Source<'src>,
    constants: ConstantListBuilder,
    resolver: Resolver,
    prototypes: PrototypeList,
}

impl<'src> Compiler<'src> {
    pub fn new(source: Source<'src>) -> Self {
        Self {
            syntax: Syntax::new(source.file_count()),
            source,
            constants: ConstantListBuilder::new(),
            resolver: Resolver::new(),
            prototypes: PrototypeList::new(),
        }
    }

    pub fn context(&self) -> &Resolver {
        &self.resolver
    }

    pub fn compile(mut self, program_name: Option<String>) -> Result<Program, Error<'src>> {
        match self.compile_inner() {
            Ok(()) => {
                let (constants, _) = self.constants.build();
                let program = Program::new(program_name, constants, self.prototypes);

                Ok(program)
            }
            Err(errors) => {
                let errors = Error::with_source_and_resolver(errors, self.source, self.resolver);

                Err(errors)
            }
        }
    }

    pub fn compile_with_extras(
        mut self,
        program_name: Option<String>,
    ) -> Result<(Program, Source<'src>, Syntax, Resolver, Vec<OperandType>), Error<'src>> {
        match self.compile_inner() {
            Ok(()) => {
                let (constants, constant_tags) = self.constants.build();
                let program = Program::new(program_name, constants, self.prototypes);

                Ok((
                    program,
                    self.source,
                    self.syntax,
                    self.resolver,
                    constant_tags,
                ))
            }
            Err(errors) => {
                let errors = Error::with_source_and_resolver(errors, self.source, self.resolver);

                Err(errors)
            }
        }
    }

    fn compile_inner(&mut self) -> Result<(), Vec<ErrorKind>> {
        let span = span!(Level::INFO, "compile");
        let _enter = span.enter();

        let mut errors = Vec::new();

        // Parsing phase
        {
            let span = span!(Level::INFO, "parse");
            let _enter = span.enter();

            let mut files_parsed = 0;

            while files_parsed < self.source.file_count() {
                let (file_id, file) = self.source.iter().nth(files_parsed).unwrap();

                let lexer = if file.is_utf8_validated() {
                    Lexer::from_utf8(file.content_as_str())
                } else {
                    Lexer::from_bytes(file.content_as_bytes())
                };
                let parser = Parser::new(file_id, lexer);
                let ParseResult {
                    syntax_tree,
                    errors: parse_errors,
                    file_module_names,
                } = parser.parse();

                self.source.set_utf8_validated(file_id);
                self.syntax.add_tree(syntax_tree).map_err(|max| {
                    panic!("The compiler expected {max} syntax trees in total.");
                });
                errors.extend(parse_errors);

                files_parsed += 1;

                for span in file_module_names {
                    let parent_file = match self.source.get_file(file_id) {
                        Ok(file) => file,
                        Err(error) => {
                            errors.push(ErrorKind::Internal(error));

                            return Err(errors);
                        }
                    };
                    let module_name_str = match parent_file.content_str(span) {
                        Ok(name) => name,
                        Err(error) => {
                            errors.push(ErrorKind::Internal(error));

                            return Err(errors);
                        }
                    };
                    let parent_path = parent_file
                        .path()
                        .and_then(|path| path.parent())
                        .unwrap_or_else(|| Path::new("."));
                    let module_path = parent_path.join(module_name_str).with_added_extension("ds");
                    let module_file = match SourceFile::file_from_path(&module_path) {
                        Ok(file) => file,
                        Err(error) => {
                            errors.push(ErrorKind::Source(error));

                            return Err(errors);
                        }
                    };

                    self.source.add_file(module_file);
                }
            }
        }

        let crate_scope_id = self.resolver.scopes.add_scope(Scope {
            kind: ScopeKind::Crate,
            parent: ScopeId::NONE,
            modules: SmallVec::new(),
            imports: SmallVec::new(),
        });
        let main_root = match self
            .syntax
            .get_tree(SourceFileId::MAIN)
            .and_then(|tree| tree.root())
        {
            Ok(root) => root,
            Err(error) => {
                errors.push(error);

                return Err(errors);
            }
        };

        // Declaration binding phase
        {
            let span = span!(Level::INFO, "declare");
            let _enter = span.enter();

            let mut declaration_binder = DeclarationBinder::new(
                &self.source,
                &self.syntax,
                &mut self.resolver,
                &mut errors,
                crate_scope_id,
            );

            match declaration_binder.visit_root(main_root) {
                Ok(()) => {}
                Err(error) => errors.push(error),
            }
        }

        // Type binding phase
        {
            let span = span!(Level::INFO, "type");
            let _enter = span.enter();

            let mut type_binder = TypeBinder::new(&self.syntax, &mut self.resolver, &mut errors);

            match type_binder.visit_root(main_root) {
                Ok(()) => {}
                Err(error) => errors.push(error),
            }
        }

        // Emission phase
        {
            let span = span!(Level::INFO, "emit");
            let _enter = span.enter();

            let _main_prototype_id = self.prototypes.reserve_slot();

            debug_assert_eq!(_main_prototype_id, PrototypeId::MAIN);

            let main_symbol_id = self.resolver.symbols.add_symbol("main");
            let (main_declaration_id, main_declaration) = match self
                .resolver
                .declarations
                .find_declaration(main_symbol_id, crate_scope_id, Visibility::Module)
            {
                Some(declaration) => declaration,
                None => {
                    errors.push(ErrorKind::Compile(CompileError::ExpectedMainFunction));

                    return Err(errors);
                }
            };
            let main_syntax_id = main_declaration.syntax.unwrap().1;
            let main_function = match self
                .syntax
                .get_tree(SourceFileId::MAIN)
                .and_then(|tree| tree.get_node(main_syntax_id))
            {
                Ok(node) => node,
                Err(error) => {
                    errors.push(error);

                    return Err(errors);
                }
            };

            match Emitter::new(
                main_function,
                Some(main_declaration_id),
                main_declaration.scope_id,
                PrototypeId::MAIN,
                None,
                (
                    &self.source,
                    &self.syntax,
                    &mut self.constants,
                    &mut self.resolver,
                    &mut self.prototypes,
                ),
            )
            .and_then(|emitter| emitter.emit())
            {
                Ok(prototype) => self.prototypes.set_slot(PrototypeId::MAIN, prototype),
                Err(error) => {
                    errors.push(error);

                    return Err(errors);
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

mod declaration_binder;
mod emitter;
pub mod error;
mod type_binder;

// #[cfg(test)]
// mod tests;

use std::{fs::File, path::Path};

use memmap2::Mmap;
use smallvec::SmallVec;
use tracing::{Level, span};

use crate::{
    compiler::{declaration_binder::DeclarationBinder, emitter::Emitter, type_binder::TypeBinder},
    constant_table::ConstantTable,
    dust_crate::Program,
    dust_error::DustError,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    project::DEFAULT_PROGRAM_PATH,
    prototype::{Prototype, PrototypeList},
    resolver::{
        Resolver,
        scope_graph::{Scope, ScopeId, ScopeKind},
        symbol_table::SymbolId,
    },
    source::{Source, SourceFile, SourceFileId, SourceIterator},
    syntax::{Syntax, SyntaxId, SyntaxVisitor},
};

pub fn compile<'src>(source_code: &'src str) -> Result<PrototypeList, DustError> {
    let mut source = Source::new();

    source.add_file(SourceFile::validated("compile", source_code));

    let compiler = Compiler::new(source);
    let program = compiler.compile(None)?;

    Ok(program.prototypes)
}

pub fn compile_main<'src>(source_code: &'src str) -> Result<Prototype, DustError> {
    let prototype = compile(source_code)?
        .into_iter()
        .next()
        .expect("The compiler failed to produce a prototype");

    Ok(prototype)
}

pub struct Compiler<'src> {
    syntax: Syntax,
    source: Source<'src>,
    constants: ConstantTable,
    resolver: Resolver,
    prototypes: PrototypeList,
}

impl<'src> Compiler<'src> {
    pub fn new(source: Source<'src>) -> Self {
        Self {
            syntax: Syntax::new(source.file_count()),
            source,
            constants: ConstantTable::new(),
            resolver: Resolver::new(),
            prototypes: PrototypeList::new(),
        }
    }

    pub fn context(&self) -> &Resolver {
        &self.resolver
    }

    pub fn compile(self, program_name: Option<String>) -> Result<Program, DustError> {
        let Compiler {
            constants,
            prototypes,
            ..
        } = self.compile_inner(&program_name)?;
        let program = Program::new(program_name, constants, prototypes);

        Ok(program)
    }

    pub fn compile_with_extras(
        self,
        program_name: Option<String>,
    ) -> Result<(Program, Source<'src>, Syntax, Resolver), DustError> {
        let Compiler {
            syntax,
            source,
            constants,
            resolver,
            prototypes,
        } = self.compile_inner(&program_name)?;
        let program = Program::new(program_name, constants, prototypes);

        Ok((program, source, syntax, resolver))
    }

    fn compile_inner(mut self, program_name: &Option<String>) -> Result<Self, DustError> {
        let span = span!(Level::INFO, "compile");
        let _enter = span.enter();

        // Parsing phase

        {
            let span = span!(Level::INFO, "parse");
            let _enter = span.enter();

            let mut parse_errors = Vec::new();
            let mut files_parsed = 0;

            while files_parsed < self.source.file_count() {
                let (file_id, file) = self.source.files_iter().nth(files_parsed).unwrap();

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

                self.syntax.add_tree(syntax_tree).map_err(|max| {
                    panic!("The compiler expected {max} syntax trees in total.");
                });
                parse_errors.extend(errors);

                files_parsed += 1;

                for span in file_module_names {
                    let parent_file = self.source.get_file(file_id)?;
                    let module_name_str = parent_file.content_str(span)?;
                    let parent_path = Path::new(parent_file.full_path())
                        .parent()
                        .unwrap_or_else(|| Path::new("/"));
                    let module_path = parent_path.join(module_name_str).with_added_extension("ds");
                    let module_file = SourceFile::base_file(module_path)?;

                    let module_file_id = self.source.add_file(module_file);
                }
            }

            if !parse_errors.is_empty() {
                return Err(DustError::parse(parse_errors, self.source));
            }
        }

        let program_symbol_id = if let Some(name) = program_name {
            self.resolver.symbols.add_symbol(name)
        } else {
            self.resolver.symbols.add_symbol(Program::DEFAULT_NAME)
        };
        let program_scope_id = self.resolver.scopes.add_scope(Scope {
            kind: ScopeKind::Project,
            parent: ScopeId::NONE,
            modules: SmallVec::new(),
            imports: SmallVec::new(),
        });
        let main_root = match self
            .syntax
            .get_tree(SourceFileId::MAIN)
            .and_then(|tree| tree.root())
        {
            Some(root) => root,
            None => {
                return self.handle_error(vec![DustError::Internal(
                    InternalError::MissingSyntaxTree(SourceFileId::MAIN),
                )]);
            }
        };

        let mut compile_errors = Vec::new();

        // Declaration binding phase
        {
            let span = span!(Level::INFO, "declare");
            let _enter = span.enter();

            let mut declaration_binder = DeclarationBinder::new(
                &mut self.source,
                &self.syntax,
                &mut self.resolver,
                program_scope_id,
            );

            match declaration_binder.visit_root(main_root) {
                Ok(()) => {}
                Err(error) => compile_errors.push(error),
            }
        }

        // Type binding phase
        {
            let span = span!(Level::INFO, "type");
            let _enter = span.enter();

            let mut type_binder = TypeBinder::new(
                SourceFileId::MAIN,
                &self.source,
                &self.syntax,
                &mut self.resolver,
                &mut compile_errors,
            );

            match type_binder.visit_root(main_root) {
                Ok(()) => {}
                Err(error) => compile_errors.push(error),
            }
        }

        // Emission phase
        let main_prototype_id = self.prototypes.reserve_slot();
        let main_prototype = {
            let span = span!(Level::INFO, "emit");
            let _enter = span.enter();

            let main_symbol_id = self.resolver.symbols.add_symbol("main");
            let (main_declaration_id, main_declaration) = match self
                .resolver
                .declarations
                .find_declaration(main_symbol_id, None, program_scope_id)
            {
                Some(declaration) => declaration,
                None => {
                    compile_errors.push(DustError::ExpectedMainFunction);

                    return self.handle_error(compile_errors);
                }
            };

            let main_module = if let Some(tree) = self.syntax.get_tree(SourceFileId::MAIN) {
                tree.root().unwrap()
            } else {
                compile_errors.push(DustError::Internal(InternalError::MissingSyntaxTree(
                    SourceFileId::MAIN,
                )));

                return self.handle_error(compile_errors);
            };
            let main_function = if let Some(syntax_node) = main_module.children().find(|node| {
                self.resolver
                    .get_declaration_binding(&node.id)
                    .is_ok_and(|bound_id| *bound_id == main_declaration_id)
            }) {
                syntax_node
            } else {
                compile_errors.push(DustError::ExpectedMainFunction);

                return self.handle_error(compile_errors);
            };

            let main_emitter = match Emitter::new(
                main_function,
                main_declaration_id,
                main_declaration.scope_id,
                main_prototype_id,
                None,
                (
                    &self.source,
                    &self.syntax,
                    &mut self.constants,
                    &mut self.resolver,
                    &mut self.prototypes,
                ),
            ) {
                Ok(emitter) => emitter,
                Err(error) => {
                    compile_errors.push(error);

                    return self.handle_error(compile_errors);
                }
            };

            match main_emitter.emit() {
                Ok(prototype) => prototype,
                Err(error) => {
                    compile_errors.push(error);

                    return self.handle_error(compile_errors);
                }
            }
        };

        self.prototypes.set_slot(main_prototype_id, main_prototype);

        Ok(self)
    }

    fn handle_error(self, errors: Vec<DustError>) -> Result<Self, DustError<'src>> {
        Err(DustError::compile(errors, self.source, self.resolver))
    }
}

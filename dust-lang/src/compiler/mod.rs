mod declaration_binder;
mod emitter;
pub mod error;
mod type_binder;

// #[cfg(test)]
// mod tests;

use smallvec::SmallVec;
use tracing::{Level, span};

use crate::{
    compiler::{
        declaration_binder::DeclarationBinder,
        emitter::Emitter,
        error::{CompileError, InternalCompileError},
        type_binder::TypeBinder,
    },
    constant_table::ConstantTable,
    dust_crate::Program,
    dust_error::DustError,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    prototype::{Prototype, PrototypeList},
    resolver::{
        Resolver,
        scope_graph::{Scope, ScopeId, ScopeKind},
        symbol_table::SymbolId,
    },
    source::{Source, SourceFile, SourceFileId},
    syntax::{Syntax, SyntaxId},
};

pub fn compile<'src>(source_code: &'src str) -> Result<PrototypeList, DustError<'src>> {
    let mut source = Source::new();

    source.add_file(SourceFile::embedded_validated("compile", source_code));

    let compiler = Compiler::new(source);
    let program = compiler.compile(None)?;

    Ok(program.prototypes)
}

pub fn compile_main<'src>(source_code: &'src str) -> Result<Prototype, DustError<'src>> {
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

    pub fn compile(self, program_name: Option<String>) -> Result<Program, DustError<'src>> {
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
    ) -> Result<(Program, Source<'src>, Syntax, Resolver), DustError<'src>> {
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

    fn compile_inner(mut self, program_name: &Option<String>) -> Result<Self, DustError<'src>> {
        let span = span!(Level::INFO, "compile");
        let _enter = span.enter();

        // Parsing phase
        {
            let span = span!(Level::INFO, "parse");
            let _enter = span.enter();

            let mut parse_errors = Vec::new();

            for (file_id, file) in self.source.iter() {
                let lexer = if file.is_utf8_validated() {
                    Lexer::from_utf8(file.content_as_str())
                } else {
                    Lexer::from_bytes(file.content_as_bytes())
                };
                let parser = Parser::new(file_id, lexer);
                let ParseResult {
                    syntax_tree,
                    errors,
                } = parser.parse();

                self.syntax.add_tree(syntax_tree).map_err(|max| {
                    panic!("The compiler expected {max} syntax trees in total.");
                });

                parse_errors.extend(errors);
            }

            if !parse_errors.is_empty() {
                return Err(DustError::parse(parse_errors, self.source));
            }
        }

        let program_symbol_id = if let Some(name) = program_name {
            self.resolver.symbols.add_named_symbol(name)
        } else {
            self.resolver.symbols.add_anonymous_symbol()
        };
        let program_scope_id = self.resolver.scopes.add_scope(Scope {
            kind: ScopeKind::Project,
            parent: ScopeId::NONE,
            modules: SmallVec::new(),
            imports: SmallVec::new(),
        });

        // Declaration binding phase
        {
            let span = span!(Level::INFO, "declare");
            let _enter = span.enter();

            let declaration_binder = DeclarationBinder::new(
                &self.source,
                &self.syntax,
                &mut self.resolver,
                program_scope_id,
            );

            match declaration_binder.bind() {
                Ok(main_declaration_id) => main_declaration_id,
                Err(error) => return self.handle_error(error),
            }
        };

        // Type binding phase
        {
            let span = span!(Level::INFO, "type");
            let _enter = span.enter();

            let type_binder = TypeBinder::new(
                SourceFileId::MAIN,
                &self.source,
                &self.syntax,
                &mut self.resolver,
            );

            match type_binder.bind() {
                Ok(()) => {}
                Err(error) => return self.handle_error(error),
            }
        }

        // Emission phase
        let main_prototype_id = self.prototypes.reserve_slot();
        let main_prototype = {
            let span = span!(Level::INFO, "emit");
            let _enter = span.enter();

            let main_symbol_id = self.resolver.symbols.add_named_symbol("main");
            let (main_declaration_id, main_declaration) = match self
                .resolver
                .declarations
                .find_declaration(main_symbol_id, None, program_scope_id)
            {
                Some(declaration) => declaration,
                None => return self.handle_error(CompileError::ExpectedMainFunction),
            };

            let main_module = if let Some(tree) = self.syntax.get_tree(SourceFileId::MAIN) {
                tree.root().unwrap()
            } else {
                return self.handle_error(CompileError::Internal(
                    InternalCompileError::MissingSyntaxTree(SourceFileId::MAIN),
                ));
            };
            let main_function = if let Some(syntax_node) = main_module.children().find(|node| {
                self.resolver
                    .get_declaration_binding(&node.id)
                    .is_ok_and(|bound_id| *bound_id == main_declaration_id)
            }) {
                syntax_node
            } else {
                return self.handle_error(CompileError::ExpectedMainFunction);
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
                Err(error) => return self.handle_error(error),
            };

            match main_emitter.emit() {
                Ok(prototype) => prototype,
                Err(error) => return self.handle_error(error),
            }
        };

        self.prototypes.set_slot(main_prototype_id, main_prototype);

        Ok(self)
    }

    fn handle_error(self, error: CompileError) -> Result<Self, DustError<'src>> {
        Err(DustError::compile(error, self.source, self.resolver))
    }
}

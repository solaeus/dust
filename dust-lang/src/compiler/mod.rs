mod declaration_binder;
mod emitter;
pub mod error;
mod type_binder;

// #[cfg(test)]
// mod tests;

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
    resolver::Resolver,
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
        } = self.compile_inner()?;
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
        } = self.compile_inner()?;
        let program = Program::new(program_name, constants, prototypes);

        Ok((program, source, syntax, resolver))
    }

    fn compile_inner(mut self) -> Result<Self, DustError<'src>> {
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

        // Declaration binding phase
        let main_function_declaration_id = {
            let span = span!(Level::INFO, "declare");
            let _enter = span.enter();

            let declaration_binder =
                DeclarationBinder::new(&self.source, &self.syntax, &mut self.resolver);

            match declaration_binder.bind_main() {
                Ok(main_declaration_id) => main_declaration_id,
                Err(error) => return self.handle_error(error),
            }
        };

        // Type binding phase
        let _main_function_type = {
            let span = span!(Level::INFO, "type");
            let _enter = span.enter();

            let type_binder = TypeBinder::new(SourceFileId::MAIN, &self.syntax, &mut self.resolver);

            match type_binder.bind_main() {
                Ok(main_type) => main_type,
                Err(error) => return self.handle_error(error),
            }
        };

        // Emission phase
        let main_prototype_id = self.prototypes.reserve_slot();
        let main_prototype = {
            let span = span!(Level::INFO, "emit");
            let _enter = span.enter();

            let main_syntax_tree = if let Some(tree) = self.syntax.get_tree(SourceFileId::MAIN) {
                tree
            } else {
                return self.handle_error(CompileError::Internal(
                    InternalCompileError::MissingSyntaxTree(SourceFileId::MAIN),
                ));
            };
            let main_function = if let Some(syntax_node) = main_syntax_tree.root() {
                syntax_node
            } else {
                return self.handle_error(CompileError::Internal(
                    InternalCompileError::MissingSyntaxNode(SyntaxId::ROOT),
                ));
            };
            let main_declaration = match self
                .resolver
                .declarations
                .get_declaration(main_function_declaration_id)
            {
                Ok(declaration) => declaration,
                Err(error) => return Err(DustError::compile(error, self.source, self.resolver)),
            };

            let main_emitter = match Emitter::new(
                main_function,
                main_function_declaration_id,
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
                Err(error) => return Err(DustError::compile(error, self.source, self.resolver)),
            };

            match main_emitter.emit_main() {
                Ok(prototype) => prototype,
                Err(error) => return Err(DustError::compile(error, self.source, self.resolver)),
            }
        };

        self.prototypes.set_slot(main_prototype_id, main_prototype);

        Ok(self)
    }

    fn handle_error(self, error: CompileError) -> Result<Self, DustError<'src>> {
        Err(DustError::compile(error, self.source, self.resolver))
    }
}

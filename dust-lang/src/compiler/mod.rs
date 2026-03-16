mod declaration_binder;
// mod emitter;
pub mod error;
// mod type_binder;

use tracing::{Level, span};

use crate::{
    compiler::{
        declaration_binder::DeclarationBinder,
        // emitter::Emitter,
        error::CompileError,
        // type_binder::TypeBinder,
    },
    constant_list::ConstantListBuilder,
    dust_type::DustType,
    error::{Error, ErrorKind},
    instruction::OperandType,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    program::Program,
    prototype::{PrototypeId, PrototypeList},
    resolver::{
        Resolver,
        declarations::{Definition, Visibility},
        scopes::{Scope, ScopeId, ScopeKind},
    },
    source::{Source, SourceFile, SourceFileId},
    syntax::{Syntax, visitor::SyntaxVisitor},
};

pub fn compile<'src>(source_files: &[(&'src str, &'src str)]) -> Result<Program, Error<'src>> {
    let mut source = Source::new();

    for (name, source_code) in source_files {
        let file = SourceFile::validated_borrowed(name, source_code);

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
            Ok(return_type) => {
                let (constants, _) = self.constants.build();
                let program = Program::new(program_name, return_type, constants, self.prototypes);

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
            Ok(return_type) => {
                let (constants, constant_tags) = self.constants.build();
                let program = Program::new(program_name, return_type, constants, self.prototypes);

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

    fn compile_inner(&mut self) -> Result<DustType, Vec<ErrorKind>> {
        let span = span!(Level::INFO, "compile");
        let _enter = span.enter();

        let mut errors = Vec::new();

        macro_rules! unwrap_or_return {
            ($result: expr) => {
                match $result {
                    Ok(value) => value,
                    Err(error) => {
                        errors.push(error.into());

                        return Err(errors);
                    }
                }
            };
        }

        // Parsing phase
        {
            let span = span!(Level::INFO, "parse");
            let _enter = span.enter();

            for (file_id, file) in self.source.iter_mut() {
                let lexer = if file.utf8_validated() {
                    Lexer::from_utf8(file.content_as_str())
                } else {
                    Lexer::from_bytes(file.content_as_bytes())
                };
                let parser = Parser::new(file_id, lexer);
                let ParseResult {
                    syntax_tree,
                    errors: parse_errors,
                    file_module_names: _,
                } = parser.parse();

                file.set_utf8_validated(true);
                self.syntax.add_tree(syntax_tree);
                errors.extend(parse_errors.into_iter().map(ErrorKind::Parse));
            }
        }

        let crate_scope_id = self.resolver.scopes.add_scope(Scope {
            kind: ScopeKind::Crate,
            parent: ScopeId::NONE,
            modules: Vec::new(),
            imports: Vec::new(),
        });
        let main_file_root = unwrap_or_return!(
            self.syntax
                .get_tree(SourceFileId::MAIN)
                .and_then(|tree| tree.root())
        );

        // Declaration binding phase
        {
            let span = span!(Level::INFO, "declare");
            let _enter = span.enter();

            let mut declaration_binder = DeclarationBinder::new(
                &self.source,
                &self.syntax,
                &mut self.resolver,
                &mut self.prototypes,
                &mut errors,
                crate_scope_id,
            );

            match declaration_binder.visit_root(main_file_root) {
                Ok(()) => {}
                Err(error) => errors.push(ErrorKind::Compile(error)),
            }
        }

        // Type binding phase
        {
            let span = span!(Level::INFO, "type");
            let _enter = span.enter();

            let _type_binder = todo!();
        }

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
        let Definition::Function { return_type_id, .. } = main_declaration.definition else {
            errors.push(ErrorKind::Compile(CompileError::ExpectedMainFunction));

            return Err(errors);
        };
        let main_syntax_id = main_declaration.syntax.unwrap().1;
        let main_function_item = unwrap_or_return!(
            self.syntax
                .get_tree(SourceFileId::MAIN)
                .and_then(|tree| tree.get_node(main_syntax_id))
        );

        // Emission phase
        {
            let span = span!(Level::INFO, "emit");
            let _enter = span.enter();

            let _main_prototype_id = self.prototypes.reserve();

            debug_assert_eq!(_main_prototype_id, PrototypeId::MAIN);

            let main_prototype = todo!();

            self.prototypes.set(PrototypeId::MAIN, main_prototype);
        }

        let main_function_return_type_id = unwrap_or_return!(
            self.resolver
                .get_external_type(return_type_id, &self.source)
        );

        if errors.is_empty() {
            Ok(main_function_return_type_id)
        } else {
            Err(errors)
        }
    }
}

mod declaration_binder;
mod emitter;
pub mod error;
pub mod resolver;
mod type_binder;
mod value_creation;

#[cfg(test)]
pub(crate) mod tests;

pub use emitter::RegisterWidth;

use smallvec::SmallVec;
use tracing::{Level, span};

use crate::{
    compiler::{
        declaration_binder::DeclarationBinder,
        emitter::Emitter,
        error::CompileError,
        resolver::{
            PrototypeId, Resolver, declarations::Definition, scopes::ScopeKind, types::TypeId,
        },
        type_binder::TypeBinder,
    },
    constants::ConstantsBuilder,
    dust_type::DustType,
    error::{Error, ErrorContext, ErrorKind},
    instruction::OperandType,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    program::Program,
    source::{Source, SourceCodeId},
    syntax::{Syntax, components::FnItem, node::SyntaxKind},
};

pub struct Compiler<'src> {
    syntax: Syntax,
    source: Source<'src>,
    constants: ConstantsBuilder,
    resolver: Resolver,
}

impl<'src> Compiler<'src> {
    pub fn new(source: Source<'src>) -> Self {
        Self {
            syntax: Syntax::with_capacity(source.file_count()),
            source,
            constants: ConstantsBuilder::new(),
            resolver: Resolver::new(),
        }
    }

    pub fn context(&self) -> &Resolver {
        &self.resolver
    }

    pub fn compile(mut self, program_name: Option<String>) -> Result<Program, Error<'src>> {
        match self.compile_inner() {
            Ok(return_type) => {
                let (constants, _) = self.constants.build();
                let program = Program::new(
                    program_name,
                    return_type,
                    constants,
                    self.resolver.into_prototypes(),
                );

                Ok(program)
            }
            Err(errors) => {
                let errors = Error::new(
                    errors,
                    ErrorContext::Full(self.source, self.syntax, Box::new(self.resolver)),
                );

                Err(errors)
            }
        }
    }

    pub fn compile_with_extras(
        mut self,
        program_name: Option<String>,
    ) -> Result<(Program, Source<'src>, Syntax, Vec<OperandType>), Error<'src>> {
        match self.compile_inner() {
            Ok(return_type) => {
                let (constants, constant_tags) = self.constants.build();
                let program = Program::new(
                    program_name,
                    return_type,
                    constants,
                    self.resolver.into_prototypes(),
                );

                Ok((program, self.source, self.syntax, constant_tags))
            }
            Err(errors) => {
                let errors = Error::new(
                    errors,
                    ErrorContext::Full(self.source, self.syntax, Box::new(self.resolver)),
                );

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

            for (source_id, file) in self.source.iter_mut() {
                let lexer = if file.utf8_validated() {
                    Lexer::with_validated_source(file.content_as_str())
                } else {
                    Lexer::with_unvalidated_source(file.content_as_bytes())
                };
                let parser = Parser::new(source_id, lexer);
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

        let crate_scope_id = self.resolver.scopes.enter_scope(ScopeKind::Module, None);

        // Declaration binding phase
        {
            let span = span!(Level::INFO, "declare");
            let _enter = span.enter();

            let main_file_root = unwrap_or_return!(
                self.syntax
                    .get_tree(SourceCodeId::MAIN)
                    .and_then(|tree| tree.root())
            );

            let mut declaration_binder = DeclarationBinder::new(
                &self.source,
                &self.syntax,
                &mut self.resolver,
                &mut errors,
                crate_scope_id,
            );

            match declaration_binder.bind_root(main_file_root) {
                Ok(()) => {}
                Err(error) => errors.push(ErrorKind::Compile(error)),
            }

            self.resolver.scopes.exit_scope(crate_scope_id);
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        // Emission phase

        let main_symbol_id = self.resolver.symbols.add_symbol("main");
        let main_declaration_id = match self
            .resolver
            .declarations
            .find_declaration_id(main_symbol_id, crate_scope_id)
        {
            Some(declaration) => *declaration,
            None => {
                errors.push(ErrorKind::Compile(CompileError::ExpectedMainFunction));

                return Err(errors);
            }
        };
        let main_declaration = self
            .resolver
            .declarations
            .get_declaration(main_declaration_id);
        let Definition::Function { .. } = main_declaration.definition else {
            errors.push(ErrorKind::Compile(CompileError::ExpectedMainFunction));

            return Err(errors);
        };

        let main_prototype_id = self
            .resolver
            .add_monomorphized_function(main_declaration_id, SmallVec::new());

        debug_assert_eq!(main_prototype_id, PrototypeId::MAIN);

        let mut main_return_type_id = None;

        while let Some(prototype_id) = self.resolver.compilation_stack.pop() {
            match self.compile_loop(prototype_id, &mut errors) {
                Ok(type_id) => {
                    if prototype_id == PrototypeId::MAIN {
                        main_return_type_id =
                            Some(unwrap_or_return!(self.resolver.resolve_type(type_id)));
                    }
                }
                Err(()) => return Err(errors),
            }
        }

        let concrete_main_return_type_id = match main_return_type_id {
            Some(type_id) => type_id,
            None => {
                errors.push(ErrorKind::Compile(CompileError::ExpectedMainFunction));

                return Err(errors);
            }
        };

        let main_function_return_type = unwrap_or_return!(
            self.resolver
                .get_external_type(concrete_main_return_type_id)
        );

        if errors.is_empty() {
            Ok(main_function_return_type)
        } else {
            Err(errors)
        }
    }

    fn compile_loop(
        &mut self,
        prototype_id: PrototypeId,
        errors: &mut Vec<ErrorKind>,
    ) -> Result<TypeId, ()> {
        macro_rules! unwrap_or_return {
            ($result: expr) => {
                match $result {
                    Ok(value) => value,
                    Err(error) => {
                        errors.push(error.into());

                        return Err(());
                    }
                }
            };
        }

        let (declaration_id, type_arguments) = self
            .resolver
            .get_monomorphized_function(prototype_id)
            .clone();
        let declaration = self.resolver.declarations.get_declaration(declaration_id);
        let Definition::Function {
            type_parameters,
            return_type_id,
            ..
        } = declaration.definition
        else {
            errors.push(ErrorKind::Compile(
                CompileError::ExpectedFunctionDefinition(declaration_id),
            ));

            return Err(());
        };
        let (position, syntax_id) = unwrap_or_return!(declaration.syntax.ok_or(
            ErrorKind::Compile(CompileError::ExpectedSyntax {
                expected: &[SyntaxKind::BlockExpression],
            })
        ));
        let function_syntax = unwrap_or_return!(
            self.syntax
                .get_tree(position.source_id)
                .and_then(|tree| tree.read_node(syntax_id))
        );
        let Ok(FnItem {
            body: Some(body),
            value_parameters,
            ..
        }) = function_syntax.as_component()
        else {
            errors.push(ErrorKind::Compile(CompileError::ExpectedSyntax {
                expected: &[SyntaxKind::BlockExpression],
            }));

            return Err(());
        };

        self.resolver.type_parameter_map.clear();

        let type_parameter_ids = match type_parameters {
            Some(type_parameter_scope_id) => {
                self.resolver.scopes.get_members(type_parameter_scope_id)
            }
            None => &[],
        };

        if type_parameter_ids.len() != type_arguments.len() {
            errors.push(ErrorKind::Compile(
                CompileError::TypeArgumentCountMismatch {
                    expected: type_parameter_ids.len(),
                    actual: type_arguments.len(),
                },
            ));

            return Err(());
        }

        self.resolver
            .type_parameter_map
            .extend(type_parameter_ids.iter().zip(type_arguments.iter()));

        {
            let span = span!(Level::INFO, "type_bind");
            let _enter = span.enter();

            let mut type_binder = TypeBinder::new(&mut self.resolver, &self.source, errors);

            type_binder.bind_function_body(body, return_type_id);
        }

        {
            let span = span!(Level::INFO, "emit");
            let _enter = span.enter();

            let mut emitter = unwrap_or_return!(Emitter::new(
                declaration_id,
                prototype_id,
                return_type_id,
                (&self.source, &mut self.constants, &mut self.resolver),
                value_parameters,
            ));

            unwrap_or_return!(emitter.emit_function_body(body));

            let prototype = unwrap_or_return!(emitter.finish());

            self.resolver.set_prototype(prototype_id, prototype);
        }

        Ok(return_type_id)
    }
}

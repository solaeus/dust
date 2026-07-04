pub mod context;
mod declaration_resolver;
pub mod error;
mod prototype_emitter;
mod type_resolver;
mod value_creation;

#[cfg(test)]
pub(crate) mod tests;

pub use prototype_emitter::RegisterWidth;

use smallvec::SmallVec;
use tracing::{Level, span};

use crate::{
    compiler::{
        context::{Context, PrototypeId, declarations::Definition, scopes::Barrier, types::TypeId},
        declaration_resolver::DeclarationResolver,
        error::CompileError,
        prototype_emitter::PrototypeEmitter,
        type_resolver::TypeResolver,
    },
    constants::ConstantsBuilder,
    dust_type::DustType,
    error::{Error, ErrorContext, ErrorKind},
    instruction::OperandType,
    lexer::Lexer,
    parser::{ParseResult, Parser},
    program::Program,
    source::{Source, SourceCodeId},
    syntax::{Syntax, SyntaxId, components::FunctionItem, node::SyntaxKind},
};

pub struct Compiler<'src> {
    syntax: Syntax,
    source: Source<'src>,
    constants: ConstantsBuilder,
    context: Context,
}

impl<'src> Compiler<'src> {
    pub fn new(source: Source<'src>) -> Self {
        Self {
            syntax: Syntax::with_capacity(source.file_count()),
            source,
            constants: ConstantsBuilder::new(),
            context: Context::new(),
        }
    }

    pub fn compile(mut self, program_name: Option<String>) -> Result<Program, Error<'src>> {
        match self.compile_inner() {
            Ok(return_type) => {
                let (constants, _) = self.constants.build();
                let prototypes = self.context.into_prototypes();
                let program = Program::new(program_name, return_type, constants, prototypes);

                Ok(program)
            }
            Err(errors) => {
                let context = ErrorContext::Full(self.source, self.syntax, Box::new(self.context));
                let errors = Error::new(errors, context);

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
                    self.context.into_prototypes(),
                );

                Ok((program, self.source, self.syntax, constant_tags))
            }
            Err(errors) => {
                let errors = Error::new(
                    errors,
                    ErrorContext::Full(self.source, self.syntax, Box::new(self.context)),
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
                    Lexer::validated(file.content_as_str())
                } else {
                    Lexer::unvalidated(file.content_as_bytes())
                };
                let parser = Parser::new(source_id, SyntaxId::ROOT, lexer);
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

        let crate_scope_id = self.context.scopes.enter_scope(Barrier::Module, None);

        // Declaration binding phase
        {
            let span = span!(Level::INFO, "declare");
            let _enter = span.enter();

            let main_file_root = unwrap_or_return!(
                self.syntax
                    .get_tree(SourceCodeId::MAIN)
                    .and_then(|tree| tree.read_root())
            );

            let mut declaration_resolver = DeclarationResolver::new(
                &self.source,
                &self.syntax,
                &mut self.context,
                &mut errors,
                crate_scope_id,
            );

            match declaration_resolver.visit_root(main_file_root) {
                Ok(()) => {}
                Err(error) => errors.push(ErrorKind::Compile(error)),
            }

            self.context.scopes.exit_scope(crate_scope_id);
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        // Emission phase

        let main_symbol_id = self.context.symbols.add_symbol("main");
        let main_declaration_id = match self
            .context
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
            .context
            .declarations
            .get_declaration(main_declaration_id);
        let Definition::Function { .. } = main_declaration.definition else {
            errors.push(ErrorKind::Compile(CompileError::ExpectedMainFunction));

            return Err(errors);
        };

        let main_prototype_id = self
            .context
            .add_monomorphized_function(main_declaration_id, SmallVec::new());

        debug_assert_eq!(main_prototype_id, PrototypeId::MAIN);

        let mut main_return_type_id = None;

        while let Some(prototype_id) = self.context.pop_from_compilation_stack() {
            match self.compile_loop(prototype_id, &mut errors) {
                Ok(type_id) => {
                    if prototype_id == PrototypeId::MAIN {
                        main_return_type_id = Some(unwrap_or_return!(
                            self.context.get_inferred_type_id(type_id)
                        ));
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

        let main_function_return_type =
            unwrap_or_return!(self.context.get_external_type(concrete_main_return_type_id));

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

        let (declaration_id, mut type_arguments) = self
            .context
            .get_monomorphized_function(prototype_id)
            .clone();
        let declaration = self.context.declarations.get_declaration(declaration_id);
        let Definition::Function {
            parent_impl_or_trait,
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
        let Ok(FunctionItem {
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

        let type_parameter_ids = unwrap_or_return!(
            self.context
                .get_type_parameter_ids(parent_impl_or_trait, type_parameters)
        );

        while type_arguments.len() < type_parameter_ids.len() {
            let inferred_type_id = self.context.types.create_inferred_type(None);

            type_arguments.push(inferred_type_id);
        }

        self.context.type_parameter_map.clear();
        self.context
            .type_parameter_map
            .extend(type_parameter_ids.into_iter().zip(type_arguments));

        {
            let span = span!(Level::INFO, "type_resolve");
            let _enter = span.enter();

            let mut type_resolver = TypeResolver::new(&mut self.context, &self.source, errors);

            unwrap_or_return!(type_resolver.visit_function_body(body, return_type_id));
        }

        {
            let span = span!(Level::INFO, "emit");
            let _enter = span.enter();

            let mut prototype_emitter = unwrap_or_return!(PrototypeEmitter::new(
                declaration_id,
                prototype_id,
                return_type_id,
                (
                    &self.source,
                    &self.syntax,
                    &mut self.constants,
                    &mut self.context
                ),
                value_parameters,
            ));

            unwrap_or_return!(prototype_emitter.visit_function_body(body));

            let prototype = unwrap_or_return!(prototype_emitter.finish());

            self.context.set_prototype(prototype_id, prototype);
        }

        let resolved_return_type_id =
            unwrap_or_return!(self.context.get_inferred_type_id(return_type_id));

        Ok(resolved_return_type_id)
    }
}

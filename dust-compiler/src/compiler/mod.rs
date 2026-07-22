pub mod context;
mod declaration_resolver;
pub mod error;
mod prototype_emitter;
pub mod prototypes;
mod type_resolver;
mod value_creation;

#[cfg(test)]
pub(crate) mod tests;

pub use prototype_emitter::RegisterWidth;

use smallvec::SmallVec;
use tracing::{Level, span};

use crate::{
    compiler::{
        context::{Context, declarations::Definition, scopes::Barrier, types::TypeId},
        declaration_resolver::DeclarationResolver,
        error::CompileError,
        prototype_emitter::PrototypeEmitter,
        prototypes::{PrototypeId, Prototypes},
        type_resolver::TypeResolver,
    },
    constants::ConstantsBuilder,
    dust_type::DustType,
    error::{Error, ErrorContext, ErrorKind},
    instruction::OperandType,
    lexer::Lexer,
    parser::Parser,
    program::Program,
    source::{Code, CodeId, Source},
    syntax::{Syntax, components::FunctionItem, node::SyntaxKind},
};

pub struct Compiler<'src> {
    syntax: Syntax,
    source: Source<'src>,
    constants: ConstantsBuilder,
    context: Context,
    prototypes: Prototypes,
}

impl<'src> Compiler<'src> {
    pub fn new(source: Source<'src>) -> Self {
        Self {
            syntax: Syntax::with_capacity(source.file_count()),
            source,
            constants: ConstantsBuilder::new(),
            context: Context::new(),
            prototypes: Prototypes::default(),
        }
    }

    pub fn compile(mut self, program_name: String) -> Result<Program, Error<'src>> {
        match self.compile_inner() {
            Ok(return_type) => {
                let (constants, _) = self.constants.build();
                let prototypes = self.prototypes.into_prototypes();
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
        program_name: String,
    ) -> Result<(Program, Source<'src>, Syntax, Vec<OperandType>), Error<'src>> {
        match self.compile_inner() {
            Ok(return_type) => {
                let (constants, constant_tags) = self.constants.build();
                let program = Program::new(
                    program_name,
                    return_type,
                    constants,
                    self.prototypes.into_prototypes(),
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

        {
            let span = span!(Level::INFO, "parsing");
            let _enter = span.enter();

            let mut current_code_id = CodeId::MAIN;
            let mut module_names = Vec::new();
            let mut parse_errors = Vec::new();

            while let Some(position) = module_names.pop() {
                let file_name = unwrap_or_return!(self.source.get_content(position));
                let mut file = unwrap_or_return!(Code::file(file_name));

                let lexer = Lexer::unvalidated(file.content_as_bytes());
                let parser =
                    Parser::new(current_code_id, lexer, &mut module_names, &mut parse_errors);
                let syntax_tree = parser.parse();

                file.set_utf8_validated(true);

                current_code_id = self.source.add_code(file);

                self.syntax.add_tree(syntax_tree);
            }

            errors.extend(parse_errors.into_iter().map(ErrorKind::Parse));
        }

        let crate_scope_id = self.context.scopes.enter_scope(Barrier::Module, None);

        {
            let span = span!(Level::INFO, "declaration_resolution");
            let _enter = span.enter();

            let main_file_root = unwrap_or_return!(
                self.syntax
                    .get_tree(CodeId::MAIN)
                    .and_then(|tree| tree.read_root())
            );

            let mut declaration_resolver = DeclarationResolver::new(
                CodeId::MAIN,
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
        let main_prototype_id = self
            .prototypes
            .monomorphize_function_to_prototype(main_declaration_id, SmallVec::new());

        debug_assert_eq!(main_prototype_id, PrototypeId::MAIN);

        let mut main_return_type_id = None;

        while let Some(prototype_id) = self.prototypes.pop_from_compilation_stack() {
            let type_id = unwrap_or_return!(self.compile_loop(prototype_id, &mut errors));

            if prototype_id == PrototypeId::MAIN {
                let inferred_type_id =
                    unwrap_or_return!(self.context.get_inferred_type_id(type_id));

                main_return_type_id = Some(inferred_type_id);
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
    ) -> Result<TypeId, ErrorKind> {
        let (declaration_id, mut type_arguments) = self
            .prototypes
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
            return Err(ErrorKind::Compile(
                CompileError::ExpectedFunctionDefinition(declaration_id),
            ));
        };
        let (position, syntax_id) =
            declaration
                .syntax
                .ok_or(ErrorKind::Compile(CompileError::ExpectedSyntax {
                    expected: &[SyntaxKind::BlockExpression],
                }))?;
        let function_syntax = self
            .syntax
            .get_tree(position.code_id)
            .and_then(|tree| tree.read_node(syntax_id))?;
        let Ok(FunctionItem {
            body: Some(body),
            value_parameters,
            ..
        }) = function_syntax.as_component()
        else {
            return Err(ErrorKind::Compile(CompileError::ExpectedSyntax {
                expected: &[SyntaxKind::BlockExpression],
            }));
        };
        let (position, _syntax_id) =
            &declaration
                .syntax
                .ok_or(ErrorKind::Compile(CompileError::ExpectedSyntax {
                    expected: &[SyntaxKind::BlockExpression],
                }))?;

        let type_parameter_ids = self
            .context
            .get_type_parameter_ids(parent_impl_or_trait, type_parameters)?;

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

            let mut type_resolver =
                TypeResolver::new(&mut self.context, &self.source, position.code_id, errors);

            type_resolver.visit_function_body(body, return_type_id)?;
        }

        {
            let span = span!(Level::INFO, "emit");
            let _enter = span.enter();

            let mut prototype_emitter = PrototypeEmitter::new(
                declaration_id,
                prototype_id,
                return_type_id,
                value_parameters,
                position.code_id,
                (
                    &self.source,
                    &self.syntax,
                    &mut self.constants,
                    &mut self.context,
                    &mut self.prototypes,
                ),
            )?;

            prototype_emitter.visit_function_body(body)?;

            let prototype = prototype_emitter.finish()?;

            self.prototypes.set_prototype(prototype_id, prototype);
        }

        let resolved_return_type_id = self.context.get_inferred_type_id(return_type_id)?;

        Ok(resolved_return_type_id)
    }
}

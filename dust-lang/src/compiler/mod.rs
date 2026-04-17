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
        emitter::{Emitter, get_register_size},
        error::CompileError,
        resolver::{
            PrototypeId, Resolver,
            declarations::{DeclarationId, Definition, Visibility},
            scopes::{Scope, ScopeId, ScopeKind},
            types::Type,
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
    source::{FileId, Source},
    syntax::{Syntax, components::FnItem},
};

pub struct Compiler<'src> {
    syntax: Syntax,
    source: Source<'src>,
    constants: ConstantsBuilder,
    resolver: Resolver,
    compilation_stack: Vec<CompilationRequest>,
}

impl<'src> Compiler<'src> {
    pub fn new(source: Source<'src>) -> Self {
        Self {
            syntax: Syntax::with_capacity(source.file_count()),
            source,
            constants: ConstantsBuilder::new(),
            resolver: Resolver::new(),
            compilation_stack: Vec::new(),
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

            for (file_id, file) in self.source.iter_mut() {
                let lexer = if file.utf8_validated() {
                    Lexer::with_validated_source(file.content_as_str())
                } else {
                    Lexer::with_unvalidated_source(file.content_as_bytes())
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
            modules: SmallVec::new(),
            imports: SmallVec::new(),
        });
        let main_file_root = unwrap_or_return!(
            self.syntax
                .get_tree(FileId::MAIN)
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
                &mut errors,
                crate_scope_id,
            );

            match declaration_binder.bind_root(main_file_root) {
                Ok(()) => {}
                Err(error) => errors.push(ErrorKind::Compile(error)),
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        // Emission phase

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
        let Definition::Function { .. } = main_declaration.definition else {
            errors.push(ErrorKind::Compile(CompileError::ExpectedMainFunction));

            return Err(errors);
        };

        let main_prototype_id = self.resolver.reserve_prototype_id();

        debug_assert_eq!(main_prototype_id, PrototypeId::MAIN);

        self.compilation_stack.push(CompilationRequest {
            declaration_id: main_declaration_id,
            prototype_id: main_prototype_id,
        });

        let mut concrete_main_return_type_id = None;

        while let Some(CompilationRequest {
            declaration_id,
            prototype_id,
        }) = self.compilation_stack.pop()
        {
            let declaration =
                *unwrap_or_return!(self.resolver.declarations.get_declaration(declaration_id));
            let Definition::Function {
                type_parameters,
                return_type_id,
                ..
            } = declaration.definition
            else {
                continue;
            };
            let (position, syntax_id) = match declaration.syntax {
                Some(syntax) => syntax,
                None => {
                    errors.push(ErrorKind::Compile(CompileError::ExpectedMainFunction));

                    return Err(errors);
                }
            };
            let function_syntax = unwrap_or_return!(
                self.syntax
                    .get_tree(position.file_id)
                    .and_then(|tree| tree.read_node(syntax_id))
            );

            let FnItem {
                value_parameters,
                body: Some(body),
                ..
            } = unwrap_or_return!(function_syntax.as_component())
            else {
                panic!();
            };

            self.resolver.type_parameter_map.clear();

            let type_parameter_declaration_ids = unwrap_or_return!(
                self.resolver
                    .declarations
                    .get_declaration_members(&type_parameters)
            );

            for &type_parameter_declaration_id in type_parameter_declaration_ids {
                let inferred_type_id = self.resolver.types.create_inferred_type(None);

                self.resolver
                    .type_parameter_map
                    .insert(type_parameter_declaration_id, inferred_type_id);
            }

            if let Some(concrete_type_arguments) = self
                .resolver
                .get_concrete_type_arguments(prototype_id)
                .cloned()
            {
                for (&type_parameter_declaration_id, concrete_type_id) in
                    type_parameter_declaration_ids
                        .iter()
                        .zip(concrete_type_arguments.iter())
                {
                    let inferred_type_id =
                        self.resolver.type_parameter_map[&type_parameter_declaration_id];
                    let inferred_type =
                        unwrap_or_return!(self.resolver.types.get_type_mut(inferred_type_id));

                    if let Type::Inferred { resolved, .. } = inferred_type {
                        *resolved = Some(*concrete_type_id);
                    }
                }

                let trait_scope_id = declaration.scope_id;
                let scope = unwrap_or_return!(self.resolver.scopes.get_scope(trait_scope_id));

                if scope.kind == ScopeKind::Trait {
                    let mut type_argument_index = type_parameter_declaration_ids.len();

                    for (declaration_id, declaration) in self.resolver.declarations.iter() {
                        if matches!(declaration.definition, Definition::TypeParameter)
                            && declaration.scope_id == trait_scope_id
                            && !self
                                .resolver
                                .type_parameter_map
                                .contains_key(&declaration_id)
                        {
                            let type_argument_id = unwrap_or_return!(
                                concrete_type_arguments
                                    .get(type_argument_index)
                                    .copied()
                                    .ok_or(CompileError::MissingTypeArgument(declaration_id))
                            );

                            self.resolver
                                .type_parameter_map
                                .insert(declaration_id, type_argument_id);

                            type_argument_index += 1;
                        }
                    }
                }
            }

            let concrete_return_type_id = {
                let span = span!(Level::INFO, "type");
                let _enter = span.enter();

                let mut type_binder = TypeBinder::new(&mut self.resolver, &self.source);

                match type_binder.bind_function_body(body, return_type_id) {
                    Ok(()) => {}
                    Err(error) => errors.push(ErrorKind::Compile(error)),
                }

                let concrete_return_type_id =
                    unwrap_or_return!(self.resolver.resolve_type(return_type_id));

                concrete_return_type_id
            };

            {
                let span = span!(Level::INFO, "emit");
                let _enter = span.enter();

                let mut emitter = match Emitter::new(
                    Some(declaration_id),
                    prototype_id,
                    concrete_return_type_id,
                    (
                        &self.source,
                        &mut self.constants,
                        &mut self.resolver,
                        &mut self.compilation_stack,
                    ),
                    value_parameters,
                ) {
                    Ok(emitter) => emitter,
                    Err(error) => {
                        errors.push(ErrorKind::Compile(error));

                        return Err(errors);
                    }
                };

                unwrap_or_return!(emitter.emit_function_body(body));

                let prototype = unwrap_or_return!(emitter.finish());

                self.resolver.set_prototype(prototype_id, prototype);

                if prototype_id == PrototypeId::MAIN {
                    concrete_main_return_type_id = Some(concrete_return_type_id);
                }
            }
        }

        let concrete_main_return_type_id = match concrete_main_return_type_id {
            Some(type_id) => type_id,
            None => {
                errors.push(ErrorKind::Compile(CompileError::ExpectedMainFunction));

                return Err(errors);
            }
        };

        let main_function_return_type_id = unwrap_or_return!(
            self.resolver
                .get_external_type(concrete_main_return_type_id, &self.source)
        );

        if errors.is_empty() {
            Ok(main_function_return_type_id)
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug)]
pub struct CompilationRequest {
    pub declaration_id: DeclarationId,
    pub prototype_id: PrototypeId,
}

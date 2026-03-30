mod declaration_binder;
mod emitter;
pub mod error;
mod type_binder;

#[cfg(test)]
pub(crate) mod tests;

use smallvec::SmallVec;
use tracing::{Level, span};

use crate::{
    compiler::{
        declaration_binder::DeclarationBinder,
        emitter::{Emitter, get_register_size},
        error::CompileError,
        type_binder::TypeBinder,
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
        CompilationRequest, Resolver,
        declarations::{Definition, Visibility},
        scopes::{Scope, ScopeId, ScopeKind},
    },
    source::{Source, SourceFile, SourceFileId},
    syntax::{Syntax, components::FunctionItem, visitor::SyntaxVisitor},
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
            modules: SmallVec::new(),
            imports: SmallVec::new(),
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
                &mut errors,
                crate_scope_id,
            );

            match declaration_binder.visit_root(main_file_root) {
                Ok(()) => {}
                Err(error) => errors.push(ErrorKind::Compile(error)),
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        // Emission phase
        let span = span!(Level::INFO, "emit");
        let _enter = span.enter();

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

        let main_prototype_id = self.prototypes.reserve();

        debug_assert_eq!(main_prototype_id, PrototypeId::MAIN);

        self.resolver
            .compilation_queue
            .push_back(CompilationRequest {
                declaration_id: main_declaration_id,
                prototype_id: main_prototype_id,
            });

        let mut concrete_main_return_type_id = None;

        while let Some(request) = self.resolver.compilation_queue.pop_front() {
            let declaration = *unwrap_or_return!(
                self.resolver
                    .declarations
                    .get_declaration(request.declaration_id)
            );
            let Definition::Function {
                return_type_id,
                value_parameters,
                type_parameters,
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
            let function_item = unwrap_or_return!(
                self.syntax
                    .get_tree(position.file_id)
                    .and_then(|tree| tree.get_node(syntax_id))
            );
            let FunctionItem { body, .. } = unwrap_or_return!(function_item.as_component());
            let scope_id = *unwrap_or_return!(self.resolver.get_scope_binding(&body.id));

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

            let mut type_binder = TypeBinder::new(&mut self.resolver);

            match type_binder.bind_function_body(body, return_type_id) {
                Ok(()) => {}
                Err(error) => errors.push(ErrorKind::Compile(error)),
            }

            let concrete_return_type_id =
                unwrap_or_return!(self.resolver.resolve_type(return_type_id));

            let argument_count =
                {
                    let mut count = 0;

                    for index in value_parameters.as_range() {
                        let parameter_type_id =
                            *unwrap_or_return!(self.resolver.types.get_type_member(index));
                        let concrete_parameter_type_id =
                            unwrap_or_return!(self.resolver.resolve_type(parameter_type_id));
                        let register_size = if let Some(size) = unwrap_or_return!(
                            get_register_size(concrete_parameter_type_id, None, &self.resolver,)
                        ) {
                            size
                        } else {
                            errors.push(ErrorKind::Compile(CompileError::CannotInferType {
                                type_id: concrete_parameter_type_id,
                                position,
                            }));

                            return Err(errors);
                        };

                        count += register_size as u16;
                    }

                    count
                };
            let return_types =
                unwrap_or_return!(self.resolver.get_operand_types(concrete_return_type_id));

            let mut emitter = match Emitter::new(
                Some(request.declaration_id),
                request.prototype_id,
                argument_count,
                return_types,
                scope_id,
                (Some(declaration.symbol_id), position),
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
                    errors.push(ErrorKind::Compile(error));

                    return Err(errors);
                }
            };

            match emitter.emit_function_body(body) {
                Ok(()) => {}
                Err(error) => {
                    errors.push(ErrorKind::Compile(error));

                    return Err(errors);
                }
            };

            let prototype = match emitter.finish() {
                Ok(prototype) => prototype,
                Err(error) => {
                    errors.push(ErrorKind::Compile(error));

                    return Err(errors);
                }
            };

            self.prototypes.set(request.prototype_id, prototype);

            if request.prototype_id == PrototypeId::MAIN {
                concrete_main_return_type_id = Some(concrete_return_type_id);
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

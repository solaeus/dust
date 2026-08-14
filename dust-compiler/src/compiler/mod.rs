pub mod context;
mod declaration_resolver;
pub mod error;
mod prototype_emitter;
pub mod prototypes;
mod type_resolver;
mod value_creation;

#[cfg(test)]
pub(crate) mod tests;

use crate::{
    compiler::{
        context::{
            Context,
            declarations::{CrateKind, Declaration, DeclarationId, Definition, ModuleKind},
            scopes::{Barrier, BarrierTracker, ScopeId},
            types::TypeId,
        },
        declaration_resolver::{DeclarationResolver, WorklistEntry},
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
    source::{CodeId, Source},
    syntax::{Syntax, components::FunctionItem, node::SyntaxKind},
};

pub struct Compiler<'src> {
    syntax: Syntax,
    source: Source<'src>,
    constants: ConstantsBuilder,
    context: Context,
    prototypes: Prototypes,
    errors: Vec<ErrorKind>,
}

impl<'src> Compiler<'src> {
    pub fn new(source: Source<'src>) -> Self {
        Self {
            syntax: Syntax::with_capacity(source.file_count()),
            source,
            constants: ConstantsBuilder::new(),
            context: Context::new(),
            prototypes: Prototypes::new(),
            errors: Vec::new(),
        }
    }

    pub fn compile(mut self) -> Result<Program, Error<'src>> {
        match self.compile_inner() {
            Ok(return_type) => {
                let (constants, _) = self.constants.build();
                let prototypes = self.prototypes.into_prototypes();
                let program = Program::new(
                    self.source.into_program_name(),
                    return_type,
                    constants,
                    prototypes,
                );

                Ok(program)
            }
            Err(error) => {
                self.errors.push(error);

                let error_context =
                    ErrorContext::Full(self.source, self.syntax, Box::new(self.context));
                let errors = Error::new(self.errors, error_context);

                Err(errors)
            }
        }
    }

    pub fn compile_with_extras(
        mut self,
    ) -> Result<(Program, Source<'src>, Syntax, Vec<OperandType>), Error<'src>> {
        match self.compile_inner() {
            Ok(return_type) => {
                let (constants, constant_tags) = self.constants.build();
                let program = Program::new(
                    self.source.program_name().clone(),
                    return_type,
                    constants,
                    self.prototypes.into_prototypes(),
                );

                Ok((program, self.source, self.syntax, constant_tags))
            }
            Err(error) => {
                self.errors.push(error);

                let error_context =
                    ErrorContext::Full(self.source, self.syntax, Box::new(self.context));
                let errors = Error::new(self.errors, error_context);

                Err(errors)
            }
        }
    }

    fn compile_inner(&mut self) -> Result<DustType, ErrorKind> {
        let program_symbol_id = self.context.symbols.add_symbol(self.source.program_name());
        let program_scope_id = self.context.scopes.enter_scope(Barrier::Module, None);

        self.context.declarations.add_declaration(Declaration {
            symbol_id: program_symbol_id,
            scope_id: program_scope_id,
            definition: Definition::Crate {
                kind: CrateKind::Program,
                inner_scope_id: program_scope_id,
            },
            syntax: None,
        });

        let main_symbol_id = self.context.symbols.add_symbol("main");
        let main_module_declaration_id = self.context.declarations.reserve_declaration_id(
            main_symbol_id,
            program_scope_id,
            None,
        );

        let mut declaration_worklist = Vec::new();
        let mut forward_references = Vec::new();

        let main_module_scope_id = self.declare_file_module(
            CodeId::MAIN,
            main_module_declaration_id,
            program_scope_id,
            false,
            &mut declaration_worklist,
            &mut forward_references,
        )?;

        let mut worklist_index = 0;

        while worklist_index < declaration_worklist.len() {
            let WorklistEntry {
                module_code_id,
                module_declaration_id,
                parent_scope_id,
                public,
            } = declaration_worklist[worklist_index];

            worklist_index += 1;

            self.declare_file_module(
                module_code_id,
                module_declaration_id,
                parent_scope_id,
                public,
                &mut declaration_worklist,
                &mut forward_references,
            )?;
        }

        let main_function_declaration_id = *self
            .context
            .declarations
            .find_declaration_id(main_symbol_id, main_module_scope_id)
            .ok_or(ErrorKind::Compile(CompileError::ExpectedMainFunction))?;

        self.prototypes
            .monomorphize_main_prototype(main_function_declaration_id);

        let main_return_type_id = self.compile_prototype(PrototypeId::MAIN)?;

        while let Some(prototype_id) = self.prototypes.pop_from_compilation_stack() {
            self.compile_prototype(prototype_id)?;
        }

        let main_function_return_type = self.context.get_external_type(main_return_type_id)?;

        Ok(main_function_return_type)
    }

    fn declare_file_module(
        &mut self,
        module_code_id: CodeId,
        module_declaration_id: DeclarationId,
        parent_scope_id: ScopeId,
        public: bool,
        worklist: &mut Vec<WorklistEntry>,
        forward_references: &mut Vec<DeclarationId>,
    ) -> Result<ScopeId, ErrorKind> {
        let source_code = self.source.get_code(module_code_id);
        let lexer = if source_code.utf8_validated() {
            Lexer::validated(source_code.content_as_str())
        } else {
            Lexer::unvalidated(source_code.content_as_bytes())
        };
        let parser = Parser::new(module_code_id, lexer, &mut self.errors);
        let syntax_tree = parser.parse();

        self.source.set_utf8_validated(module_code_id);
        self.syntax.add_tree(syntax_tree);

        let module_scope_id = self
            .context
            .scopes
            .enter_scope(Barrier::Module, Some(parent_scope_id));

        self.context.declarations.set_reserved_declaration(
            module_declaration_id,
            Definition::Module {
                public,
                kind: ModuleKind::File {
                    code_id: module_code_id,
                },
                inner_scope_id: module_scope_id,
            },
        );

        let declaration_resolver = DeclarationResolver::new(
            module_code_id,
            &mut self.source,
            &mut self.context,
            worklist,
            forward_references,
            &mut self.errors,
            module_scope_id,
        );
        let module_root = self
            .syntax
            .get_tree(module_code_id)
            .and_then(|tree| tree.read_root())?;

        declaration_resolver.visit_root(module_root)?;
        self.context.scopes.exit_scope(module_scope_id);

        for forward_reference_id in forward_references.drain(..) {
            let forward_reference = *self
                .context
                .declarations
                .get_declaration(forward_reference_id);

            let resolved_declaration_id = {
                let mut crossed_barriers = BarrierTracker::default();
                let mut current_scope_id = forward_reference.scope_id;

                loop {
                    if let Some(declaration_id) = self
                        .context
                        .declarations
                        .find_declaration_id(forward_reference.symbol_id, current_scope_id)
                        .copied()
                    {
                        let declaration = self.context.declarations.get_declaration(declaration_id);

                        if crossed_barriers.should_block(&declaration.definition) {
                            return Err(ErrorKind::Compile(CompileError::Undeclared {
                                symbol_id: forward_reference.symbol_id,
                                usage_position: forward_reference
                                    .syntax
                                    .ok_or(CompileError::MissingSyntax(declaration_id))?
                                    .0,
                            }));
                        }

                        break declaration_id;
                    }

                    let scope = self.context.scopes.get_scope(current_scope_id);
                    current_scope_id = if let Some(parent) = scope.parent {
                        parent
                    } else {
                        return Err(ErrorKind::Compile(CompileError::Undeclared {
                            symbol_id: forward_reference.symbol_id,
                            usage_position: forward_reference
                                .syntax
                                .ok_or(CompileError::MissingSyntax(forward_reference_id))?
                                .0,
                        }));
                    };

                    crossed_barriers.add(scope.barrier);
                }
            };

            self.context
                .declarations
                .resolve_forward_reference(forward_reference_id, resolved_declaration_id);
        }

        Ok(module_scope_id)
    }

    fn compile_prototype(&mut self, prototype_id: PrototypeId) -> Result<TypeId, ErrorKind> {
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
        let FunctionItem {
            value_parameters,
            body,
            ..
        } = function_syntax.as_component()?;
        let body = body.ok_or(ErrorKind::Compile(CompileError::ExpectedSyntax {
            expected: &[SyntaxKind::BlockExpression],
        }))?;
        let (position, _syntax_id) =
            declaration
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

        let mut type_resolver = TypeResolver::new(
            &mut self.context,
            &self.source,
            position.code_id,
            &mut self.errors,
        );

        type_resolver.visit_function_body(body, return_type_id)?;

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

        let resolved_return_type_id = self.context.get_inferred_type_id(return_type_id)?;

        Ok(resolved_return_type_id)
    }
}

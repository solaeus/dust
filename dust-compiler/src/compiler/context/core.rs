use crate::compiler::context::{Context, Definition, ScopeId, Type, VariantKind, scopes::Barrier};

pub fn add_core(context: &mut Context) {
    let core_scope_id = context.scopes.enter_scope(Barrier::Module, None);

    add_option(context, core_scope_id);

    context.scopes.exit_scope(core_scope_id);

    debug_assert_eq!(core_scope_id, ScopeId::CORE);
}

fn add_option(context: &mut Context, core_scope_id: ScopeId) {
    let option_symbol_id = context.symbols.add_symbol("Option");
    let some_symbol_id = context.symbols.add_symbol("Some");
    let none_symbol_id = context.symbols.add_symbol("None");
    let t_symbol_id = context.symbols.add_symbol("T");
    let zero_symbol_id = context.symbols.add_index_symbol(0);

    let option_declaration_id =
        context
            .declarations
            .reserve_declaration_id(option_symbol_id, core_scope_id, None);
    let some_declaration_id =
        context
            .declarations
            .reserve_declaration_id(some_symbol_id, core_scope_id, None);
    let none_declaration_id =
        context
            .declarations
            .reserve_declaration_id(none_symbol_id, core_scope_id, None);
    let t_declaration_id =
        context
            .declarations
            .reserve_declaration_id(t_symbol_id, core_scope_id, None);
    let zero_declaration_id =
        context
            .declarations
            .reserve_declaration_id(zero_symbol_id, core_scope_id, None);

    let option_scope_id = context
        .scopes
        .enter_scope(Barrier::Item, Some(core_scope_id));

    context.scopes.add_to_current_scope(some_declaration_id);
    context.scopes.add_to_current_scope(none_declaration_id);

    let type_parameter_scope_id = context
        .scopes
        .enter_scope(Barrier::Members, Some(option_scope_id));

    context.scopes.add_to_current_scope(t_declaration_id);

    let variants_scope_id = context
        .scopes
        .enter_scope(Barrier::Members, Some(type_parameter_scope_id));

    context.scopes.add_to_current_scope(some_declaration_id);
    context.scopes.add_to_current_scope(none_declaration_id);

    let some_fields_scope_id = context
        .scopes
        .enter_scope(Barrier::Members, Some(variants_scope_id));

    context.scopes.add_to_current_scope(zero_declaration_id);
    context.scopes.exit_scope(some_fields_scope_id);
    context.scopes.exit_scope(variants_scope_id);
    context.scopes.exit_scope(type_parameter_scope_id);
    context.scopes.exit_scope(option_scope_id);

    let t_type_id = context.types.add_type(Type::Generic {
        declaration_id: t_declaration_id,
    });

    context.declarations.set_reserved_declaration(
        option_declaration_id,
        Definition::EnumType {
            public: true,
            type_parameters: Some(type_parameter_scope_id),
            variants: variants_scope_id,
        },
    );
    context.declarations.set_reserved_declaration(
        some_declaration_id,
        Definition::Variant {
            discriminant: 0,
            enum_declaration_id: option_declaration_id,
            fields: Some(some_fields_scope_id),
            kind: VariantKind::TupleFields,
        },
    );
    context.declarations.set_reserved_declaration(
        none_declaration_id,
        Definition::Variant {
            discriminant: 1,
            enum_declaration_id: option_declaration_id,
            fields: None,
            kind: VariantKind::Unit,
        },
    );
    context.declarations.set_reserved_declaration(
        t_declaration_id,
        Definition::TypeParameter {
            is_self: false,
            bounds: None,
        },
    );
    context.declarations.set_reserved_declaration(
        zero_declaration_id,
        Definition::Field {
            public: true,
            parent_struct: some_declaration_id,
            type_id: t_type_id,
        },
    );
}

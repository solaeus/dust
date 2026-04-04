#![allow(clippy::disallowed_methods)]

mod array_expression;
mod array_repeat_expression;
mod assignment_expression;
mod block_expression;
mod boolean_expression;
mod byte_expression;
mod call_expression;
mod character_expression;
mod comparison_expression;
mod const_item;
mod expression_statement;
mod field_access_expression;
mod float_expression;
mod function_item;
mod grouped_expression;
mod if_expression;
mod impl_item;
mod index_expression;
mod integer_expression;
mod let_statement;
mod logic_expression;
mod math_expression;
mod negation_expression;
mod not_expression;
mod path_expression;
mod range_expression;
mod struct_expression;
mod while_expression;

use crate::{
    compiler::{
        emitter::{Emitter, get_register_size},
        tests::type_bind_function,
    },
    constant_list::ConstantListBuilder,
    prototype::{Prototype, PrototypeList},
    resolver::declarations::{Definition, Visibility},
    source::{Source, SourceFile},
    syntax::components::FunctionItem,
};

fn emit_function(source_code: &str) -> Prototype {
    let (syntax, mut resolver, crate_scope_id) = type_bind_function(source_code);

    let foo_symbol = resolver.symbols.add_symbol("foo");
    let (declaration_id, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let foo_declaration = *foo_declaration;

    let Definition::Function {
        return_type_id,
        value_parameters,
        ..
    } = foo_declaration.definition
    else {
        panic!();
    };

    let (position, syntax_id) = foo_declaration.syntax.unwrap();

    let function_item = syntax
        .get_tree(position.file_id)
        .and_then(|tree| tree.get_node(syntax_id))
        .unwrap();
    let FunctionItem {
        parameters, body, ..
    } = function_item.as_component().unwrap();

    let scope_id = *resolver.get_scope_binding(&body.id).unwrap();

    let concrete_return_type_id = resolver.resolve_type(return_type_id).unwrap();

    let mut argument_count = 0u16;
    for index in value_parameters.as_range() {
        let parameter_type_id = *resolver.types.get_type_member(index).unwrap();
        let concrete_parameter_type_id = resolver.resolve_type(parameter_type_id).unwrap();
        let register_size = get_register_size(concrete_parameter_type_id, None, &resolver).unwrap();
        argument_count += register_size.unwrap_or(0) as u16;
    }

    let return_types = resolver.get_operand_types(concrete_return_type_id).unwrap();

    let mut source = Source::new();
    source.add_file(SourceFile::validated_borrowed("test", source_code));

    let mut constants = ConstantListBuilder::new();
    let mut prototypes = PrototypeList::new();
    let prototype_id = prototypes.reserve();
    let mut compilation_stack = Vec::new();

    let mut emitter = Emitter::new(
        Some(declaration_id),
        prototype_id,
        argument_count,
        return_types,
        scope_id,
        (
            &source,
            &mut constants,
            &mut resolver,
            &mut prototypes,
            &mut compilation_stack,
        ),
    )
    .unwrap();

    emitter.bind_parameters(parameters).unwrap();
    emitter.emit_function_body(body).unwrap();
    emitter.finish().unwrap()
}

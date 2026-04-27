#![allow(clippy::disallowed_macros)]
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
mod enum_expression;
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
    compiler::{emitter::Emitter, resolver::declarations::Definition, tests::type_bind_function},
    constants::ConstantsBuilder,
    prototype::Prototype,
    source::{Source, SourceCode},
    syntax::components::FnItem,
};

fn emit_function(source_code: &str) -> Prototype {
    let (syntax, mut resolver, crate_scope_id) = type_bind_function(source_code);

    let foo_symbol = resolver.symbols.add_symbol("foo");
    let declaration_id = *resolver
        .declarations
        .find_declaration_id(foo_symbol, crate_scope_id)
        .unwrap();
    let foo_declaration = resolver
        .declarations
        .get_declaration(declaration_id)
        .unwrap();

    let Definition::Function { return_type_id, .. } = foo_declaration.definition else {
        panic!();
    };

    let (position, syntax_id) = foo_declaration.syntax.unwrap();

    let function_item = syntax
        .get_tree(position.source_id)
        .and_then(|tree| tree.read_node(syntax_id))
        .unwrap();
    let FnItem {
        value_parameters,
        body,
        ..
    } = function_item.as_component().unwrap();

    let concrete_return_type_id = resolver.resolve_type(return_type_id).unwrap();

    let mut source = Source::new();
    source.add_code(SourceCode::validated_borrowed("test", source_code));

    let mut constants = ConstantsBuilder::new();
    let prototype_id = resolver.reserve_prototype_id();
    let mut compilation_stack = Vec::new();

    let mut emitter = Emitter::new(
        Some(declaration_id),
        prototype_id,
        concrete_return_type_id,
        (
            &source,
            &mut constants,
            &mut resolver,
            &mut compilation_stack,
        ),
        value_parameters,
    )
    .unwrap();

    emitter.emit_function_body(body.unwrap()).unwrap();
    emitter.finish().unwrap()
}

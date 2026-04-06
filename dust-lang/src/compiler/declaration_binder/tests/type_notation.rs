use crate::{
    compiler::declaration_binder::tests::bind_declarations_with_errors,
    resolver::{
        declarations::{Definition, Visibility},
        types::{Type, TypeId},
    },
    source::{Source, SourceCode, SourceFileId},
    syntax::node::SyntaxKind,
};

use super::bind_declarations;

fn parameter_type_of_foo(source_code: &str) -> TypeId {
    let mut source = Source::new();

    source.add_file(SourceCode::validated_borrowed("test", source_code));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let foo_symbol = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Function {
        value_parameters, ..
    } = foo_declaration.definition
    else {
        panic!();
    };

    assert_eq!(value_parameters.len(), 1);

    let parameter_types = resolver.types.get_type_members(value_parameters).unwrap();

    parameter_types[0]
}

#[test]
fn boolean_type() {
    let type_id = parameter_type_of_foo("fn foo(x: bool) {}");

    assert_eq!(type_id, TypeId::BOOLEAN);
}

#[test]
fn i8_type() {
    let type_id = parameter_type_of_foo("fn foo(x: i8) {}");

    assert_eq!(type_id, TypeId::I_8);
}

#[test]
fn i16_type() {
    let type_id = parameter_type_of_foo("fn foo(x: i16) {}");

    assert_eq!(type_id, TypeId::I_16);
}

#[test]
fn i32_type() {
    let type_id = parameter_type_of_foo("fn foo(x: i32) {}");

    assert_eq!(type_id, TypeId::I_32);
}

#[test]
fn i64_type() {
    let type_id = parameter_type_of_foo("fn foo(x: i64) {}");

    assert_eq!(type_id, TypeId::I_64);
}

#[test]
fn i128_type() {
    let type_id = parameter_type_of_foo("fn foo(x: i128) {}");

    assert_eq!(type_id, TypeId::I_128);
}

#[test]
fn isize_type() {
    let type_id = parameter_type_of_foo("fn foo(x: isize) {}");

    assert_eq!(type_id, TypeId::I_SIZE);
}

#[test]
fn u8_type() {
    let type_id = parameter_type_of_foo("fn foo(x: u8) {}");

    assert_eq!(type_id, TypeId::U_8);
}

#[test]
fn u16_type() {
    let type_id = parameter_type_of_foo("fn foo(x: u16) {}");

    assert_eq!(type_id, TypeId::U_16);
}

#[test]
fn u32_type() {
    let type_id = parameter_type_of_foo("fn foo(x: u32) {}");

    assert_eq!(type_id, TypeId::U_32);
}

#[test]
fn u64_type() {
    let type_id = parameter_type_of_foo("fn foo(x: u64) {}");

    assert_eq!(type_id, TypeId::U_64);
}

#[test]
fn u128_type() {
    let type_id = parameter_type_of_foo("fn foo(x: u128) {}");

    assert_eq!(type_id, TypeId::U_128);
}

#[test]
fn usize_type() {
    let type_id = parameter_type_of_foo("fn foo(x: usize) {}");

    assert_eq!(type_id, TypeId::U_SIZE);
}

#[test]
fn f32_type() {
    let type_id = parameter_type_of_foo("fn foo(x: f32) {}");

    assert_eq!(type_id, TypeId::F_32);
}

#[test]
fn f64_type() {
    let type_id = parameter_type_of_foo("fn foo(x: f64) {}");

    assert_eq!(type_id, TypeId::F_64);
}

#[test]
fn character_type() {
    let type_id = parameter_type_of_foo("fn foo(x: char) {}");

    assert_eq!(type_id, TypeId::CHARACTER);
}

#[test]
fn tuple_type_empty() {
    let mut source = Source::new();

    source.add_file(SourceCode::validated_borrowed("test", "fn foo(x: ()) {}"));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let foo_symbol = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Function {
        value_parameters, ..
    } = foo_declaration.definition
    else {
        panic!();
    };

    let parameter_types = resolver.types.get_type_members(value_parameters).unwrap();
    let tuple_type = resolver.types.get_type(parameter_types[0]).unwrap();
    let Type::Tuple { element_type_ids } = tuple_type else {
        panic!();
    };

    assert_eq!(value_parameters.len(), 1);
    assert!(element_type_ids.is_empty());
}

#[test]
fn tuple_type_multiple() {
    let mut source = Source::new();
    source.add_file(SourceCode::validated_borrowed(
        "test",
        "fn foo(x: (i64, bool)) {}",
    ));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let foo_symbol = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Function {
        value_parameters, ..
    } = foo_declaration.definition
    else {
        panic!();
    };

    let parameter_types = resolver.types.get_type_members(value_parameters).unwrap();
    let tuple_type = resolver.types.get_type(parameter_types[0]).unwrap();
    let Type::Tuple { element_type_ids } = tuple_type else {
        panic!();
    };
    let elements = resolver.types.get_type_members(*element_type_ids).unwrap();

    assert_eq!(value_parameters.len(), 1);
    assert_eq!(elements, &[TypeId::I_64, TypeId::BOOLEAN]);
}

#[test]
fn slice_type() {
    let mut source = Source::new();

    source.add_file(SourceCode::validated_borrowed(
        "test",
        "fn foo(x: [i64]) {}",
    ));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let foo_symbol = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Function {
        value_parameters, ..
    } = foo_declaration.definition
    else {
        panic!();
    };

    let parameter_types = resolver.types.get_type_members(value_parameters).unwrap();
    let slice_type = resolver.types.get_type(parameter_types[0]).unwrap();
    let Type::Slice {
        element_type_id, ..
    } = slice_type
    else {
        panic!();
    };

    assert_eq!(value_parameters.len(), 1);
    assert_eq!(*element_type_id, TypeId::I_64);
}

#[test]
fn function_type_basic() {
    let mut source = Source::new();

    source.add_file(SourceCode::validated_borrowed(
        "test",
        "fn foo(x: fn(i64) -> i64) {}",
    ));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let foo_symbol = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Function {
        value_parameters, ..
    } = foo_declaration.definition
    else {
        panic!();
    };

    assert_eq!(value_parameters.len(), 1);

    let parameter_types = resolver.types.get_type_members(value_parameters).unwrap();
    let function_type = resolver.types.get_type(parameter_types[0]).unwrap();
    let Type::Function {
        value_parameters: function_value_parameters,
        return_type,
    } = function_type
    else {
        panic!();
    };
    let parameters = resolver
        .types
        .get_type_members(*function_value_parameters)
        .unwrap();

    assert_eq!(parameters, &[TypeId::I_64]);
    assert_eq!(*return_type, TypeId::I_64);
}

#[test]
fn function_type_no_params() {
    let mut source = Source::new();

    source.add_file(SourceCode::validated_borrowed(
        "test",
        "fn foo(x: fn() -> bool) {}",
    ));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let foo_symbol = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Function {
        value_parameters, ..
    } = foo_declaration.definition
    else {
        panic!();
    };

    let parameter_types = resolver.types.get_type_members(value_parameters).unwrap();
    let function_type = resolver.types.get_type(parameter_types[0]).unwrap();
    let Type::Function {
        value_parameters: function_value_parameters,
        return_type,
        ..
    } = function_type
    else {
        panic!();
    };

    assert!(function_value_parameters.is_empty());
    assert_eq!(*return_type, TypeId::BOOLEAN);
}

#[test]
fn function_type_multiple_params() {
    let mut source = Source::new();

    source.add_file(SourceCode::validated_borrowed(
        "test",
        "fn foo(x: fn(i64, bool) -> char) {}",
    ));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let foo_symbol = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Function {
        value_parameters, ..
    } = foo_declaration.definition
    else {
        panic!();
    };

    let parameter_types = resolver.types.get_type_members(value_parameters).unwrap();
    let function_type = resolver.types.get_type(parameter_types[0]).unwrap();
    let Type::Function {
        value_parameters: function_value_parameters,
        return_type,
        ..
    } = function_type
    else {
        panic!();
    };
    let function_parameters = resolver
        .types
        .get_type_members(*function_value_parameters)
        .unwrap();

    assert_eq!(function_parameters, &[TypeId::I_64, TypeId::BOOLEAN]);
    assert_eq!(*return_type, TypeId::CHARACTER);
}

#[test]
fn function_type_no_return() {
    let mut source = Source::new();

    source.add_file(SourceCode::validated_borrowed(
        "test",
        "fn foo(x: fn(i64)) {}",
    ));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let foo_symbol = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Function {
        value_parameters, ..
    } = foo_declaration.definition
    else {
        panic!();
    };

    let parameter_types = resolver.types.get_type_members(value_parameters).unwrap();
    let function_type = resolver.types.get_type(parameter_types[0]).unwrap();
    let Type::Function {
        value_parameters,
        return_type,
        ..
    } = function_type
    else {
        panic!();
    };
    let value_parameters = resolver.types.get_type_members(*value_parameters).unwrap();

    assert_eq!(value_parameters, &[TypeId::I_64]);
    assert_eq!(*return_type, TypeId::UNIT);
}

#[test]
fn type_path_to_struct() {
    let mut source = Source::new();

    source.add_file(SourceCode::validated_borrowed(
        "test",
        "struct Bar {} fn foo(x: Bar) {}",
    ));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let bar_symbol = resolver.symbols.add_symbol("Bar");
    let (bar_declaration_id, _) = resolver
        .declarations
        .find_declaration(bar_symbol, crate_scope_id, Visibility::Module)
        .unwrap();

    let foo_symbol = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Function {
        value_parameters, ..
    } = foo_declaration.definition
    else {
        panic!();
    };

    let parameter_types = resolver.types.get_type_members(value_parameters).unwrap();
    let parameter_type = resolver.types.get_type(parameter_types[0]).unwrap();
    let Type::Algebraic {
        declaration_id,
        type_arguments,
    } = parameter_type
    else {
        panic!();
    };

    assert_eq!(*declaration_id, bar_declaration_id);
    assert!(type_arguments.is_empty());
}

#[test]
fn type_path_to_enum() {
    let mut source = Source::new();

    source.add_file(SourceCode::validated_borrowed(
        "test",
        "enum Color { Red } fn foo(x: Color) {}",
    ));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let color_symbol = resolver.symbols.add_symbol("Color");
    let (color_declaration_id, _) = resolver
        .declarations
        .find_declaration(color_symbol, crate_scope_id, Visibility::Module)
        .unwrap();

    let foo_symbol = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Function {
        value_parameters, ..
    } = foo_declaration.definition
    else {
        panic!();
    };

    let parameter_types = resolver.types.get_type_members(value_parameters).unwrap();
    let parameter_type = resolver.types.get_type(parameter_types[0]).unwrap();
    let Type::Algebraic { declaration_id, .. } = parameter_type else {
        panic!();
    };

    assert_eq!(*declaration_id, color_declaration_id);
}

#[test]
fn type_path_to_type_parameter() {
    let mut source = Source::new();

    source.add_file(SourceCode::validated_borrowed("test", "fn foo<T>(x: T) {}"));

    let (_syntax, mut resolver, crate_scope_id) = bind_declarations(&source);
    let foo_symbol = resolver.symbols.add_symbol("foo");
    let (_, foo_declaration) = resolver
        .declarations
        .find_declaration(foo_symbol, crate_scope_id, Visibility::Module)
        .unwrap();
    let Definition::Function {
        type_parameters,
        value_parameters,
        ..
    } = foo_declaration.definition
    else {
        panic!();
    };

    let type_parameter_ids = resolver
        .declarations
        .get_declaration_members(&type_parameters)
        .unwrap();

    assert_eq!(type_parameters.len(), 1);

    let t_declaration_id = type_parameter_ids[0];
    let parameter_types = resolver.types.get_type_members(value_parameters).unwrap();
    let parameter_type = resolver.types.get_type(parameter_types[0]).unwrap();
    let Type::Generic { declaration_id } = parameter_type else {
        panic!();
    };

    assert_eq!(*declaration_id, t_declaration_id);
}

#[test]
fn type_path_to_non_type_errors() {
    let mut source = Source::new();

    source.add_file(SourceCode::validated_borrowed(
        "test",
        "fn bar() {} fn foo(x: bar) {}",
    ));

    let (_syntax, _resolver, _crate_scope_id, errors) = bind_declarations_with_errors(&source);

    assert!(!errors.is_empty());
}

#[test]
fn type_path_in_turbofish_binds_declaration() {
    let mut source = Source::new();

    source.add_file(SourceCode::validated_borrowed(
        "test",
        "struct Bar {} fn foo<T>() {} fn main() { foo::<Bar>(); }",
    ));

    let (syntax, mut resolver, crate_scope_id) = bind_declarations(&source);

    let bar_symbol = resolver.symbols.add_symbol("Bar");
    let (bar_declaration_id, _) = resolver
        .declarations
        .find_declaration(bar_symbol, crate_scope_id, Visibility::Module)
        .unwrap();

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let type_path = tree
        .iter()
        .find(|node| node.node.kind == SyntaxKind::TypePath)
        .unwrap();

    let declaration_id = resolver.get_declaration_binding(&type_path.id).unwrap();

    assert_eq!(*declaration_id, bar_declaration_id);
}

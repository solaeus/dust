use crate::{
    compiler::{resolver::types::TypeId, tests::type_bind_function},
    source::SourceFileId, syntax::node::SyntaxKind,
};

#[test]
fn trait_method_on_value() {
    type_bind_function(
        r#"
        trait Greet {
            fn greet(self) -> i32;
        }

        struct Person {}

        impl Greet for Person {
            fn greet(self) -> i32 { 42 }
        }

        fn foo(person: Person) -> i32 {
            person.greet()
        }
        "#,
    );
}

#[test]
fn trait_method_return_type() {
    let (syntax, mut resolver, _) = type_bind_function(
        r#"
        trait Greet {
            fn greet(self) -> i32;
        }

        struct Person {}

        impl Greet for Person {
            fn greet(self) -> i32 { 42 }
        }

        fn foo(person: Person) -> i32 {
            person.greet()
        }
        "#,
    );

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let call_expression = tree
        .iter()
        .find(|node| node.node.kind == SyntaxKind::CallExpression)
        .unwrap();

    let type_id = *resolver.get_type_binding(&call_expression.id).unwrap();
    let resolved = resolver.resolve_type(type_id).unwrap();

    assert_eq!(resolved, TypeId::I_32);
}

#[test]
fn trait_method_with_parameter() {
    type_bind_function(
        r#"
        trait Transform {
            fn transform(self, factor: i32) -> i32;
        }

        struct Value {
            x: i32,
        }

        impl Transform for Value {
            fn transform(self, factor: i32) -> i32 { self.x * factor }
        }

        fn foo(value: Value) -> i32 {
            value.transform(5)
        }
        "#,
    );
}

#[test]
fn trait_associated_function_via_path() {
    type_bind_function(
        r#"
        trait Create {
            fn create() -> i32;
        }

        struct Factory {}

        impl Create for Factory {
            fn create() -> i32 { 42 }
        }

        fn foo() -> i32 {
            Factory::create()
        }
        "#,
    );
}

#[test]
fn default_trait_method() {
    type_bind_function(
        r#"
        trait Greet {
            fn greet(self) -> i32 { 42 }
        }

        struct Person {}

        impl Greet for Person {}

        fn foo(person: Person) -> i32 {
            person.greet()
        }
        "#,
    );
}

#[test]
fn default_method_calling_implemented_method() {
    type_bind_function(
        r#"
        struct Point {
            x: i32,
        }

        trait GetX {
            fn access_x(self) -> i32;
            fn get_x(self) -> i32 { self.access_x() }
        }

        impl GetX for Point {
            fn access_x(self) -> i32 { self.x }
        }

        fn foo(p: Point) -> i32 {
            p.get_x()
        }
        "#,
    );
}

#[test]
fn self_as_return_type() {
    let (syntax, mut resolver, _) = type_bind_function(
        r#"
        struct Point {
            x: i32,
        }

        trait Identity {
            fn identity(self) -> Self;
        }

        impl Identity for Point {
            fn identity(self) -> Self { self }
        }

        fn foo(p: Point) -> Point {
            p.identity()
        }
        "#,
    );

    let tree = syntax.get_tree(SourceFileId::MAIN).unwrap();
    let call_expression = tree
        .iter()
        .find(|node| node.node.kind == SyntaxKind::CallExpression)
        .unwrap();

    let type_id = *resolver.get_type_binding(&call_expression.id).unwrap();
    let resolved = resolver.resolve_type(type_id).unwrap();
    let point_type = resolver.types.get_type(resolved).unwrap();

    assert!(
        matches!(point_type, crate::compiler::resolver::types::Type::Algebraic { .. }),
        "expected Self to resolve to Point struct, got {point_type:?}"
    );
}

#[test]
fn self_as_parameter_type() {
    type_bind_function(
        r#"
        struct Point {
            x: i32,
        }

        trait Combine {
            fn combine(self, other: Self) -> i32;
        }

        impl Combine for Point {
            fn combine(self, other: Self) -> i32 { self.x + other.x }
        }

        fn foo(a: Point, b: Point) -> i32 {
            a.combine(b)
        }
        "#,
    );
}

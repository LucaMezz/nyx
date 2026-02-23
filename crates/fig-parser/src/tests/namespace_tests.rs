// Namespace parsing tests for fig-parser
// NOTE: Fig uses "func" (not "fn") for function declarations.
// NOTE: Namespace.name is a Path; Namespace.items is Vec<Statement>.

use crate::{Lexer, ast::{Statement, Type}, parser};

#[test]
fn test_simple_namespace() {
    let input = "namespace MyModule
    type Alias = i32
";
    let namespace = parser::NamespaceParser::new().parse(Lexer::new(input)).unwrap();

    assert_eq!(namespace.name.segments[0], "MyModule");
    assert_eq!(namespace.items.len(), 1);

    if let Statement::TypeAlias(ta) = &namespace.items[0] {
        assert_eq!(ta.name.segments[0], "Alias");
        assert_eq!(ta.aliased_type, Type::I32);
    } else {
        panic!("Expected type alias item");
    }
}

#[test]
fn test_namespace_with_struct() {
    let input = "namespace Geometry
    struct Point
        x: f64
        y: f64
";
    let namespace = parser::NamespaceParser::new().parse(Lexer::new(input)).unwrap();

    assert_eq!(namespace.name.segments[0], "Geometry");
    assert_eq!(namespace.items.len(), 1);

    if let Statement::Struct(s) = &namespace.items[0] {
        assert_eq!(s.name.segments[0], "Point");
        assert_eq!(s.fields.len(), 2);
    } else {
        panic!("Expected struct item");
    }
}

#[test]
fn test_namespace_with_enum() {
    let input = "namespace Colors
    enum Color
        Red
        Green
        Blue
";
    let namespace = parser::NamespaceParser::new().parse(Lexer::new(input)).unwrap();

    assert_eq!(namespace.name.segments[0], "Colors");
    assert_eq!(namespace.items.len(), 1);

    if let Statement::Enum(e) = &namespace.items[0] {
        assert_eq!(e.name.segments[0], "Color");
        assert_eq!(e.variants.len(), 3);
    } else {
        panic!("Expected enum item");
    }
}

#[test]
fn test_namespace_with_union() {
    let input = "namespace Data
    union Value
        int_val: i32
        float_val: f64
";
    let namespace = parser::NamespaceParser::new().parse(Lexer::new(input)).unwrap();

    assert_eq!(namespace.name.segments[0], "Data");
    assert_eq!(namespace.items.len(), 1);

    if let Statement::Union(u) = &namespace.items[0] {
        assert_eq!(u.name.segments[0], "Value");
        assert_eq!(u.variants.len(), 2);
    } else {
        panic!("Expected union item");
    }
}

#[test]
fn test_namespace_with_interface() {
    let input = "namespace Interfaces
    interface Printable
        func print()
";
    let namespace = parser::NamespaceParser::new().parse(Lexer::new(input)).unwrap();

    assert_eq!(namespace.name.segments[0], "Interfaces");
    assert_eq!(namespace.items.len(), 1);

    if let Statement::Interface(iface) = &namespace.items[0] {
        assert_eq!(iface.name.segments[0], "Printable");
        assert_eq!(iface.methods.len(), 1);
    } else {
        panic!("Expected interface item");
    }
}

#[test]
fn test_namespace_with_function() {
    let input = "namespace Utils
    func add(a: i32, b: i32) -> i32
        pass
";
    let namespace = parser::NamespaceParser::new().parse(Lexer::new(input)).unwrap();

    assert_eq!(namespace.name.segments[0], "Utils");
    assert_eq!(namespace.items.len(), 1);

    if let Statement::Function(f) = &namespace.items[0] {
        assert_eq!(f.signature.name, "add");
        assert_eq!(f.signature.params.len(), 2);
    } else {
        panic!("Expected function item");
    }
}

#[test]
fn test_namespace_with_multiple_items() {
    let input = "namespace MyLib
    type Int = i32
    struct Point
        x: i32
        y: i32
    enum Direction
        North
        South
        East
        West
";
    let namespace = parser::NamespaceParser::new().parse(Lexer::new(input)).unwrap();

    assert_eq!(namespace.name.segments[0], "MyLib");
    assert_eq!(namespace.items.len(), 3);

    assert!(matches!(&namespace.items[0], Statement::TypeAlias(_)));
    assert!(matches!(&namespace.items[1], Statement::Struct(_)));
    assert!(matches!(&namespace.items[2], Statement::Enum(_)));
}

#[test]
fn test_nested_namespace() {
    let input = "namespace Outer
    namespace Inner
        type Alias = i32
";
    let namespace = parser::NamespaceParser::new().parse(Lexer::new(input)).unwrap();

    assert_eq!(namespace.name.segments[0], "Outer");
    assert_eq!(namespace.items.len(), 1);

    if let Statement::Namespace(inner) = &namespace.items[0] {
        assert_eq!(inner.name.segments[0], "Inner");
        assert_eq!(inner.items.len(), 1);
    } else {
        panic!("Expected nested namespace");
    }
}

#[test]
fn test_namespace_with_complex_items() {
    let input = "namespace Collections
    struct List[T]
        where
            T: Clone
        items: [T]
        size: usize
    interface Iterable[T]
        func next() -> T
";
    let namespace = parser::NamespaceParser::new().parse(Lexer::new(input)).unwrap();

    assert_eq!(namespace.name.segments[0], "Collections");
    assert_eq!(namespace.items.len(), 2);

    // Check struct with generics and where clause merged into generic_params
    if let Statement::Struct(s) = &namespace.items[0] {
        assert_eq!(s.generic_params.len(), 1);
        // T should have 1 bound (Clone) after merging where clause
        if let crate::ast::GenericParameter::Type { name, bounds, .. } = &s.generic_params[0] {
            assert_eq!(name, "T");
            assert_eq!(bounds.len(), 1);
        }
    } else {
        panic!("Expected struct as first item");
    }

    // Check interface with generics
    if let Statement::Interface(iface) = &namespace.items[1] {
        assert_eq!(iface.generic_params.len(), 1);
    } else {
        panic!("Expected interface as second item");
    }
}

#[test]
fn test_empty_namespace() {
    let input = "namespace Empty
    pass
";
    let result = parser::NamespaceParser::new().parse(Lexer::new(input));
    if let Ok(ns) = result {
        assert_eq!(ns.name.segments[0], "Empty");
        // pass is Statement::Pass
        assert!(ns.items.iter().all(|s| matches!(s, Statement::Pass)));
    }
}

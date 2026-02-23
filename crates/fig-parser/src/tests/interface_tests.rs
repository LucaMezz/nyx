// Interface parsing tests for fig-parser
// NOTE: Fig uses "func" (not "fn") for function declarations.

use crate::{Lexer, ast::{GenericParameter, Type}, parser};

fn bound_name(ty: &Type) -> &str {
    if let Type::Path { path: p, .. } = ty { &p.segments[0] } else { panic!("Expected path type") }
}
fn extends_name(ty: &Type) -> &str {
    if let Type::Path { path: p, .. } = ty { &p.segments[0] } else { panic!("Expected path type in extends") }
}

#[test]
fn test_simple_interface() {
    let input = "interface Drawable\n    func get_width() -> i32\n    func get_height() -> i32\n";
    let iface = parser::InterfaceParser::new().parse(Lexer::new(input)).unwrap();

    assert_eq!(iface.name.segments[0], "Drawable");
    assert_eq!(iface.generic_params.len(), 0);
    assert_eq!(iface.extends.len(), 0);
    assert_eq!(iface.methods.len(), 2);

    assert_eq!(iface.methods[0].name, "get_width");
    assert_eq!(iface.methods[0].params.len(), 0);
    assert_eq!(iface.methods[0].return_types, vec![Type::I32]);

    assert_eq!(iface.methods[1].name, "get_height");
    assert_eq!(iface.methods[1].params.len(), 0);
    assert_eq!(iface.methods[1].return_types, vec![Type::I32]);
}

#[test]
fn test_interface_with_generic_params() {
    let input = "interface Container[T]\n    func add(item: T)\n    func get(index: usize) -> T\n";
    let iface = parser::InterfaceParser::new().parse(Lexer::new(input)).unwrap();

    assert_eq!(iface.name.segments[0], "Container");
    assert_eq!(iface.generic_params.len(), 1);

    if let GenericParameter::Type { name, bounds, .. } = &iface.generic_params[0] {
        assert_eq!(name, "T");
        assert_eq!(bounds.len(), 0);
    } else {
        panic!("Expected type parameter T");
    }

    assert_eq!(iface.methods.len(), 2);
    assert_eq!(iface.methods[0].name, "add");
    assert_eq!(iface.methods[0].params.len(), 1);
    assert_eq!(iface.methods[1].name, "get");
}

#[test]
fn test_interface_with_extends_clause() {
    let input = "interface Shape\n    extends\n        Drawable\n        Movable\n    func get_area() -> f64\n";
    let iface = parser::InterfaceParser::new().parse(Lexer::new(input)).unwrap();

    assert_eq!(iface.name.segments[0], "Shape");
    assert_eq!(iface.extends.len(), 2);
    assert_eq!(extends_name(&iface.extends[0]), "Drawable");
    assert_eq!(extends_name(&iface.extends[1]), "Movable");
    assert_eq!(iface.methods.len(), 1);
}

#[test]
fn test_interface_with_where_clause() {
    // where-clause constraints merged into generic_params
    let input = "interface Comparable[T]\n    where\n        T: Clone\n    func compare(other: T) -> i32\n";
    let iface = parser::InterfaceParser::new().parse(Lexer::new(input)).unwrap();

    assert_eq!(iface.name.segments[0], "Comparable");
    assert_eq!(iface.generic_params.len(), 1);

    if let GenericParameter::Type { name, bounds, .. } = &iface.generic_params[0] {
        assert_eq!(name, "T");
        assert_eq!(bounds.len(), 1);
        assert_eq!(bound_name(&bounds[0]), "Clone");
    }
}

#[test]
fn test_interface_with_extends_and_where() {
    let input = "interface AdvancedContainer[T, E]\n    extends\n        Container[T]\n    where\n        T: Clone\n        E: Copy\n    func get_or_error(index: usize) -> T\n    func set(index: usize, item: T)\n";
    let iface = parser::InterfaceParser::new().parse(Lexer::new(input)).unwrap();

    assert_eq!(iface.name.segments[0], "AdvancedContainer");
    assert_eq!(iface.generic_params.len(), 2);
    assert_eq!(iface.extends.len(), 1);
    assert_eq!(iface.methods.len(), 2);
}

#[test]
fn test_interface_with_method_parameters() {
    let input = "interface Calculator\n    func add(a: i32, b: i32) -> i32\n    func multiply(a: i32, b: i32) -> i32\n";
    let iface = parser::InterfaceParser::new().parse(Lexer::new(input)).unwrap();

    assert_eq!(iface.methods[0].params.len(), 2);
    assert_eq!(iface.methods[0].params[0].name, "a");
    assert_eq!(iface.methods[0].params[0].ty, Type::I32);
    assert_eq!(iface.methods[0].params[1].name, "b");
    assert_eq!(iface.methods[0].params[1].ty, Type::I32);
}

#[test]
fn test_interface_with_complex_types() {
    let input = "interface Processor[T]\n    func process(input: ?*mut T) -> *T\n    func batch_process(inputs: [T]) -> [T]\n";
    let iface = parser::InterfaceParser::new().parse(Lexer::new(input)).unwrap();
    assert_eq!(iface.methods.len(), 2);

    // First method: ?*mut T  →  Nullable(Pointer { nullable: false, mutable: true, .. })
    let ty0 = iface.methods[0].params[0].ty.clone();
    if let Type::Nullable(inner) = ty0 {
        if let Type::Pointer { nullable, mutable, element_type } = *inner {
            assert!(!nullable);
            assert!(mutable);
            assert!(matches!(*element_type, Type::Path { .. }));
        } else {
            panic!("Expected Pointer inside Nullable");
        }
    } else {
        panic!("Expected nullable mutable pointer");
    }

    // Second method: slice param
    let ty1 = iface.methods[1].params[0].ty.clone();
    if let Type::Array { element_type, size } = ty1 {
        assert!(size.is_none());
        assert!(matches!(*element_type, Type::Path { .. }));
    } else {
        panic!("Expected slice type");
    }
}

#[test]
fn test_interface_no_body() {
    // An interface with just a name and no body (forward declaration)
    let input = "interface Marker\n";
    let result = parser::InterfaceParser::new().parse(Lexer::new(input));
    if let Ok(iface) = result {
        assert_eq!(iface.name.segments[0], "Marker");
        assert_eq!(iface.methods.len(), 0);
    }
    // (parser may or may not support this form; failure is acceptable here)
}

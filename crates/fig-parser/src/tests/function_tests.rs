// Comprehensive tests for Function parsing
// NOTE: Fig uses "func" (not "fn") for function declarations.
// NOTE: Generic params come BEFORE the function name: func[T] name(...)
// NOTE: Function bodies require a trailing newline after the last statement.

use crate::ast::*;
use super::helpers;

fn bound_name(ty: &Type) -> &str {
    if let Type::Path { path: p, .. } = ty { &p.segments[0] } else { panic!("Expected path type as bound") }
}

#[test]
fn test_simple_function() {
    let input = "func add(x: i32, y: i32) -> i32\n    pass\n";
    let f = helpers::parse_function(input).unwrap();
    let sig = f.signature;
    assert_eq!(sig.name, "add");
    assert_eq!(sig.params.len(), 2);
    assert_eq!(sig.return_types, vec![Type::I32]);
    assert_eq!(sig.outer_generic_params.len(), 0);

    assert_eq!(sig.params[0].name, "x");
    assert_eq!(sig.params[0].ty, Type::I32);
    assert_eq!(sig.params[1].name, "y");
    assert_eq!(sig.params[1].ty, Type::I32);
}

#[test]
fn test_most_basic_function() {
    let input = "func main()\n    pass\n";
    let f = helpers::parse_function(input).unwrap();
    let sig = f.signature;
    assert_eq!(sig.name, "main");
    assert_eq!(sig.params.len(), 0);
    assert_eq!(sig.return_types, vec![]);
}

#[test]
fn test_function_no_params() {
    let input = "func main() -> i32\n    pass\n";
    let f = helpers::parse_function(input).unwrap();
    let sig = f.signature;
    assert_eq!(sig.name, "main");
    assert_eq!(sig.params.len(), 0);
    assert_eq!(sig.return_types, vec![Type::I32]);
}

#[test]
fn test_function_no_return_type() {
    let input = "func print(x: i32)\n    pass\n";
    let f = helpers::parse_function(input).unwrap();
    let sig = f.signature;
    assert_eq!(sig.name, "print");
    assert_eq!(sig.params.len(), 1);
    assert_eq!(sig.return_types, vec![]);
}

#[test]
fn test_function_with_generic_params() {
    // Generic params come BEFORE the function name: func[T] identity(x: T) -> T
    let input = "func[T] identity(x: T) -> T\n    pass\n";
    let f = helpers::parse_function(input).unwrap();
    let sig = f.signature;
    assert_eq!(sig.name, "identity");
    assert_eq!(sig.outer_generic_params.len(), 1);
    assert_eq!(sig.params.len(), 1);

    if let GenericParameter::Type { name, bounds, .. } = &sig.outer_generic_params[0] {
        assert_eq!(name, "T");
        assert_eq!(bounds.len(), 0);
    } else {
        panic!("Expected type parameter");
    }
}

#[test]
fn test_function_with_bounded_generic() {
    let input = "func[T: Clone] clone(x: T) -> T\n    pass\n";
    let f = helpers::parse_function(input).unwrap();
    let sig = f.signature;
    assert_eq!(sig.outer_generic_params.len(), 1);

    if let GenericParameter::Type { name, bounds, .. } = &sig.outer_generic_params[0] {
        assert_eq!(name, "T");
        assert_eq!(bounds.len(), 1);
        assert_eq!(bound_name(&bounds[0]), "Clone");
    } else {
        panic!("Expected bounded type parameter");
    }
}

#[test]
fn test_function_with_multiple_generics() {
    let input = "func[T, U] pair(x: T, y: U) -> T\n    pass\n";
    let f = helpers::parse_function(input).unwrap();
    let sig = f.signature;
    assert_eq!(sig.outer_generic_params.len(), 2);
    assert_eq!(sig.params.len(), 2);
}

#[test]
fn test_function_with_where_clause() {
    // Where-clause constraints are merged into generic_params bounds.
    let input = "func[T, U] process(x: T, y: U) -> T\n    where\n        T: Clone\n        U: Copy\n    pass\n";
    let f = helpers::parse_function(input).unwrap();
    let sig = f.signature;
    // T and U appear in generic_params with bounds merged in from the where clause
    assert_eq!(sig.outer_generic_params.len(), 2);

    if let GenericParameter::Type { name, bounds, .. } = &sig.outer_generic_params[0] {
        assert_eq!(name, "T");
        assert_eq!(bounds.len(), 1);
        assert_eq!(bound_name(&bounds[0]), "Clone");
    } else {
        panic!("Expected T: Clone");
    }

    if let GenericParameter::Type { name, bounds, .. } = &sig.outer_generic_params[1] {
        assert_eq!(name, "U");
        assert_eq!(bounds.len(), 1);
        assert_eq!(bound_name(&bounds[0]), "Copy");
    } else {
        panic!("Expected U: Copy");
    }
}

#[test]
fn test_function_with_complex_types() {
    let input = "func[T] process(ptr: *i32, arr: [u8], ref_val: *mut T) -> *u32\n    pass\n";
    let f = helpers::parse_function(input).unwrap();
    let sig = f.signature;
    assert_eq!(sig.params.len(), 3);

    // ptr: *i32
    assert_eq!(sig.params[0].name, "ptr");
    if let Type::Pointer { element_type, nullable, mutable } = &sig.params[0].ty {
        assert!(!nullable);
        assert!(!mutable);
        assert_eq!(**element_type, Type::I32);
    } else {
        panic!("Expected pointer type");
    }

    // arr: [u8]
    assert_eq!(sig.params[1].name, "arr");
    if let Type::Array { element_type, size } = &sig.params[1].ty {
        assert!(size.is_none());
        assert_eq!(**element_type, Type::U8);
    } else {
        panic!("Expected slice type");
    }

    // ref_val: *mut T
    assert_eq!(sig.params[2].name, "ref_val");
    if let Type::Pointer { mutable, .. } = &sig.params[2].ty {
        assert!(mutable);
    } else {
        panic!("Expected mutable pointer type");
    }

    // return type *u32
    if let Some(Type::Pointer { element_type, nullable, mutable }) = sig.return_types.first() {
        assert!(!nullable);
        assert!(!mutable);
        assert_eq!(**element_type, Type::U32);
    } else {
        panic!("Expected pointer return type");
    }
}

#[test]
fn test_function_with_array_return() {
    let input = "func get_array() -> [i32]\n    pass\n";
    let f = helpers::parse_function(input).unwrap();
    let sig = f.signature;

    if let Some(Type::Array { element_type, size }) = sig.return_types.first() {
        assert!(size.is_none());
        assert_eq!(**element_type, Type::I32);
    } else {
        panic!("Expected array return type");
    }
}

#[test]
fn test_function_with_const_param() {
    let input = "func[const N: usize] array_func(x: i32) -> i32\n    pass\n";
    let f = helpers::parse_function(input).unwrap();
    let sig = f.signature;
    assert_eq!(sig.outer_generic_params.len(), 1);

    if let GenericParameter::Const { name, ty } = &sig.outer_generic_params[0] {
        assert_eq!(name, "N");
        assert_eq!(ty, &Type::USize);
    } else {
        panic!("Expected const parameter");
    }
}

#[test]
fn test_function_with_mixed_generics() {
    let input = "func[T: Clone, const N: usize, U] complex(x: T, y: U) -> T\n    pass\n";
    let f = helpers::parse_function(input).unwrap();
    let sig = f.signature;
    assert_eq!(sig.outer_generic_params.len(), 3);

    if let GenericParameter::Type { name, bounds, .. } = &sig.outer_generic_params[0] {
        assert_eq!(name, "T");
        assert_eq!(bounds.len(), 1);
    }
    if let GenericParameter::Const { name, ty } = &sig.outer_generic_params[1] {
        assert_eq!(name, "N");
        assert_eq!(ty, &Type::USize);
    }
    if let GenericParameter::Type { name, bounds, .. } = &sig.outer_generic_params[2] {
        assert_eq!(name, "U");
        assert_eq!(bounds.len(), 0);
    }
}

#[test]
fn test_function_with_multiple_bounds() {
    let input = "func[T: Clone + Copy + Send] multi_bound(x: T) -> T\n    pass\n";
    let f = helpers::parse_function(input).unwrap();
    let sig = f.signature;

    if let GenericParameter::Type { name, bounds, .. } = &sig.outer_generic_params[0] {
        assert_eq!(name, "T");
        assert_eq!(bounds.len(), 3);
        assert_eq!(bound_name(&bounds[0]), "Clone");
        assert_eq!(bound_name(&bounds[1]), "Copy");
        assert_eq!(bound_name(&bounds[2]), "Send");
    } else {
        panic!("Expected bounded type parameter");
    }
}

#[test]
fn test_function_with_single_param() {
    let input = "func square(x: i32) -> i32\n    pass\n";
    let f = helpers::parse_function(input).unwrap();
    assert_eq!(f.signature.params.len(), 1);
}

#[test]
fn test_function_with_trailing_comma() {
    let input = "func add(x: i32, y: i32,) -> i32\n    pass\n";
    let f = helpers::parse_function(input).unwrap();
    assert_eq!(f.signature.params.len(), 2);
}

// Tests for type alias parsing
// NOTE: where-clause constraints are merged into generic_params by the parser.

use crate::{
    Lexer,
    ast::{GenericParameter, Type},
    parser,
};

fn bound_name(ty: &Type) -> &str {
    if let Type::Path { path: p, .. } = ty { &p.segments[0] } else { panic!("Expected path type as bound") }
}

#[test]
fn test_simple_type_alias() {
    let input = "type MyInt = i32";
    let ta = parser::TypeAliasParser::new().parse(Lexer::new(input)).unwrap();
    assert_eq!(ta.name.segments[0], "MyInt");
    assert_eq!(ta.generic_params.len(), 0);
    assert_eq!(ta.aliased_type, Type::I32);
}

#[test]
fn test_type_alias_with_generic_params() {
    let input = "type MyVec[T] = [T]";
    let ta = parser::TypeAliasParser::new().parse(Lexer::new(input)).unwrap();
    assert_eq!(ta.name.segments[0], "MyVec");
    assert_eq!(ta.generic_params.len(), 1);

    if let GenericParameter::Type { name, bounds, .. } = &ta.generic_params[0] {
        assert_eq!(name, "T");
        assert_eq!(bounds.len(), 0);
    } else {
        panic!("Expected type parameter");
    }

    if let Type::Array { element_type, size } = &ta.aliased_type {
        assert!(size.is_none());
        // element type is T (a path type)
        assert!(matches!(element_type.as_ref(), Type::Path { .. }));
    } else {
        panic!("Expected slice type");
    }
}

#[test]
fn test_type_alias_with_bounded_generic() {
    let input = "type MyPtr[T: Copy] = *T";
    let ta = parser::TypeAliasParser::new().parse(Lexer::new(input)).unwrap();
    assert_eq!(ta.name.segments[0], "MyPtr");
    assert_eq!(ta.generic_params.len(), 1);

    if let GenericParameter::Type { name, bounds, .. } = &ta.generic_params[0] {
        assert_eq!(name, "T");
        assert_eq!(bounds.len(), 1);
        assert_eq!(bound_name(&bounds[0]), "Copy");
    } else {
        panic!("Expected type parameter");
    }
}

#[test]
fn test_type_alias_with_multiple_bounds() {
    let input = "type MyType[T: Clone + Copy] = *T";
    let ta = parser::TypeAliasParser::new().parse(Lexer::new(input)).unwrap();

    if let GenericParameter::Type { name, bounds, .. } = &ta.generic_params[0] {
        assert_eq!(name, "T");
        assert_eq!(bounds.len(), 2);
        assert_eq!(bound_name(&bounds[0]), "Clone");
        assert_eq!(bound_name(&bounds[1]), "Copy");
    } else {
        panic!("Expected type parameter");
    }
}

#[test]
fn test_type_alias_with_const_param() {
    let input = "type MyArray[const N: usize] = i32";
    let ta = parser::TypeAliasParser::new().parse(Lexer::new(input)).unwrap();
    assert_eq!(ta.generic_params.len(), 1);

    if let GenericParameter::Const { name, ty } = &ta.generic_params[0] {
        assert_eq!(name, "N");
        assert_eq!(ty, &Type::USize);
    } else {
        panic!("Expected const parameter");
    }
}

#[test]
fn test_type_alias_with_where_clause() {
    // where-clause is merged into generic_params
    let input = "type MyResult[T, E]\n    where\n        T: Clone\n        E: Copy\n    = i32";
    let ta = parser::TypeAliasParser::new().parse(Lexer::new(input)).unwrap();
    assert_eq!(ta.name.segments[0], "MyResult");
    // T and E appear in generic_params with bounds merged in
    assert_eq!(ta.generic_params.len(), 2);

    if let GenericParameter::Type { name, bounds, .. } = &ta.generic_params[0] {
        assert_eq!(name, "T");
        assert_eq!(bounds.len(), 1);
        assert_eq!(bound_name(&bounds[0]), "Clone");
    } else { panic!("Expected T: Clone"); }

    if let GenericParameter::Type { name, bounds, .. } = &ta.generic_params[1] {
        assert_eq!(name, "E");
        assert_eq!(bounds.len(), 1);
        assert_eq!(bound_name(&bounds[0]), "Copy");
    } else { panic!("Expected E: Copy"); }
}

#[test]
fn test_type_alias_with_complex_where_clause() {
    let input = "type ComplexType[T, U, V]\n    where\n        T: Clone + Send\n        U: Copy\n        V: Debug + Display\n    = *T";
    let ta = parser::TypeAliasParser::new().parse(Lexer::new(input)).unwrap();
    assert_eq!(ta.generic_params.len(), 3);

    if let GenericParameter::Type { name, bounds, .. } = &ta.generic_params[0] {
        assert_eq!(name, "T"); assert_eq!(bounds.len(), 2);
    } else { panic!("Expected T with 2 bounds"); }

    if let GenericParameter::Type { name, bounds, .. } = &ta.generic_params[1] {
        assert_eq!(name, "U"); assert_eq!(bounds.len(), 1);
    } else { panic!("Expected U: Copy"); }

    if let GenericParameter::Type { name, bounds, .. } = &ta.generic_params[2] {
        assert_eq!(name, "V"); assert_eq!(bounds.len(), 2);
    } else { panic!("Expected V with 2 bounds"); }
}

#[test]
fn test_type_alias_with_mixed_params_and_where_clause() {
    // T: Clone in param list, where clause adds Send → merged: T has [Clone, Send]
    let input = "type MixedType[T: Clone, const N: usize]\n    where\n        T: Send\n    = *T";
    let ta = parser::TypeAliasParser::new().parse(Lexer::new(input)).unwrap();
    assert_eq!(ta.generic_params.len(), 2);

    if let GenericParameter::Type { name, bounds, .. } = &ta.generic_params[0] {
        assert_eq!(name, "T");
        assert_eq!(bounds.len(), 2); // Clone + Send merged
    }
    if let GenericParameter::Const { name, ty } = &ta.generic_params[1] {
        assert_eq!(name, "N");
        assert_eq!(ty, &Type::USize);
    }
}

#[test]
fn test_type_alias_pointer_type() {
    let input = "type RawPtr = *Something";
    let ta = parser::TypeAliasParser::new().parse(Lexer::new(input)).unwrap();
    if let Type::Pointer { nullable, mutable, element_type } = ta.aliased_type {
        assert!(!nullable);
        assert!(!mutable);
        assert!(matches!(*element_type, Type::Path { .. }));
    } else {
        panic!("Expected pointer type");
    }
}

#[test]
fn test_type_alias_slice_type() {
    let input = "type Bytes = [u8]";
    let ta = parser::TypeAliasParser::new().parse(Lexer::new(input)).unwrap();
    if let Type::Array { element_type, size } = ta.aliased_type {
        assert!(size.is_none());
        assert_eq!(*element_type, Type::U8);
    } else {
        panic!("Expected slice type");
    }
}

#[test]
fn test_type_alias_fixed_array_type() {
    let input = "type SmallBuf = [i32; 10]";
    let ta = parser::TypeAliasParser::new().parse(Lexer::new(input)).unwrap();
    if let Type::Array { element_type, size } = ta.aliased_type {
        assert!(size.is_some());
        assert_eq!(*element_type, Type::I32);
    } else {
        panic!("Expected fixed array type");
    }
}

#[test]
fn test_type_alias_error_union() {
    let input = "type Result[T] = T ! IoError";
    let ta = parser::TypeAliasParser::new().parse(Lexer::new(input)).unwrap();
    if let Type::ErrorUnion { ok_type, err_type } = ta.aliased_type {
        // ok_type is the path T
        assert!(matches!(*ok_type, Type::Path { .. }));
        if let Type::Path { path, .. } = *err_type { assert_eq!(path.segments[0], "IoError"); } else { panic!("Expected Path type for err_type"); }
    } else {
        panic!("Expected ErrorUnion type");
    }
}

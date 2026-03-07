// Comprehensive tests for Struct, Enum, and Union parsing

fn path_type_name(ty: &crate::ast::Type) -> &str {
    if let crate::ast::Type::Path { path: p, .. } = ty { &p.segments[0] } else { panic!("Expected path type") }
}
fn bound_name(ty: &crate::ast::Type) -> &str { path_type_name(ty) }

#[cfg(test)]
mod struct_tests {
    use super::*;
    use crate::ast::*;
    use super::super::helpers;

    #[test]
    fn test_simple_struct() {
        let input = "struct Point\n    x: i32\n    y: i32\n";
        let s = helpers::parse_struct(input).unwrap();
        assert_eq!(s.name.segments[0], "Point");
        assert_eq!(s.fields.len(), 2);
        assert_eq!(s.generic_params.len(), 0);
        assert_eq!(s.requires.len(), 0);

        assert_eq!(s.fields[0].name, "x");
        assert!(matches!(s.fields[0].ty, Type::I32));
        assert_eq!(s.fields[1].name, "y");
        assert!(matches!(s.fields[1].ty, Type::I32));
    }

    #[test]
    fn test_struct_with_generic_params() {
        let input = "struct Container[T]\n    value: T\n";
        let s = helpers::parse_struct(input).unwrap();
        assert_eq!(s.name.segments[0], "Container");
        assert_eq!(s.generic_params.len(), 1);
        assert_eq!(s.fields.len(), 1);

        if let GenericParameter::Type { name, bounds, .. } = &s.generic_params[0] {
            assert_eq!(name, "T");
            assert_eq!(bounds.len(), 0);
        } else {
            panic!("Expected type parameter");
        }
    }

    #[test]
    fn test_struct_with_bounded_generic() {
        let input = "struct Container[T: Clone]\n    value: T\n";
        let s = helpers::parse_struct(input).unwrap();
        if let GenericParameter::Type { name, bounds, .. } = &s.generic_params[0] {
            assert_eq!(name, "T");
            assert_eq!(bounds.len(), 1);
        } else {
            panic!("Expected bounded type parameter");
        }
    }

    #[test]
    fn test_struct_with_multiple_bounded_generics() {
        let input = "struct Pair[T: Clone + Copy, U: Send]\n    first: T\n    second: U\n";
        let s = helpers::parse_struct(input).unwrap();
        assert_eq!(s.generic_params.len(), 2);
        assert_eq!(s.fields.len(), 2);

        if let GenericParameter::Type { name, bounds, .. } = &s.generic_params[0] {
            assert_eq!(name, "T");
            assert_eq!(bounds.len(), 2);
        }
        if let GenericParameter::Type { name, bounds, .. } = &s.generic_params[1] {
            assert_eq!(name, "U");
            assert_eq!(bounds.len(), 1);
        }
    }

    #[test]
    fn test_struct_with_const_param() {
        let input = "struct Array[const N: usize]\n    data: i32\n";
        let s = helpers::parse_struct(input).unwrap();
        assert_eq!(s.generic_params.len(), 1);
        if let GenericParameter::Const { name, ty } = &s.generic_params[0] {
            assert_eq!(name, "N");
            assert_eq!(ty, &Type::USize);
        } else {
            panic!("Expected const parameter");
        }
    }

    #[test]
    fn test_struct_with_where_clause() {
        // where-clause constraints are merged into generic_params
        let input = "struct Container[T, U]\n    where\n        T: Clone\n        U: Copy\n    first: T\n    second: U\n";
        let s = helpers::parse_struct(input).unwrap();
        assert_eq!(s.generic_params.len(), 2);

        if let GenericParameter::Type { name, bounds, .. } = &s.generic_params[0] {
            assert_eq!(name, "T");
            assert_eq!(bounds.len(), 1);
            assert_eq!(bound_name(&bounds[0]), "Clone");
        }
        if let GenericParameter::Type { name, bounds, .. } = &s.generic_params[1] {
            assert_eq!(name, "U");
            assert_eq!(bounds.len(), 1);
            assert_eq!(bound_name(&bounds[0]), "Copy");
        }
    }

    #[test]
    fn test_struct_with_requires_clause() {
        let input = "struct Container[T]\n    requires\n        Clone\n        Send\n    value: T\n";
        let s = helpers::parse_struct(input).unwrap();
        assert_eq!(s.requires.len(), 2);
        assert_eq!(path_type_name(&s.requires[0]), "Clone");
        assert_eq!(path_type_name(&s.requires[1]), "Send");
    }

    #[test]
    fn test_struct_with_requires_and_where() {
        let input = "struct Complex[T, U]\n    requires\n        Debug\n    where\n        T: Clone\n        U: Copy\n    first: T\n    second: U\n";
        let s = helpers::parse_struct(input).unwrap();
        assert_eq!(s.requires.len(), 1);
        assert_eq!(s.generic_params.len(), 2);
        // Both T and U should have bounds from the merged where clause
        if let GenericParameter::Type { bounds, .. } = &s.generic_params[0] { assert_eq!(bounds.len(), 1); }
        if let GenericParameter::Type { bounds, .. } = &s.generic_params[1] { assert_eq!(bounds.len(), 1); }
    }

    #[test]
    fn test_struct_with_complex_types() {
        let input = "struct ComplexStruct\n    pointer: ?*i32\n    array: [u8]\n    reference: *mut bool\n";
        let s = helpers::parse_struct(input).unwrap();
        assert_eq!(s.fields.len(), 3);

        // pointer: ?*i32  →  Nullable(Pointer { nullable: false, .. })
        let ty0 = s.fields[0].ty.clone();
        if let Type::Nullable(inner) = ty0 {
            if let Type::Pointer { element_type, nullable, mutable } = *inner {
                assert!(!nullable);
                assert!(!mutable);
                assert_eq!(*element_type, Type::I32);
            } else { panic!("Expected Pointer inside Nullable"); }
        } else { panic!("Expected nullable pointer"); }

        // array: [u8]
        let ty1 = s.fields[1].ty.clone();
        if let Type::Array { element_type, size } = ty1 {
            assert!(size.is_none());
            assert_eq!(*element_type, Type::U8);
        } else { panic!("Expected slice type"); }

        // reference: *mut bool
        let ty2 = s.fields[2].ty.clone();
        if let Type::Pointer { element_type, nullable, mutable } = ty2 {
            assert!(!nullable);
            assert!(mutable);
            assert_eq!(*element_type, Type::Bool);
        } else { panic!("Expected mutable pointer"); }
    }

    #[test]
    fn test_struct_with_single_field() {
        let input = "struct Wrapper\n    value: i32\n";
        let s = helpers::parse_struct(input).unwrap();
        assert_eq!(s.fields.len(), 1);
    }
}

#[cfg(test)]
mod enum_tests {
    use super::*;
    use crate::ast::*;
    use super::super::helpers;

    #[test]
    fn test_simple_enum() {
        let input = "enum Color\n    Red\n    Green\n    Blue\n";
        let e = helpers::parse_enum(input).unwrap();
        assert_eq!(e.name.segments[0], "Color");
        assert_eq!(e.variants.len(), 3);
        assert_eq!(e.generic_params.len(), 0);
        assert_eq!(e.requires.len(), 0);
        assert_eq!(e.representation, None);

        assert_eq!(e.variants[0].name, "Red");
        assert!(e.variants[0].value.is_none());
        assert_eq!(e.variants[1].name, "Green");
        assert_eq!(e.variants[2].name, "Blue");
    }

    #[test]
    fn test_enum_with_representation() {
        let input = "enum[u8] Color\n    Red\n    Green\n    Blue\n";
        let e = helpers::parse_enum(input).unwrap();
        assert_eq!(e.name.segments[0], "Color");
        assert_eq!(e.variants.len(), 3);
        assert_eq!(e.representation, Some(Type::U8));
    }

    #[test]
    fn test_enum_with_i32_representation() {
        let input = "enum[i32] Status\n    Ok = 0\n    Error = 1\n    Pending = 2\n";
        let e = helpers::parse_enum(input).unwrap();
        assert_eq!(e.name.segments[0], "Status");
        assert_eq!(e.representation, Some(Type::I32));
        assert_eq!(e.variants.len(), 3);
    }

    #[test]
    fn test_enum_with_usize_representation() {
        let input = "enum[usize] Flags\n    None = 0\n    Read = 1\n    Write = 2\n    Execute = 4\n";
        let e = helpers::parse_enum(input).unwrap();
        assert_eq!(e.representation, Some(Type::USize));
    }

    #[test]
    fn test_enum_with_discriminants() {
        let input = "enum Status\n    Ok = 0\n    Error = 1\n    Pending = 2\n";
        let e = helpers::parse_enum(input).unwrap();
        assert_eq!(e.variants.len(), 3);
        assert!(e.variants[0].value.is_some());
        assert!(e.variants[1].value.is_some());
        assert!(e.variants[2].value.is_some());
    }

    #[test]
    fn test_enum_mixed_discriminants() {
        let input = "enum Mixed\n    A\n    B = 10\n    C\n";
        let e = helpers::parse_enum(input).unwrap();
        assert_eq!(e.variants.len(), 3);
        assert!(e.variants[0].value.is_none());
        assert!(e.variants[1].value.is_some());
        assert!(e.variants[2].value.is_none());
    }

    #[test]
    fn test_enum_with_generic_params() {
        let input = "enum Option[T]\n    Some\n    None\n";
        let e = helpers::parse_enum(input).unwrap();
        assert_eq!(e.name.segments[0], "Option");
        assert_eq!(e.generic_params.len(), 1);
        if let GenericParameter::Type { name, bounds, .. } = &e.generic_params[0] {
            assert_eq!(name, "T");
            assert_eq!(bounds.len(), 0);
        } else { panic!("Expected type parameter"); }
    }

    #[test]
    fn test_enum_with_bounded_generic() {
        let input = "enum Result[T: Clone, E: Copy]\n    Ok\n    Err\n";
        let e = helpers::parse_enum(input).unwrap();
        assert_eq!(e.generic_params.len(), 2);
        if let GenericParameter::Type { name, bounds, .. } = &e.generic_params[0] {
            assert_eq!(name, "T"); assert_eq!(bounds.len(), 1);
        }
        if let GenericParameter::Type { name, bounds, .. } = &e.generic_params[1] {
            assert_eq!(name, "E"); assert_eq!(bounds.len(), 1);
        }
    }

    #[test]
    fn test_enum_with_where_clause() {
        // where-clause constraints merged into generic_params
        let input = "enum Container[T, E]\n    where\n        T: Clone\n        E: Send\n    Some\n    None\n";
        let e = helpers::parse_enum(input).unwrap();
        assert_eq!(e.generic_params.len(), 2);
        if let GenericParameter::Type { name, bounds, .. } = &e.generic_params[0] {
            assert_eq!(name, "T"); assert_eq!(bounds.len(), 1);
        }
        if let GenericParameter::Type { name, bounds, .. } = &e.generic_params[1] {
            assert_eq!(name, "E"); assert_eq!(bounds.len(), 1);
        }
    }

    #[test]
    fn test_enum_with_requires_clause() {
        let input = "enum MyEnum[T]\n    requires\n        Clone\n        Copy\n    Variant1\n    Variant2\n";
        let e = helpers::parse_enum(input).unwrap();
        assert_eq!(e.requires.len(), 2);
        assert_eq!(path_type_name(&e.requires[0]), "Clone");
        assert_eq!(path_type_name(&e.requires[1]), "Copy");
    }

    #[test]
    fn test_enum_with_requires_and_where() {
        let input = "enum Complex[T, E]\n    requires\n        Debug\n    where\n        T: Clone\n        E: Copy\n    VariantA\n    VariantB\n";
        let e = helpers::parse_enum(input).unwrap();
        assert_eq!(e.requires.len(), 1);
        assert_eq!(e.generic_params.len(), 2);
    }

    #[test]
    fn test_enum_single_variant() {
        let input = "enum Unit\n    Value\n";
        let e = helpers::parse_enum(input).unwrap();
        assert_eq!(e.variants.len(), 1);
        assert_eq!(e.variants[0].name, "Value");
    }

    #[test]
    fn test_enum_with_const_param() {
        let input = "enum Array[const N: usize]\n    Empty\n    Full\n";
        let e = helpers::parse_enum(input).unwrap();
        assert_eq!(e.generic_params.len(), 1);
        if let GenericParameter::Const { name, ty } = &e.generic_params[0] {
            assert_eq!(name, "N"); assert_eq!(ty, &Type::USize);
        } else { panic!("Expected const parameter"); }
    }
}

#[cfg(test)]
mod union_tests {
    use super::*;
    use crate::ast::*;
    use super::super::helpers;

    #[test]
    fn test_simple_union() {
        let input = "union Value\n    int_val: i32\n    float_val: f32\n";
        let u = helpers::parse_union(input).unwrap();
        assert_eq!(u.name.segments[0], "Value");
        assert_eq!(u.variants.len(), 2);
        assert_eq!(u.generic_params.len(), 0);
        assert_eq!(u.requires.len(), 0);

        assert_eq!(u.variants[0].name, "int_val");
        assert!(matches!(u.variants[0].ty, Type::I32));
        assert_eq!(u.variants[1].name, "float_val");
        assert!(matches!(u.variants[1].ty, Type::F32));
    }

    #[test]
    fn test_union_with_generic_params() {
        let input = "union Container[T]\n    value: T\n    pointer: *T\n";
        let u = helpers::parse_union(input).unwrap();
        assert_eq!(u.name.segments[0], "Container");
        assert_eq!(u.generic_params.len(), 1);
        if let GenericParameter::Type { name, bounds, .. } = &u.generic_params[0] {
            assert_eq!(name, "T"); assert_eq!(bounds.len(), 0);
        } else { panic!("Expected type parameter"); }
    }

    #[test]
    fn test_union_with_bounded_generic() {
        let input = "union Data[T: Clone, U: Copy]\n    first: T\n    second: U\n";
        let u = helpers::parse_union(input).unwrap();
        assert_eq!(u.generic_params.len(), 2);
        if let GenericParameter::Type { name, bounds, .. } = &u.generic_params[0] {
            assert_eq!(name, "T"); assert_eq!(bounds.len(), 1);
        }
        if let GenericParameter::Type { name, bounds, .. } = &u.generic_params[1] {
            assert_eq!(name, "U"); assert_eq!(bounds.len(), 1);
        }
    }

    #[test]
    fn test_union_with_where_clause() {
        let input = "union Container[T, U]\n    where\n        T: Clone\n        U: Copy\n    first: T\n    second: U\n";
        let u = helpers::parse_union(input).unwrap();
        assert_eq!(u.generic_params.len(), 2);
        if let GenericParameter::Type { name, bounds, .. } = &u.generic_params[0] {
            assert_eq!(name, "T"); assert_eq!(bounds.len(), 1);
        }
        if let GenericParameter::Type { name, bounds, .. } = &u.generic_params[1] {
            assert_eq!(name, "U"); assert_eq!(bounds.len(), 1);
        }
    }

    #[test]
    fn test_union_with_requires_clause() {
        let input = "union MyUnion[T]\n    requires\n        Clone\n        Send\n    variant1: T\n    variant2: i32\n";
        let u = helpers::parse_union(input).unwrap();
        assert_eq!(u.requires.len(), 2);
        assert_eq!(path_type_name(&u.requires[0]), "Clone");
        assert_eq!(path_type_name(&u.requires[1]), "Send");
    }

    #[test]
    fn test_union_with_requires_and_where() {
        let input = "union Complex[T, U]\n    requires\n        Debug\n    where\n        T: Clone\n        U: Copy\n    first: T\n    second: U\n";
        let u = helpers::parse_union(input).unwrap();
        assert_eq!(u.requires.len(), 1);
        assert_eq!(u.generic_params.len(), 2);
    }

    #[test]
    fn test_union_with_complex_types() {
        let input = "union ComplexUnion\n    pointer: *i32\n    array: [u8]\n    reference: ?*mut bool\n";
        let u = helpers::parse_union(input).unwrap();
        assert_eq!(u.variants.len(), 3);

        let ty0 = u.variants[0].ty.clone();
        if let Type::Pointer { element_type, nullable, mutable } = ty0 {
            assert_eq!(*element_type, Type::I32); assert!(!nullable); assert!(!mutable);
        } else { panic!("Expected pointer"); }

        let ty1 = u.variants[1].ty.clone();
        if let Type::Array { element_type, size } = ty1 {
            assert!(size.is_none()); assert_eq!(*element_type, Type::U8);
        } else { panic!("Expected slice"); }

        // ?*mut bool  →  Nullable(Pointer { nullable: false, mutable: true, .. })
        let ty2 = u.variants[2].ty.clone();
        if let Type::Nullable(inner) = ty2 {
            if let Type::Pointer { element_type, nullable, mutable } = *inner {
                assert_eq!(*element_type, Type::Bool); assert!(!nullable); assert!(mutable);
            } else { panic!("Expected Pointer inside Nullable"); }
        } else { panic!("Expected nullable mutable pointer"); }
    }

    #[test]
    fn test_union_single_variant() {
        let input = "union Single\n    value: i32\n";
        let u = helpers::parse_union(input).unwrap();
        assert_eq!(u.variants.len(), 1);
    }

    #[test]
    fn test_union_with_const_param() {
        let input = "union Array[const N: usize]\n    data: i32\n    size: usize\n";
        let u = helpers::parse_union(input).unwrap();
        assert_eq!(u.generic_params.len(), 1);
        if let GenericParameter::Const { name, ty } = &u.generic_params[0] {
            assert_eq!(name, "N"); assert_eq!(ty, &Type::USize);
        } else { panic!("Expected const parameter"); }
    }
}

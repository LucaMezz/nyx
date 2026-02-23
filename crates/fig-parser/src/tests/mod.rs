#[cfg(test)]
mod function_tests;

#[cfg(test)]
mod struct_enum_union_tests;

#[cfg(test)]
mod type_alias_tests;

#[cfg(test)]
mod interface_tests;

#[cfg(test)]
mod namespace_tests;

use super::*;
use crate::ast::*;

#[test]
fn test_lexer_integration() {
    let input = "let x = 1\n";
    let lexer = Lexer::new(input);
    let tokens: Vec<_> = lexer.collect();
    assert!(!tokens.is_empty());
}

#[test]
fn test_simple_integer() {
    let input = "42";
    let lexer = Lexer::new(input);
    let result = parser::ExpressionParser::new().parse(lexer);
    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Expression::IntegerLiteral(_)));
}

#[test]
fn test_simple_boolean() {
    let input = "true";
    let lexer = Lexer::new(input);
    let result = parser::ExpressionParser::new().parse(lexer);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Expression::BooleanLiteral(true));
}

#[test]
fn test_binary_add() {
    let input = "1 + 2";
    let lexer = Lexer::new(input);
    let result = parser::ExpressionParser::new().parse(lexer);
    assert!(result.is_ok());
    if let Expression::BinaryOp(op) = result.unwrap() {
        assert_eq!(op.op, BinaryOperator::Add);
    } else {
        panic!("Expected binary operation");
    }
}

#[test]
fn test_precedence_mul_add() {
    // In LALRPOP, lower level number = tighter binding.
    // + is level 9, * is level 10, so + binds tighter than *.
    // Thus: 2 + 3 * 4 parses as (2 + 3) * 4 = Multiply(Add(2,3), 4)
    let input = "2 + 3 * 4";
    let lexer = Lexer::new(input);
    let result = parser::ExpressionParser::new().parse(lexer);
    assert!(result.is_ok());
    if let Expression::BinaryOp(mul_op) = result.unwrap() {
        assert_eq!(mul_op.op, BinaryOperator::Multiply);
        if let Expression::BinaryOp(add_op) = mul_op.lhs.as_ref() {
            assert_eq!(add_op.op, BinaryOperator::Add);
        } else {
            panic!("Expected addition on LHS of multiply");
        }
    } else {
        panic!("Expected multiplication at root");
    }
}

#[test]
fn test_unary_negation() {
    let input = "-5";
    let lexer = Lexer::new(input);
    let result = parser::ExpressionParser::new().parse(lexer);
    assert!(result.is_ok());
    if let Expression::UnaryOp(op) = result.unwrap() {
        assert_eq!(op.op, UnaryOperator::Negate);
    } else {
        panic!("Expected unary operation");
    }
}

#[test]
fn test_logical_or() {
    let input = "true || false";
    let lexer = Lexer::new(input);
    let result = parser::ExpressionParser::new().parse(lexer);
    assert!(result.is_ok());
}

#[test]
fn test_comparison() {
    let input = "5 < 10";
    let lexer = Lexer::new(input);
    let result = parser::ExpressionParser::new().parse(lexer);
    assert!(result.is_ok());
    if let Expression::BinaryOp(op) = result.unwrap() {
        assert_eq!(op.op, BinaryOperator::LessThan);
    } else {
        panic!("Expected comparison operation");
    }
}

#[test]
fn test_parenthesized() {
    // (2 + 3) * 4 — mul is at root, lhs is parenthesized
    let input = "(2 + 3) * 4";
    let lexer = Lexer::new(input);
    let result = parser::ExpressionParser::new().parse(lexer);
    assert!(result.is_ok());
    if let Expression::BinaryOp(mul_op) = result.unwrap() {
        assert_eq!(mul_op.op, BinaryOperator::Multiply);
        assert!(matches!(mul_op.lhs.as_ref(), Expression::Parenthesized(_)));
    } else {
        panic!("Expected multiplication at root");
    }
}

#[test]
fn test_array_literal_empty() {
    let input = "[]";
    let lexer = Lexer::new(input);
    let result = parser::ExpressionParser::new().parse(lexer);
    assert!(result.is_ok());
    if let Expression::ArrayLiteral(arr) = result.unwrap() {
        assert_eq!(arr.elements.len(), 0);
    } else {
        panic!("Expected array literal");
    }
}

#[test]
fn test_array_literal_with_elements() {
    let input = "[1, 2, 3]";
    let lexer = Lexer::new(input);
    let result = parser::ExpressionParser::new().parse(lexer);
    assert!(result.is_ok());
    if let Expression::ArrayLiteral(arr) = result.unwrap() {
        assert_eq!(arr.elements.len(), 3);
    } else {
        panic!("Expected array literal");
    }
}

#[test]
fn test_bitwise_operations() {
    let input = "5 & 3 | 1";
    let lexer = Lexer::new(input);
    let result = parser::ExpressionParser::new().parse(lexer);
    assert!(result.is_ok());
}

#[test]
fn test_address_of_expression() {
    let input = "&x";
    let lexer = Lexer::new(input);
    let result = parser::ExpressionParser::new().parse(lexer);
    assert!(result.is_ok());
    if let Expression::UnaryOp(op) = result.unwrap() {
        assert_eq!(op.op, UnaryOperator::AddressOf);
        assert!(matches!(&*op.operand, Expression::Path(_)));
    } else {
        panic!("Expected AddressOf unary operation");
    }
}

#[test]
fn test_dereference_expression() {
    let input = "*y";
    let lexer = Lexer::new(input);
    let result = parser::ExpressionParser::new().parse(lexer);
    assert!(result.is_ok());
    if let Expression::UnaryOp(op) = result.unwrap() {
        assert_eq!(op.op, UnaryOperator::Dereference);
        assert!(matches!(&*op.operand, Expression::Path(_)));
    } else {
        panic!("Expected Dereference unary operation");
    }
}

#[test]
fn test_nested_address_of_dereference() {
    let input = "&*z";
    let lexer = Lexer::new(input);
    let result = parser::ExpressionParser::new().parse(lexer);
    assert!(result.is_ok());
    if let Expression::UnaryOp(outer_op) = result.unwrap() {
        assert_eq!(outer_op.op, UnaryOperator::AddressOf);
        if let Expression::UnaryOp(inner_op) = &*outer_op.operand {
            assert_eq!(inner_op.op, UnaryOperator::Dereference);
            assert!(matches!(&*inner_op.operand, Expression::Path(_)));
        } else {
            panic!("Expected nested Dereference operation");
        }
    } else {
        panic!("Expected outer AddressOf operation");
    }
}

#[test]
fn test_dereference_address_of_expression() {
    let input = "*&z";
    let lexer = Lexer::new(input);
    let result = parser::ExpressionParser::new().parse(lexer);
    assert!(result.is_ok());
    if let Expression::UnaryOp(outer_op) = result.unwrap() {
        assert_eq!(outer_op.op, UnaryOperator::Dereference);
        if let Expression::UnaryOp(inner_op) = &*outer_op.operand {
            assert_eq!(inner_op.op, UnaryOperator::AddressOf);
            assert!(matches!(&*inner_op.operand, Expression::Path(_)));
        } else {
            panic!("Expected nested AddressOf operation");
        }
    } else {
        panic!("Expected outer Dereference operation");
    }
}

#[test]
fn test_unary_with_binary_precedence() {
    let input = "&x + 5";
    let lexer = Lexer::new(input);
    let result = parser::ExpressionParser::new().parse(lexer);
    assert!(result.is_ok());
    if let Expression::BinaryOp(binary_op) = result.unwrap() {
        assert_eq!(binary_op.op, BinaryOperator::Add);
        if let Expression::UnaryOp(unary_op) = binary_op.lhs.as_ref() {
            assert_eq!(unary_op.op, UnaryOperator::AddressOf);
            assert!(matches!(&*unary_op.operand, Expression::Path(_)));
        } else {
            panic!("Expected AddressOf on LHS of binary op");
        }
        assert!(matches!(binary_op.rhs.as_ref(), Expression::IntegerLiteral(_)));
    } else {
        panic!("Expected binary operation");
    }
}

#[test]
fn test_unary_with_parenthesized() {
    let input = "&(x + 5)";
    let lexer = Lexer::new(input);
    let result = parser::ExpressionParser::new().parse(lexer);
    assert!(result.is_ok());
    if let Expression::UnaryOp(unary_op) = result.unwrap() {
        assert_eq!(unary_op.op, UnaryOperator::AddressOf);
        if let Expression::Parenthesized(inner_expr) = &*unary_op.operand {
            assert!(matches!(inner_expr.as_ref(), Expression::BinaryOp(_)));
        } else {
            panic!("Expected parenthesized expression after AddressOf");
        }
    } else {
        panic!("Expected Unary operation");
    }
}

// ── Type parsing tests ───────────────────────────────────────────────────────

#[test]
fn test_parse_primitive_type_u8() {
    let result = parser::TypeParser::new().parse(Lexer::new("u8"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Type::U8);
}

#[test]
fn test_parse_primitive_type_i32() {
    let result = parser::TypeParser::new().parse(Lexer::new("i32"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Type::I32);
}

#[test]
fn test_parse_primitive_type_f64() {
    let result = parser::TypeParser::new().parse(Lexer::new("f64"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Type::F64);
}

#[test]
fn test_parse_primitive_type_bool() {
    let result = parser::TypeParser::new().parse(Lexer::new("bool"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Type::Bool);
}

#[test]
fn test_parse_raw_pointer() {
    let result = parser::TypeParser::new().parse(Lexer::new("*u8"));
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        Type::Pointer { nullable: false, mutable: false, element_type: Box::new(Type::U8) }
    );
}

#[test]
fn test_parse_typed_pointer_u32() {
    let result = parser::TypeParser::new().parse(Lexer::new("*u32"));
    assert!(result.is_ok());
    if let Type::Pointer { element_type, nullable, mutable } = result.unwrap() {
        assert!(!nullable);
        assert!(!mutable);
        assert_eq!(*element_type, Type::U32);
    } else {
        panic!("Expected pointer");
    }
}

#[test]
fn test_parse_mutable_pointer() {
    let result = parser::TypeParser::new().parse(Lexer::new("*mut bool"));
    assert!(result.is_ok());
    if let Type::Pointer { element_type, nullable, mutable } = result.unwrap() {
        assert!(!nullable);
        assert!(mutable);
        assert_eq!(*element_type, Type::Bool);
    } else {
        panic!("Expected mutable pointer");
    }
}

#[test]
fn test_parse_nullable_pointer() {
    // With the new grammar `?*i32` → Nullable(Pointer { nullable: false, .. })
    let result = parser::TypeParser::new().parse(Lexer::new("?*i32"));
    assert!(result.is_ok());
    if let Type::Nullable(inner) = result.unwrap() {
        if let Type::Pointer { element_type, nullable, mutable } = *inner {
            assert!(!nullable);
            assert!(!mutable);
            assert_eq!(*element_type, Type::I32);
        } else {
            panic!("Expected Pointer inside Nullable");
        }
    } else {
        panic!("Expected nullable pointer");
    }
}

#[test]
fn test_parse_nested_pointer_i64() {
    let result = parser::TypeParser::new().parse(Lexer::new("**i64"));
    assert!(result.is_ok());
    if let Type::Pointer { element_type: outer, .. } = result.unwrap() {
        if let Type::Pointer { element_type: inner, .. } = *outer {
            assert_eq!(*inner, Type::I64);
        } else {
            panic!("Expected nested pointer");
        }
    } else {
        panic!("Expected outer pointer");
    }
}

#[test]
fn test_parse_slice_type() {
    let result = parser::TypeParser::new().parse(Lexer::new("[u8]"));
    assert!(result.is_ok());
    if let Type::Array { element_type, size } = result.unwrap() {
        assert!(size.is_none());
        assert_eq!(*element_type, Type::U8);
    } else {
        panic!("Expected slice type");
    }
}

#[test]
fn test_parse_fixed_array_type() {
    // [u8; 4] is a fixed-size array
    let result = parser::TypeParser::new().parse(Lexer::new("[u8; 4]"));
    assert!(result.is_ok());
    if let Type::Array { element_type, size } = result.unwrap() {
        assert!(size.is_some());
        assert_eq!(*element_type, Type::U8);
    } else {
        panic!("Expected fixed array type");
    }
}

#[test]
fn test_parse_error_union_type() {
    // T ! E  is the "ok-or-error" type union; use TopLevelTypeParser
    let result = parser::TopLevelTypeParser::new().parse(Lexer::new("i32 ! IoError"));
    assert!(result.is_ok());
    if let Type::ErrorUnion { ok_type, err_type } = result.unwrap() {
        assert_eq!(*ok_type, Type::I32);
        if let Type::Path { path, .. } = *err_type { assert_eq!(path.segments, vec!["IoError".to_string()]); } else { panic!("Expected Path type for err_type"); }
    } else {
        panic!("Expected ErrorUnion type");
    }
}

#[test]
fn test_parse_pointer_error_union() {
    // *T ! E  should be  (*T) ! E; use TopLevelTypeParser
    let result = parser::TopLevelTypeParser::new().parse(Lexer::new("*i32 ! IoError"));
    assert!(result.is_ok());
    if let Type::ErrorUnion { ok_type, err_type } = result.unwrap() {
        assert!(matches!(*ok_type, Type::Pointer { .. }));
        if let Type::Path { path, .. } = *err_type { assert_eq!(path.segments[0], "IoError"); } else { panic!("Expected Path type for err_type"); }
    } else {
        panic!("Expected ErrorUnion wrapping pointer");
    }
}

#[test]
fn test_parse_nullable_pointer_error_union() {
    // ?*T ! E  should be  (?*T) ! E = Nullable(Pointer{..}) ! E; use TopLevelTypeParser
    let result = parser::TopLevelTypeParser::new().parse(Lexer::new("?*u8 ! IoError"));
    assert!(result.is_ok());
    if let Type::ErrorUnion { ok_type, .. } = result.unwrap() {
        if let Type::Nullable(inner) = *ok_type {
            assert!(matches!(*inner, Type::Pointer { .. }));
        } else {
            panic!("Expected Nullable(Pointer) inside error union");
        }
    } else {
        panic!("Expected ErrorUnion");
    }
}

#[test]
fn test_parse_as_expression_with_error_union() {
    // x as i32 ! E  should be  x as (i32 ! E)
    let input = "x as i32 ! IoError";
    let lexer = Lexer::new(input);
    let result = parser::ExpressionParser::new().parse(lexer);
    assert!(result.is_ok(), "Failed: {:?}", result);
    if let Expression::Cast(cast) = result.unwrap() {
        assert!(matches!(*cast.target_type, Type::ErrorUnion { .. }));
    } else {
        panic!("Expected Cast expression");
    }
}

#[test]
fn test_parse_generics_list() {
    let input = "[T: Mappable, U: Copy, const N: usize]";
    let lexer = Lexer::new(input);
    let result = parser::GenericParameterListParser::new().parse(lexer);
    assert!(result.is_ok());
    let params = result.unwrap();
    assert_eq!(params.len(), 3);

    if let GenericParameter::Type { name, bounds, .. } = &params[0] {
        assert_eq!(name, "T");
        assert_eq!(bounds.len(), 1);
    } else {
        panic!("Expected type parameter T");
    }

    if let GenericParameter::Type { name, bounds, .. } = &params[1] {
        assert_eq!(name, "U");
        assert_eq!(bounds.len(), 1);
    } else {
        panic!("Expected type parameter U");
    }

    if let GenericParameter::Const { name, ty } = &params[2] {
        assert_eq!(name, "N");
        assert_eq!(ty, &Type::USize);
    } else {
        panic!("Expected const parameter N");
    }
}

#[test]
fn test_indentation_tokens() {
    let input = "let x = 1\n    let y = 2\nlet z = 3";
    let lexer = Lexer::new(input);
    let tokens: Vec<_> = lexer.collect();

    let has_newline = tokens.iter().any(|t| matches!(t, Ok((_, fig_lexer::Token::Newline, _))));
    let has_indent  = tokens.iter().any(|t| matches!(t, Ok((_, fig_lexer::Token::Indent,   _))));
    let has_dedent  = tokens.iter().any(|t| matches!(t, Ok((_, fig_lexer::Token::Dedent,   _))));

    assert!(has_newline, "Expected NEWLINE tokens");
    assert!(has_indent,  "Expected INDENT token");
    assert!(has_dedent,  "Expected DEDENT token");
}

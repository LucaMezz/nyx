//! Test helpers that parse language constructs using the top-level `SourceFileParser`
//! instead of requiring individual per-rule public entry points.
//!
//! This keeps the LALRPOP grammar lean (only `pub SourceFile`) while still
//! allowing unit tests to easily exercise every construct.

use crate::{ast::*, parser::SourceFileParser, Lexer};

type ParseResult<T> = Result<T, String>;

// ── Private helpers ──────────────────────────────────────────────────────────

fn parse_source(input: &str) -> ParseResult<SourceFile> {
    let mut errors = vec![];
    SourceFileParser::new()
        .parse(&mut errors, Lexer::new(input))
        .map_err(|e| format!("{e:?}"))
}

fn first_item(input: &str) -> ParseResult<NamespaceItem> {
    parse_source(input)?
        .items
        .into_iter()
        .next()
        .ok_or_else(|| "empty source file".to_string())
}

// ── Top-level item helpers ───────────────────────────────────────────────────

/// Parse a `func` definition (with body) from a snippet.
pub fn parse_function(input: &str) -> ParseResult<Function> {
    match first_item(input)? {
        NamespaceItem::Function(f) => Ok(f),
        other => Err(format!("expected Function, got: {other:?}")),
    }
}

/// Parse a `func` forward declaration (without body) from a snippet.
pub fn parse_function_decl(input: &str) -> ParseResult<FunctionDeclaration> {
    match first_item(input)? {
        NamespaceItem::FunctionDeclaration(f) => Ok(f),
        other => Err(format!("expected FunctionDeclaration, got: {other:?}")),
    }
}

/// Parse a `struct` declaration from a snippet.
pub fn parse_struct(input: &str) -> ParseResult<Struct> {
    match first_item(input)? {
        NamespaceItem::Struct(s) => Ok(s),
        other => Err(format!("expected Struct, got: {other:?}")),
    }
}

/// Parse an `enum` declaration from a snippet.
pub fn parse_enum(input: &str) -> ParseResult<Enum> {
    match first_item(input)? {
        NamespaceItem::Enum(e) => Ok(e),
        other => Err(format!("expected Enum, got: {other:?}")),
    }
}

/// Parse a `union` declaration from a snippet.
pub fn parse_union(input: &str) -> ParseResult<Union> {
    match first_item(input)? {
        NamespaceItem::Union(u) => Ok(u),
        other => Err(format!("expected Union, got: {other:?}")),
    }
}

/// Parse an `interface` declaration from a snippet.
pub fn parse_interface(input: &str) -> ParseResult<Interface> {
    match first_item(input)? {
        NamespaceItem::Interface(i) => Ok(i),
        other => Err(format!("expected Interface, got: {other:?}")),
    }
}

/// Parse a `namespace` declaration from a snippet.
pub fn parse_namespace(input: &str) -> ParseResult<Namespace> {
    match first_item(input)? {
        NamespaceItem::Namespace(n) => Ok(n),
        other => Err(format!("expected Namespace, got: {other:?}")),
    }
}

/// Parse a `type` alias from a snippet.
///
/// - Inline form (`type Foo = Bar\n`) requires a trailing newline in the input.
/// - Where-clause form ends correctly with a `DEDENT` generated from EOF.
pub fn parse_type_alias(input: &str) -> ParseResult<TypeAlias> {
    match first_item(input)? {
        NamespaceItem::TypeAlias(ta) => Ok(ta),
        other => Err(format!("expected TypeAlias, got: {other:?}")),
    }
}

// ── Nested-context helpers ───────────────────────────────────────────────────

/// Parse a bare type expression (including error-union `T ! E`) by embedding
/// it in a type-alias declaration, which uses `TopLevelType` on the RHS.
///
/// ```text
/// type _TestType_ = TYPE\n
/// ```
pub fn parse_type(input: &str) -> ParseResult<Type> {
    // Trim any trailing newline from the snippet to avoid double-newline
    let input = input.trim_end_matches('\n');
    let src = format!("type _TestType_ = {input}\n");
    parse_type_alias(&src).map(|ta| ta.aliased_type)
}

/// Parse a standalone expression by embedding it in a `let` binding inside a
/// temporary function body.
///
/// ```text
/// func _testfn_()
///     let _result_ = EXPR
/// ```
pub fn parse_expression(input: &str) -> ParseResult<Expression> {
    let input = input.trim_end_matches('\n');
    let src = format!("func _testfn_()\n    let _result_ = {input}\n");
    let sf = parse_source(&src)?;
    match sf.items.into_iter().next() {
        Some(NamespaceItem::Function(f)) => {
            match f.body.statements.into_iter().next() {
                Some(Statement::Let(l)) => Ok(*l.value),
                Some(s) => Err(format!("expected Let statement, got: {s:?}")),
                None => Err("empty function body".to_string()),
            }
        }
        other => Err(format!("expected Function, got: {other:?}")),
    }
}

/// Parse a generic-parameter list like `[T: Clone, U: Copy, const N: usize]`
/// by embedding it in a function declaration.
///
/// The `input` must include the surrounding `[` and `]`.
///
/// ```text
/// func [T: Clone, …] _testfn_()
///     pass
/// ```
pub fn parse_generic_params(input: &str) -> ParseResult<Vec<GenericParameter>> {
    let input = input.trim();
    let src = format!("func {input} _testfn_()\n    pass\n");
    parse_function(&src).map(|f| f.signature.outer_generic_params)
}

//! Beautiful parse-error diagnostics using `codespan-reporting`.
//!
//! # Design
//!
//! This module is intentionally self-contained so it can be used at every
//! layer of the compiler pipeline:
//!
//! * **Unit / integration tests** — call [`format_parse_error`] to get a
//!   plain-text rendering (no ANSI codes) that can be embedded in the test
//!   failure message.
//! * **CLI / REPL** — call [`emit_parse_error`] with
//!   `StandardStream::stderr(ColorChoice::Auto)` for a fully coloured
//!   terminal experience.
//! * **LSP / IDE** — call [`build_diagnostic`] to obtain the raw
//!   [`Diagnostic`] value and attach it to any `codespan-reporting`-compatible
//!   file database you already own.
//!
//! None of these entry-points panic; they write errors to the supplied
//! writer or return a `String`.

use codespan_reporting::diagnostic::{Diagnostic, Label, Severity};
use codespan_reporting::files::SimpleFile;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{Buffer, WriteColor};

use fig_lexer::Token;

use crate::LexicalError;

// ============================================================================
// Public type alias
// ============================================================================

/// The concrete parse-error type produced by the Fig LALRPOP parser.
pub type FigParseError = lalrpop_util::ParseError<usize, Token, LexicalError>;

// ============================================================================
// Primary entry points
// ============================================================================

/// Render a parse error as a plain-text string (no ANSI colour codes).
///
/// Suitable for embedding in test failure messages, log files, and any context
/// where colour escape sequences would be distracting.
///
/// `filename` is shown in the header line; it does not need to exist on disk.
pub fn format_parse_error(filename: &str, source: &str, err: &FigParseError) -> String {
    let mut buf = Buffer::no_color();
    emit_parse_error(&mut buf, filename, source, err);
    String::from_utf8_lossy(buf.as_slice()).into_owned()
}

/// Emit a parse error to any [`WriteColor`] sink.
///
/// Use `codespan_reporting::term::termcolor::StandardStream` for coloured
/// terminal output, or `Buffer::no_color()` for plain text.
///
/// # Example
/// ```ignore
/// use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
///
/// let stderr = StandardStream::stderr(ColorChoice::Auto);
/// emit_parse_error(&mut stderr.lock(), "main.fig", source, &err);
/// ```
pub fn emit_parse_error<W: WriteColor>(
    writer: &mut W,
    filename: &str,
    source: &str,
    err: &FigParseError,
) {
    let file = SimpleFile::new(filename, source);
    let diag = build_diagnostic(err);
    let config = term::Config::default();
    // If emission fails (e.g. broken pipe) there is nothing useful we can do.
    let _ = term::emit(writer, &config, &file, &diag);
}

/// Build a raw [`Diagnostic`] from a parse error.
///
/// The file-id type parameter is `()`, matching a
/// [`codespan_reporting::files::SimpleFile`].  If you maintain your own
/// multi-file database, wrap this in a conversion that substitutes the
/// appropriate file id.
pub fn build_diagnostic(err: &FigParseError) -> Diagnostic<()> {
    use lalrpop_util::ParseError::*;

    match err {
        // ── Invalid / unrecognised character ─────────────────────────────────
        InvalidToken { location } => Diagnostic::error()
            .with_message("invalid token")
            .with_labels(vec![Label::primary((), *location..*location + 1)
                .with_message("this character is not valid here")]),

        // ── Unexpected end of file ────────────────────────────────────────────
        UnrecognizedEof { location, expected: _ } => Diagnostic::error()
            .with_message("unexpected end of file")
            .with_labels(vec![
                Label::primary((), *location..*location).with_message("file ends here"),
            ]),

        // ── A token the grammar did not expect at this position ───────────────
        UnrecognizedToken { token: (start, tok, end), expected: _ } => {
            let tok_label = token_name(tok);
            Diagnostic::error()
                .with_message(format!("unexpected token `{tok_label}`"))
                .with_labels(vec![Label::primary((), *start..*end)
                    .with_message(format!("`{tok_label}` is not valid at this position"))])
        }

        // ── Extra / trailing token ────────────────────────────────────────────
        ExtraToken { token: (start, tok, end) } => {
            let tok_label = token_name(tok);
            Diagnostic::error()
                .with_message(format!("unexpected extra token `{tok_label}`"))
                .with_labels(vec![Label::primary((), *start..*end)
                    .with_message("this token should not be here")])
        }

        // ── Lexer error bubbled up ────────────────────────────────────────────
        User { error } => Diagnostic::error()
            .with_message(format!("lexer error: {error}"))
            .with_notes(vec!["the source contains a character sequence the lexer cannot tokenise".to_owned()]),
    }
}

// ============================================================================
// Token display name
// ============================================================================

/// Return a concise, human-readable name for a [`Token`] value.
///
/// Used when describing the *found* (unexpected) token in an error message.
pub fn token_name(tok: &Token) -> &'static str {
    use Token::*;
    match tok {
        // Structural
        Indent          => "indentation",
        Dedent          => "dedentation",
        Newline         => "newline",
        // Keywords
        Func            => "keyword `func`",
        Let             => "keyword `let`",
        Mut             => "keyword `mut`",
        Const           => "keyword `const`",
        Type            => "keyword `type`",
        Struct          => "keyword `struct`",
        Enum            => "keyword `enum`",
        Union           => "keyword `union`",
        Interface       => "keyword `interface`",
        Impl            => "keyword `impl`",
        True            => "`true`",
        False           => "`false`",
        Null            => "`null`",
        SelfLower       => "`self`",
        If              => "keyword `if`",
        Else            => "keyword `else`",
        Elif            => "keyword `elif`",
        For             => "keyword `for`",
        While           => "keyword `while`",
        Break           => "keyword `break`",
        Continue        => "keyword `continue`",
        Match           => "keyword `match`",
        Return          => "keyword `return`",
        In              => "keyword `in`",
        Where           => "keyword `where`",
        Requires        => "keyword `requires`",
        Extends         => "keyword `extends`",
        Namespace       => "keyword `namespace`",
        Pass            => "keyword `pass`",
        Block           => "keyword `block`",
        Using           => "keyword `using`",
        Extern          => "keyword `extern`",
        Packed          => "keyword `packed`",
        Public          => "keyword `public`",
        Export          => "keyword `export`",
        Private         => "keyword `private`",
        As              => "keyword `as`",
        Sizeof          => "keyword `sizeof`",
        Alignof         => "keyword `alignof`",
        Offsetof        => "keyword `offsetof`",
        // Primitive type keywords
        U8              => "type `u8`",
        U16             => "type `u16`",
        U32             => "type `u32`",
        U64             => "type `u64`",
        USize           => "type `usize`",
        ISize           => "type `isize`",
        I8              => "type `i8`",
        I16             => "type `i16`",
        I32             => "type `i32`",
        I64             => "type `i64`",
        F32             => "type `f32`",
        F64             => "type `f64`",
        Bool            => "type `bool`",
        // Operators
        Plus            => "`+`",
        Minus           => "`-`",
        Star            => "`*`",
        Slash           => "`/`",
        Percent         => "`%`",
        EqEq            => "`==`",
        Ne              => "`!=`",
        Lt              => "`<`",
        Gt              => "`>`",
        Le              => "`<=`",
        Ge              => "`>=`",
        AndAnd          => "`&&`",
        OrOr            => "`||`",
        Bang            => "`!`",
        And             => "`&`",
        Or              => "`|`",
        Caret           => "`^`",
        Tilde           => "`~`",
        Shl             => "`<<`",
        Shr             => "`>>`",
        Eq              => "`=`",
        PlusEq          => "`+=`",
        MinusEq         => "`-=`",
        StarEq          => "`*=`",
        SlashEq         => "`/=`",
        PercentEq       => "`%=`",
        AndEq           => "`&=`",
        OrEq            => "`|=`",
        CaretEq         => "`^=`",
        ShlEq           => "`<<=`",
        ShrEq           => "`>>=`",
        Arrow           => "`->`",
        FatArrow        => "`=>`",
        Question        => "`?`",
        // Delimiters
        LParen          => "`(`",
        RParen          => "`)`",
        LBrace          => "`{`",
        RBrace          => "`}`",
        LBracket        => "`[`",
        RBracket        => "`]`",
        ColonColon      => "`::`",
        Colon           => "`:`",
        Semicolon       => "`;`",
        Comma           => "`,`",
        Dot             => "`.`",
        Underscore      => "`_`",
        // Punctuation
        Hash            => "`#`",
        // Value-carrying
        Ident(_)                    => "identifier",
        FloatLiteral(_)             => "float literal",
        IntegerLiteral(_)           => "integer literal",
        CharLiteral(_)              => "character literal",
        StringLiteral(_)            => "string literal",
        InterpolatedStringLiteral(_) => "interpolated string literal",
        Comment                     => "comment",
    }
}

// ============================================================================
// Helpers for "expected" note formatting
// ============================================================================

/// Convert a LALRPOP grammar terminal string into a friendly display label.
#[allow(dead_code)]
fn terminal_label(s: &str) -> &'static str {
    match s {
        // Structural
        "NEWLINE"       => "newline",
        "INDENT"        => "indentation block",
        "DEDENT"        => "end of block",
        // Literals & generic terminals
        "ident"         => "identifier",
        "int"           => "integer literal",
        "float"         => "float literal",
        "char"          => "character literal",
        "string"        => "string literal",
        "interpstring"  => "interpolated string",
        // Boolean / null
        "true"          => "`true`",
        "false"         => "`false`",
        "null"          => "`null`",
        "self"          => "`self`",
        // Keywords
        "func"          => "keyword `func`",
        "let"           => "keyword `let`",
        "mut"           => "keyword `mut`",
        "const"         => "keyword `const`",
        "type"          => "keyword `type`",
        "struct"        => "keyword `struct`",
        "enum"          => "keyword `enum`",
        "union"         => "keyword `union`",
        "interface"     => "keyword `interface`",
        "namespace"     => "keyword `namespace`",
        "if"            => "keyword `if`",
        "else"          => "keyword `else`",
        "elif"          => "keyword `elif`",
        "for"           => "keyword `for`",
        "while"         => "keyword `while`",
        "break"         => "keyword `break`",
        "continue"      => "keyword `continue`",
        "match"         => "keyword `match`",
        "return"        => "keyword `return`",
        "pass"          => "keyword `pass`",
        "block"         => "keyword `block`",
        "in"            => "keyword `in`",
        "as"            => "keyword `as`",
        "using"         => "keyword `using`",
        "extern"        => "keyword `extern`",
        "packed"        => "keyword `packed`",
        "where"         => "keyword `where`",
        "requires"      => "keyword `requires`",
        "extends"       => "keyword `extends`",
        "sizeof"        => "`sizeof`",
        "alignof"       => "`alignof`",
        "offsetof"      => "`offsetof`",
        // Visibility
        "public"        => "keyword `public`",
        "export"        => "keyword `export`",
        "private"       => "keyword `private`",
        // Primitive types
        "u8"            => "type `u8`",
        "u16"           => "type `u16`",
        "u32"           => "type `u32`",
        "u64"           => "type `u64`",
        "usize"         => "type `usize`",
        "isize"         => "type `isize`",
        "i8"            => "type `i8`",
        "i16"           => "type `i16`",
        "i32"           => "type `i32`",
        "i64"           => "type `i64`",
        "f32"           => "type `f32`",
        "f64"           => "type `f64`",
        "bool"          => "type `bool`",
        // Operators
        "+"             => "`+`",
        "-"             => "`-`",
        "*"             => "`*`",
        "/"             => "`/`",
        "%"             => "`%`",
        "=="            => "`==`",
        "!="            => "`!=`",
        "<"             => "`<`",
        ">"             => "`>`",
        "<="            => "`<=`",
        ">="            => "`>=`",
        "&&"            => "`&&`",
        "||"            => "`||`",
        "!"             => "`!`",
        "&"             => "`&`",
        "|"             => "`|`",
        "^"             => "`^`",
        "~"             => "`~`",
        "<<"            => "`<<`",
        ">>"            => "`>>`",
        "="             => "`=`",
        "+="            => "`+=`",
        "-="            => "`-=`",
        "*="            => "`*=`",
        "/="            => "`/=`",
        "%="            => "`%=`",
        "&="            => "`&=`",
        "|="            => "`|=`",
        "^="            => "`^=`",
        "<<="           => "`<<=`",
        ">>="           => "`>>=`",
        "->"            => "`->`",
        "=>"            => "`=>`",
        "?"             => "`?`",
        // Delimiters
        "("             => "`(`",
        ")"             => "`)`",
        "{"             => "`{`",
        "}"             => "`}`",
        "["             => "`[`",
        "]"             => "`]`",
        "::"            => "`::`",
        ":"             => "`:`",
        ";"             => "`;`",
        ","             => "`,`",
        "."             => "`.`",
        "_"             => "`_`",
        "#"             => "`#`",
        // Fall-through: echo the original string
        other           => {
            // Safety: LALRPOP terminal strings are 'static because they come
            // from the grammar file which is compiled into the binary.
            // We can't return a &'static str for the dynamic case, so we leak
            // a tiny allocation for any unknown terminal.
            Box::leak(other.to_owned().into_boxed_str())
        }
    }
}

// Tokens that collectively mean "an expression is expected here".
#[allow(dead_code)]
const EXPR_STARTERS: &[&str] = &[
    "ident", "int", "float", "char", "string", "interpstring",
    "true", "false", "null", "self", "(", "[", "sizeof", "alignof", "offsetof",
    // Unary prefix operators that can begin an expression
    "!", "~", "-", "&", "*", "+",
];

// Tokens that collectively mean "a type is expected here".
#[allow(dead_code)]
const TYPE_STARTERS: &[&str] = &[
    "ident", "u8", "u16", "u32", "u64", "usize", "isize",
    "i8", "i16", "i32", "i64", "f32", "f64", "bool", "*", "?", "[", "func",
];

/// Produce a `Vec<String>` of note lines describing what was expected.
///
/// Currently disabled — the LALRPOP expected-set can be very large and crowds
/// the error message. Re-enable by calling this from [`build_diagnostic`].
#[allow(dead_code)]
fn expected_note(expected: &[String]) -> Vec<String> {
    if expected.is_empty() {
        return Vec::new();
    }

    // Strip surrounding quotes that LALRPOP adds to every terminal label.
    let stripped: Vec<&str> = expected
        .iter()
        .map(|s| s.trim_matches('"'))
        .collect();

    let set: std::collections::HashSet<&str> = stripped.iter().copied().collect();

    // Check how many expression / type starters are present.
    let expr_matches = EXPR_STARTERS.iter().filter(|&&s| set.contains(s)).count();
    let type_matches = TYPE_STARTERS.iter().filter(|&&s| set.contains(s)).count();

    // Build the summary phrase(s).
    let mut summary_parts: Vec<&str> = Vec::new();

    if expr_matches >= 5 {
        summary_parts.push("an expression");
    }
    if type_matches >= 3 {
        summary_parts.push("a type");
    }

    // Collect the tokens that are NOT covered by a summary phrase.
    let covered: std::collections::HashSet<&str> = {
        let mut c = std::collections::HashSet::new();
        if expr_matches >= 5 {
            c.extend(EXPR_STARTERS.iter().copied());
        }
        if type_matches >= 3 {
            c.extend(TYPE_STARTERS.iter().copied());
        }
        c
    };

    let mut remaining: Vec<&str> = set
        .iter()
        .copied()
        .filter(|s| !covered.contains(s))
        // Skip pure structural/indent tokens unless they're the only option.
        .filter(|s| !matches!(*s, "NEWLINE" | "INDENT" | "DEDENT") || expected.len() == 1)
        .collect();

    remaining.sort_unstable();

    // Friendly labels for the remaining tokens.
    let mut labels: Vec<&str> = remaining.iter().map(|s| terminal_label(s)).collect();
    labels.sort_unstable();
    labels.dedup();

    // Merge summary phrases and remaining token labels.
    let mut all: Vec<&str> = summary_parts;
    all.extend(labels);
    all.dedup();

    if all.is_empty() {
        return Vec::new();
    }

    let note = if all.len() == 1 {
        format!("expected {}", all[0])
    } else {
        let (last, rest) = all.split_last().unwrap();
        format!("expected {} or {}", rest.join(", "), last)
    };

    vec![note]
}

// ============================================================================
// Test / CLI error wrapper
// ============================================================================

/// An error type that renders a pre-formatted diagnostic string *as-is*,
/// preserving newlines and box-drawing characters.
///
/// The standard library's `String: Error` implementation displays correctly
/// via `Display`, but test frameworks (e.g. `datatest_stable`) call `Debug`
/// on the boxed error, which escapes every `\n` to a literal backslash-n and
/// wraps the whole thing in quotes — turning a beautiful multi-line diagnostic
/// into a single unreadable line.
///
/// This wrapper's `Debug` impl delegates to `Display` (i.e. writes the raw
/// string), so the diagnostic is rendered correctly in test failure output.
#[derive(Clone)]
pub struct DiagnosticError(pub String);

impl std::fmt::Debug for DiagnosticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Write the raw string — no quoting, no escape sequences.
        f.write_str(&self.0)
    }
}

impl std::fmt::Display for DiagnosticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for DiagnosticError {}

// ============================================================================
// Severity helpers (for future use in warnings / lints)
// ============================================================================

/// Build a [`Diagnostic`] of a given severity with a free-form message and a
/// single primary label.  Useful for the semantic analysis passes that will
/// be built on top of this infrastructure.
pub fn simple_diagnostic(
    severity: Severity,
    message: impl Into<String>,
    span: std::ops::Range<usize>,
    label: impl Into<String>,
) -> Diagnostic<()> {
    Diagnostic::new(severity)
        .with_message(message)
        .with_labels(vec![Label::primary((), span).with_message(label)])
}

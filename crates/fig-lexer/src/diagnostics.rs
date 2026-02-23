//! Beautiful lexer-error diagnostics using `codespan-reporting`.
//!
//! # Design
//!
//! Mirrors the layout of `fig-parser::diagnostics` so every layer of the
//! compiler pipeline speaks the same diagnostic language:
//!
//! * **Unit / integration tests** — call [`format_lex_error`] for a
//!   plain-text, no-ANSI rendering suitable for test failure messages.
//! * **CLI / REPL** — call [`emit_lex_error`] with a
//!   `StandardStream::stderr(ColorChoice::Auto)` writer for full colour.
//! * **LSP / IDE** — call [`build_lex_diagnostic`] to get a raw
//!   [`Diagnostic`] value that you can attach to your own file database.
//!
//! None of these entry-points panic.

use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFile;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{Buffer, WriteColor};

use crate::LexicalError;

// ============================================================================
// Primary entry points
// ============================================================================

/// Render a single lexical error as a plain-text string (no ANSI codes).
///
/// `filename` is shown in the header; it does not need to exist on disk.
pub fn format_lex_error(filename: &str, source: &str, err: &LexicalError) -> String {
    let mut buf = Buffer::no_color();
    emit_lex_error(&mut buf, filename, source, err);
    String::from_utf8_lossy(buf.as_slice()).into_owned()
}

/// Render all lexical errors accumulated during a lex pass as a single
/// plain-text string.  Each error is separated by a blank line.
pub fn format_lex_errors(filename: &str, source: &str, errors: &[LexicalError]) -> String {
    errors
        .iter()
        .map(|e| format_lex_error(filename, source, e))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Emit a single lexical error to any [`WriteColor`] sink.
///
/// # Example
/// ```ignore
/// use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
///
/// let stderr = StandardStream::stderr(ColorChoice::Auto);
/// emit_lex_error(&mut stderr.lock(), "main.fig", source, &err);
/// ```
pub fn emit_lex_error<W: WriteColor>(
    writer: &mut W,
    filename: &str,
    source: &str,
    err: &LexicalError,
) {
    let file = SimpleFile::new(filename, source);
    let diag = build_lex_diagnostic(err);
    let config = term::Config::default();
    let _ = term::emit(writer, &config, &file, &diag);
}

/// Build a raw [`Diagnostic`] from a [`LexicalError`].
///
/// Uses `()` as the file-id, matching [`codespan_reporting::files::SimpleFile`].
/// For multi-file setups substitute the appropriate file id from your database.
pub fn build_lex_diagnostic(err: &LexicalError) -> Diagnostic<()> {
    match err {
        LexicalError::InvalidInteger { span, reason } => Diagnostic::error()
            .with_message("invalid integer literal")
            .with_labels(vec![
                Label::primary((), span.clone()).with_message(reason.as_str()),
            ]),

        LexicalError::InvalidFloat { span, reason } => Diagnostic::error()
            .with_message("invalid float literal")
            .with_labels(vec![
                Label::primary((), span.clone()).with_message(reason.as_str()),
            ]),

        LexicalError::InvalidCharLiteral { span, reason } => Diagnostic::error()
            .with_message("invalid character literal")
            .with_labels(vec![
                Label::primary((), span.clone()).with_message(reason.as_str()),
            ]),

        LexicalError::InvalidStringLiteral { span, reason } => Diagnostic::error()
            .with_message("invalid string literal")
            .with_labels(vec![
                Label::primary((), span.clone()).with_message(reason.as_str()),
            ]),

        LexicalError::InvalidEscapeSequence { span, sequence } => Diagnostic::error()
            .with_message(format!("invalid escape sequence `{sequence}`"))
            .with_labels(vec![
                Label::primary((), span.clone())
                    .with_message("this escape sequence is not recognised"),
            ])
            .with_notes(vec![
                "valid escape sequences: \\n  \\r  \\t  \\\\  \\'  \\\"  \\0  \\xNN".to_owned(),
            ]),

        LexicalError::UnexpectedCharacter { span, character } => Diagnostic::error()
            .with_message(format!("unexpected character `{character}`"))
            .with_labels(vec![
                Label::primary((), span.clone())
                    .with_message("this character is not valid in Fig source"),
            ]),

        LexicalError::UnrecognizedToken { span, text } => Diagnostic::error()
            .with_message(format!("unrecognized token `{text}`"))
            .with_labels(vec![
                Label::primary((), span.clone())
                    .with_message("the lexer cannot tokenise this sequence"),
            ]),

        LexicalError::InvalidToken => Diagnostic::error()
            .with_message("invalid token")
            .with_notes(vec![
                "a character or sequence was encountered that the lexer cannot handle".to_owned(),
            ]),
    }
}

// ============================================================================
// Test / CLI error wrapper
// ============================================================================

/// An error type that renders a pre-formatted diagnostic string *as-is*,
/// preserving newlines and box-drawing characters in test output.
///
/// See `fig-parser::diagnostics::DiagnosticError` for full rationale.
/// Both crates expose this type so callers never need to mix crate imports
/// just to get proper test failure rendering.
#[derive(Clone)]
pub struct DiagnosticError(pub String);

impl std::fmt::Debug for DiagnosticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::fmt::Display for DiagnosticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for DiagnosticError {}

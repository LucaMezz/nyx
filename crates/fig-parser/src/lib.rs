use lalrpop_util::lalrpop_mod;
use fig_lexer::{IndentLexer, Token};

pub mod ast;
pub mod diagnostics;
pub mod pretty_print;

// Re-export the most commonly needed diagnostics items at the crate root
// so downstream code doesn't have to reach into the sub-module.
pub use diagnostics::{
    build_diagnostic,
    emit_parse_error,
    format_parse_error,
    format_parse_errors,
    simple_diagnostic,
    token_name,
    DiagnosticError,
    FigParseError,
    FigRecoveredError,
};

/// Split the raw content of an interpolated-string literal into text and
/// expression-placeholder parts.
///
/// `raw` is the content between `$"` and the closing `"`, for example
/// `"Point({p.x}, {p.y})"` (without the outer quotes).
///
/// Expression spans like `{p.x}` are currently stored as
/// `InterpolatedPart::Text("{p.x}")` because the expressions are not
/// re-parsed at this stage; a later semantic pass can convert them.
pub fn parse_interp_parts(raw: String) -> Vec<ast::InterpolatedPart> {
    use ast::InterpolatedPart;
    let mut parts: Vec<InterpolatedPart> = Vec::new();
    let mut text = String::new();
    let mut expr = String::new();
    let mut depth: usize = 0;

    for c in raw.chars() {
        match (depth, c) {
            // Opening brace: flush any accumulated text, start an expression span
            (0, '{') => {
                if !text.is_empty() {
                    parts.push(InterpolatedPart::Text(std::mem::take(&mut text)));
                }
                depth = 1;
            }
            // Nested opening brace inside an expression
            (_, '{') => {
                depth += 1;
                expr.push(c);
            }
            // Closing brace that completes the outermost expression span
            (1, '}') => {
                depth = 0;
                // Store the raw expression source text as a Text placeholder.
                // The surrounding `{}` are preserved so downstream code knows
                // it was an interpolation site.
                parts.push(InterpolatedPart::Text(format!("{{{}}}", expr)));
                expr.clear();
            }
            // Nested closing brace
            (_, '}') => {
                depth -= 1;
                expr.push(c);
            }
            // Regular character outside an expression
            (0, _) => text.push(c),
            // Character inside an expression
            (_, _) => expr.push(c),
        }
    }

    // Flush any trailing text
    if !text.is_empty() {
        parts.push(InterpolatedPart::Text(text));
    }
    parts
}

#[cfg(test)]
mod tests;

lalrpop_mod!(pub parser);

pub use parser::*;

#[derive(Default, Debug, Clone, PartialEq)]
pub enum LexicalError {
    #[default]
    InvalidToken,
}

impl std::fmt::Display for LexicalError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

pub struct Lexer<'input> {
    indent_lexer: IndentLexer<'input>,
    position: usize,
    input: &'input str,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Self {
            indent_lexer: IndentLexer::new(input),
            position: 0,
            input,
        }
    }

    /// Return the original source string this lexer was created from.
    pub fn source(&self) -> &'input str {
        self.input
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Token, usize, LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.indent_lexer.next() {
            Some(Ok(token)) => {
                let span = self.indent_lexer.span();
                self.position = span.end;
                Some(Ok((span.start, token, span.end)))
            }
            Some(Err(_)) => {
                let _pos = self.position;
                self.position += 1;
                Some(Err(LexicalError::InvalidToken))
            }
            None => None,
        }
    }
}

// ============================================================================
// Byte-offset utilities (kept for use by diagnostics and other passes)
// ============================================================================

/// Convert a UTF-8 byte offset into a 1-based `(line, column)` pair.
///
/// Both line and column are 1-based.  If `offset` exceeds the length of
/// `source` it is clamped to the last valid position.
pub fn offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let offset = offset.min(source.len());
    let prefix = &source[..offset];
    let line = prefix.chars().filter(|&c| c == '\n').count() + 1;
    let col = prefix.rfind('\n').map_or(offset, |nl| offset - nl - 1) + 1;
    (line, col)
}

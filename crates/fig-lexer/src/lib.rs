mod float;
mod integer;
mod error;
pub mod diagnostics;

pub use float::{parse_float, FloatExponent, FloatLiteral, FloatSuffix};
pub use integer::{parse_integer, Base, IntegerLiteral, IntegerSuffix};
pub use error::LexicalError;
pub use diagnostics::{build_lex_diagnostic, emit_lex_error, format_lex_error, format_lex_errors, DiagnosticError};

use logos::Logos;
use serde::Serialize;

#[derive(Logos, Debug, PartialEq, Clone, Serialize)]
#[logos(error = LexicalError)]
#[logos(skip r"[ \t]+")] // Skip spaces and tabs only (not newlines)
pub enum Token {
    // Indentation tokens (emitted by IndentLexer wrapper)
    Indent,
    Dedent,
    
    // Newline token
    #[regex(r"(\r?\n)+")]
    Newline,
    // Keywords
    #[token("func")]
    Func,
    #[token("let")]
    Let,
    #[token("mut")]
    Mut,
    #[token("const")]
    Const,
    #[token("type")]
    Type,
    #[token("struct")]
    Struct,
    #[token("enum")]
    Enum,
    #[token("union")]
    Union,
    #[token("interface")]
    Interface,
    #[token("impl")]
    Impl,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("null")]
    Null,
    #[token("self")]
    SelfLower,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("elif")]
    Elif,
    #[token("for")]
    For,
    #[token("while")]
    While,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("match")]
    Match,
    #[token("return")]
    Return,
    #[token("in")]
    In,
    #[token("where")]
    Where,
    #[token("requires")]
    Requires,
    #[token("extends")]
    Extends,
    #[token("namespace")]
    Namespace,
    #[token("pass")]
    Pass,
    #[token("block")]
    Block,
    #[token("using")]
    Using,
    #[token("extern")]
    Extern,
    #[token("packed")]
    Packed,
    #[token("public")]
    Public,
    #[token("export")]
    Export,
    #[token("private")]
    Private,
    #[token("as")]
    As,
    #[token("sizeof")]
    Sizeof,
    #[token("alignof")]
    Alignof,
    #[token("offsetof")]
    Offsetof,

    // Primitive Types
    #[token("u8")]
    U8,
    #[token("u16")]
    U16,
    #[token("u32")]
    U32,
    #[token("u64")]
    U64,
    #[token("usize")]
    USize,
    #[token("isize")]
    ISize,
    #[token("i8")]
    I8,
    #[token("i16")]
    I16,
    #[token("i32")]
    I32,
    #[token("i64")]
    I64,
    #[token("f32")]
    F32,
    #[token("f64")]
    F64,
    #[token("bool")]
    Bool,

    // Operators
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("==")]
    EqEq,
    #[token("!=")]
    Ne,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("<=")]
    Le,
    #[token(">=")]
    Ge,
    #[token("&&")]
    AndAnd,
    #[token("||")]
    OrOr,
    #[token("!")]
    Bang,
    #[token("&")]
    And,
    #[token("|")]
    Or,
    #[token("^")]
    Caret,
    #[token("~")]
    Tilde,
    #[token("<<")]
    Shl,
    #[token(">>")]
    Shr,
    #[token("=")]
    Eq,
    #[token("+=")]
    PlusEq,
    #[token("-=")]
    MinusEq,
    #[token("*=")]
    StarEq,
    #[token("/=")]
    SlashEq,
    #[token("%=")]
    PercentEq,
    #[token("&=")]
    AndEq,
    #[token("|=")]
    OrEq,
    #[token("^=")]
    CaretEq,
    #[token("<<=")]
    ShlEq,
    #[token(">>=")]
    ShrEq,
    #[token("->")]
    Arrow, // Function return type
    #[token("=>")]
    FatArrow,
    #[token("?")]
    Question,

    // Delimiters and Punctuation
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("::")]
    ColonColon,
    #[token(":")]
    Colon,
    #[token(";")]
    Semicolon,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,

    // Special wildcard identifier
    #[token("_")]
    Underscore,

    // Identifiers - Must come after keywords and special identifiers to avoid false positives
    // Modified to not match a single underscore
    #[regex("[a-zA-Z][a-zA-Z0-9_]*|_[a-zA-Z0-9_]+", |lex| lex.slice().to_string())]
    Ident(String),

    // ------------------------
    // Unsuffixed Literals
    // ------------------------
    #[regex(
        r"[0-9][_0-9]*\.[0-9][_0-9]*(?:e[+-]?[0-9][_0-9]*)?(f32|f64)?",
        parse_float
    )]
    #[regex(r"[0-9][_0-9]*e[+-]?[0-9][_0-9]*(f32|f64)?", parse_float)]
    FloatLiteral(FloatLiteral),

    #[regex(
        r"[0-9][_0-9]*(i8|i16|i32|i64|isize|u8|u16|u32|u64|usize)?",
        parse_integer
    )]
    #[regex(
        r"0b[01][_01]*(i8|i16|i32|i64|isize|u8|u16|u32|u64|usize)?",
        parse_integer
    )]
    #[regex(
        r"0o_?[0-7][_0-7]*(i8|i16|i32|i64|isize|u8|u16|u32|u64|usize)?",
        parse_integer
    )]
    #[regex(
        r"0x_?[0-9a-fA-F][_0-9a-fA-F]*(i8|i16|i32|i64|isize|u8|u16|u32|u64|usize)?",
        parse_integer
    )]
    IntegerLiteral(IntegerLiteral), // unsuffixed integers

    // Interpolated string literal: $"text {expr} text"
    // The raw content (between $" and the closing ") is stored as-is; the parser
    // splits it into Text/Expression parts via parse_interp_parts.
    #[regex(r#"\$"(?:[^"\\]|\\.)*""#, lex_interp_string)]
    InterpolatedStringLiteral(String),

    // String literal with escape sequences (including \xNN hex escapes)
    #[regex(r#""(?:[^"\\]|\\.)*""#, unescape_literal)]
    // This regex will NOT match unclosed strings, so Logos will emit an error for them
    StringLiteral(String),

    // Character literal with basic escape sequences
    #[regex(r#"'(?:[^'\\]|\\.)'"#, unescape_literal)]
    CharLiteral(String),

    // Metadata prefix - '#' symbol for annotations. The identifier following it will be `Ident`.
    #[token("#")]
    Hash,

    // Whitespace and Comments - Skipped by Logos
    #[regex(r"//[^\n]*", logos::skip, allow_greedy = true)] // Single-line comments
    #[regex(r"/\*[^*]*\*+(?:[^/*][^*]*\*+)*/", logos::skip, allow_greedy = true)]
    // Multi-line comments
    Comment,
}

// Helper function to unescape string and character literals
fn unescape_literal(lex: &mut logos::Lexer<Token>) -> Result<String, LexicalError> {
    let lex_slice = lex.slice();
    let span = lex.span();
    let mut unescaped = String::with_capacity(lex_slice.len());
    
    // Remove leading/trailing quotes for both char and string literals
    // The slice will be like "'a'" or "\"hello\""
    let inner_slice = &lex_slice[1..lex_slice.len() - 1]; // Removes ' or "

    let mut chars = inner_slice.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(escaped_char) = chars.next() {
                match escaped_char {
                    'n' => unescaped.push('\n'),
                    'r' => unescaped.push('\r'),
                    't' => unescaped.push('\t'),
                    '\\' => unescaped.push('\\'),
                    '0' => unescaped.push('\0'),
                    '\'' => unescaped.push('\''),
                    '"' => unescaped.push('"'),
                    // \xNN — two-digit hex escape (e.g. \x1b for ESC)
                    'x' => {
                        let h1 = chars.next().ok_or_else(|| LexicalError::InvalidEscapeSequence {
                            span: span.clone(),
                            sequence: "\\x".to_string(),
                        })?;
                        let h2 = chars.next().ok_or_else(|| LexicalError::InvalidEscapeSequence {
                            span: span.clone(),
                            sequence: format!("\\x{}", h1),
                        })?;
                        let hex_str = format!("{}{}", h1, h2);
                        let code = u32::from_str_radix(&hex_str, 16).map_err(|_| {
                            LexicalError::InvalidEscapeSequence {
                                span: span.clone(),
                                sequence: format!("\\x{}", hex_str),
                            }
                        })?;
                        let ch = char::from_u32(code).ok_or_else(|| LexicalError::InvalidEscapeSequence {
                            span: span.clone(),
                            sequence: format!("\\x{}", hex_str),
                        })?;
                        unescaped.push(ch);
                    }
                    _ => {
                        // Invalid escape sequence
                        return Err(LexicalError::InvalidEscapeSequence {
                            span,
                            sequence: format!("\\{}", escaped_char),
                        });
                    }
                }
            } else {
                // Backslash at the end of the string, which is an error
                return Err(LexicalError::InvalidEscapeSequence {
                    span,
                    sequence: "\\".to_string(),
                });
            }
        } else {
            unescaped.push(c);
        }
    }
    Ok(unescaped)
}

/// Captures the raw content of an interpolated string (between `$"` and the closing `"`)
/// without processing escape sequences or `{...}` expression spans.
/// Those are handled later by the parser's `parse_interp_parts` helper.
fn lex_interp_string(lex: &mut logos::Lexer<Token>) -> String {
    let slice = lex.slice();
    // slice looks like:  $"Point({p.x}, {p.y})"
    // strip leading  $"  (2 bytes) and trailing  "  (1 byte)
    slice[2..slice.len() - 1].to_string()
}

/// Wrapper around Logos lexer that handles indentation-based block structure.
/// Emits INDENT tokens when indentation increases by 4 spaces,
/// and DEDENT tokens when indentation decreases by 4 spaces.
pub struct IndentLexer<'source> {
    lexer: logos::Lexer<'source, Token>,
    source: &'source str,
    indent_stack: Vec<usize>, // Stack of indentation levels
    pending_tokens: Vec<Token>, // Queue of tokens to emit
    at_line_start: bool,
    last_was_newline: bool,
}

impl<'source> IndentLexer<'source> {
    pub fn new(source: &'source str) -> Self {
        Self {
            lexer: Token::lexer(source),
            source,
            indent_stack: vec![0], // Start with 0 indentation
            pending_tokens: Vec::new(),
            at_line_start: true,
            last_was_newline: true,
        }
    }

    /// Get the current span of the underlying lexer
    pub fn span(&self) -> std::ops::Range<usize> {
        self.lexer.span()
    }

    /// Get the current slice from the underlying lexer
    pub fn slice(&self) -> &'source str {
        self.lexer.slice()
    }

    /// Calculate indentation level at the current position (before current token)
    fn calculate_line_indentation(&self, token_start: usize) -> usize {
        // Walk backwards from token_start to find the last newline
        let mut pos = token_start;
        while pos > 0 && self.source.as_bytes()[pos - 1] != b'\n' {
            pos -= 1;
        }
        
        // Now count spaces from pos to token_start
        let mut indent = 0;
        for i in pos..token_start {
            match self.source.as_bytes()[i] {
                b' ' => indent += 1,
                b'\t' => indent += 4, // Tab = 4 spaces
                _ => break,
            }
        }
        
        indent
    }

    /// Check if line is blank or only contains comments
    fn is_blank_or_comment_line(&self, token_start: usize) -> bool {
        // Walk backwards to find line start
        let mut pos = token_start;
        while pos > 0 && self.source.as_bytes()[pos - 1] != b'\n' {
            pos -= 1;
        }
        
        // Check what's between line start and current position (should be only whitespace)
        // and what comes after current position
        let line_before = &self.source[pos..token_start];
        let line_after = &self.source[token_start..];
        
        // If current position is a newline or starts with //, it's blank or comment
        line_before.chars().all(|c| c == ' ' || c == '\t') &&
            (line_after.starts_with('\n') || line_after.starts_with("//"))
    }

    /// Process indentation change after seeing a newline
    fn handle_indentation(&mut self, token_start: usize) -> Vec<Token> {
        let indent_level = self.calculate_line_indentation(token_start);
        let current_indent = *self.indent_stack.last().unwrap();
        let mut tokens = Vec::new();

        if indent_level > current_indent {
            // Increased indentation
            let diff = indent_level - current_indent;
            let num_indents = diff / 4;
            
            for i in 1..=num_indents {
                self.indent_stack.push(current_indent + (i * 4));
                tokens.push(Token::Indent);
            }
        } else if indent_level < current_indent {
            // Decreased indentation
            while let Some(&level) = self.indent_stack.last() {
                if level <= indent_level {
                    break;
                }
                self.indent_stack.pop();
                tokens.push(Token::Dedent);
            }
        }

        tokens
    }
}

impl<'source> Iterator for IndentLexer<'source> {
    type Item = Result<Token, LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        // First, check if we have pending tokens to emit
        if !self.pending_tokens.is_empty() {
            return Some(Ok(self.pending_tokens.remove(0)));
        }

        // Get next token from underlying lexer
        match self.lexer.next() {
            Some(Ok(Token::Newline)) => {
                if self.last_was_newline {
                    // Collapse consecutive blank lines into a single Newline
                    return self.next();
                }
                self.last_was_newline = true;
                self.at_line_start = true;
                Some(Ok(Token::Newline))
            }
            Some(Ok(token)) => {
                // If we're at the start of a line (after newline), handle indentation
                if self.last_was_newline && !matches!(token, Token::Newline) {
                    self.last_was_newline = false;
                    
                    let token_start = self.lexer.span().start;
                    
                    // Skip blank lines and comment lines for indentation purposes
                    if !self.is_blank_or_comment_line(token_start) {
                        let indent_tokens = self.handle_indentation(token_start);
                        
                        if !indent_tokens.is_empty() {
                            // Add indent/dedent tokens to pending (all except first)
                            for i in 1..indent_tokens.len() {
                                self.pending_tokens.push(indent_tokens[i].clone());
                            }
                            // Store the real token at the end
                            self.pending_tokens.push(token);
                            // Return first indent/dedent token
                            return Some(Ok(indent_tokens[0].clone()));
                        }
                    }
                }
                
                Some(Ok(token))
            }
            Some(Err(e)) => Some(Err(e)),
            None => {
                // EOF: emit remaining dedents
                if self.indent_stack.len() > 1 {
                    self.indent_stack.pop();
                    Some(Ok(Token::Dedent))
                } else {
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use logos::Logos;

    fn lexer_test_helper(input: &str, expected_tokens: Vec<Token>) {
        let mut lex = Token::lexer(input);
        for expected_token in expected_tokens {
            assert_eq!(lex.next().unwrap().unwrap(), expected_token);
        }
        assert_eq!(lex.next(), None); // Ensure no more tokens
    }

    // --- Simple Tests ---

    #[test]
    fn test_keywords() {
        lexer_test_helper(
            "let mut const type struct union interface impl true false if else for while break continue match return usize in",
            vec![
                Token::Let,
                Token::Mut,
                Token::Const,
                Token::Type,
                Token::Struct,
                Token::Union,
                Token::Interface,
                Token::Impl,
                Token::True,
                Token::False,
                Token::If,
                Token::Else,
                Token::For,
                Token::While,
                Token::Break,
                Token::Continue,
                Token::Match,
                Token::Return,
                Token::USize,
                Token::In,
            ],
        );
    }

    #[test]
    fn test_primitive_types() {
        lexer_test_helper(
            "u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 bool",
            vec![
                Token::U8,
                Token::U16,
                Token::U32,
                Token::U64,
                Token::I8,
                Token::I16,
                Token::I32,
                Token::I64,
                Token::F32,
                Token::F64,
                Token::Bool,
            ],
        );
    }

    #[test]
    fn test_operators() {
        lexer_test_helper(
            "+ - * / % == != < > <= >= && || ! & | ^ ~ << >> = += -= *= /= %= &= |= ^= <<= >>=",
            vec![
                Token::Plus,
                Token::Minus,
                Token::Star,
                Token::Slash,
                Token::Percent,
                Token::EqEq,
                Token::Ne,
                Token::Lt,
                Token::Gt,
                Token::Le,
                Token::Ge,
                Token::AndAnd,
                Token::OrOr,
                Token::Bang,
                Token::And,
                Token::Or,
                Token::Caret,
                Token::Tilde,
                Token::Shl,
                Token::Shr,
                Token::Eq,
                Token::PlusEq,
                Token::MinusEq,
                Token::StarEq,
                Token::SlashEq,
                Token::PercentEq,
                Token::AndEq,
                Token::OrEq,
                Token::CaretEq,
                Token::ShlEq,
                Token::ShrEq,
            ],
        );
    }

    #[test]
    fn test_delimiters_and_punctuation() {
        lexer_test_helper(
            "() {} [] : ; , . ->",
            vec![
                Token::LParen,
                Token::RParen,
                Token::LBrace,
                Token::RBrace,
                Token::LBracket,
                Token::RBracket,
                Token::Colon,
                Token::Semicolon,
                Token::Comma,
                Token::Dot,
                Token::Arrow,
            ],
        );
    }

    #[test]
    fn test_identifiers_and_underscore() {
        lexer_test_helper(
            "myVar _ another_var _123 var_name_long _leading_underscore",
            vec![
                Token::Ident("myVar".to_string()),
                Token::Underscore,
                Token::Ident("another_var".to_string()),
                Token::Ident("_123".to_string()),
                Token::Ident("var_name_long".to_string()),
                Token::Ident("_leading_underscore".to_string()),
            ],
        );
    }

    // --- Complex Tests (Numeric Literals with Suffixes) ---

    #[test]
    fn test_integer_literals_with_suffixes() {
        lexer_test_helper(
            "123u8 0b101u16 0o77u32 0xAFu64 10i8 0b11i16 0o12i32 0xFFi64 5usize",
            vec![
                Token::IntegerLiteral(
                    IntegerLiteral::builder()
                        .base(Base::Decimal)
                        .digits("123".to_string())
                        .suffix(Some(IntegerSuffix::U8))
                        .build()
                        .unwrap(),
                ),
                Token::IntegerLiteral(
                    IntegerLiteral::builder()
                        .base(Base::Binary)
                        .digits("101".to_string())
                        .suffix(Some(IntegerSuffix::U16))
                        .build()
                        .unwrap(),
                ),
                Token::IntegerLiteral(
                    IntegerLiteral::builder()
                        .base(Base::Octal)
                        .digits("77".to_string())
                        .suffix(Some(IntegerSuffix::U32))
                        .build()
                        .unwrap(),
                ),
                Token::IntegerLiteral(
                    IntegerLiteral::builder()
                        .base(Base::Hex)
                        .digits("AF".to_string())
                        .suffix(Some(IntegerSuffix::U64))
                        .build()
                        .unwrap(),
                ),
                Token::IntegerLiteral(
                    IntegerLiteral::builder()
                        .base(Base::Decimal)
                        .digits("10".to_string())
                        .suffix(Some(IntegerSuffix::I8))
                        .build()
                        .unwrap(),
                ),
                Token::IntegerLiteral(
                    IntegerLiteral::builder()
                        .base(Base::Binary)
                        .digits("11".to_string())
                        .suffix(Some(IntegerSuffix::I16))
                        .build()
                        .unwrap(),
                ),
                Token::IntegerLiteral(
                    IntegerLiteral::builder()
                        .base(Base::Octal)
                        .digits("12".to_string())
                        .suffix(Some(IntegerSuffix::I32))
                        .build()
                        .unwrap(),
                ),
                Token::IntegerLiteral(
                    IntegerLiteral::builder()
                        .base(Base::Hex)
                        .digits("FF".to_string())
                        .suffix(Some(IntegerSuffix::I64))
                        .build()
                        .unwrap(),
                ),
                Token::IntegerLiteral(
                    IntegerLiteral::builder()
                        .base(Base::Decimal)
                        .digits("5".to_string())
                        .suffix(Some(IntegerSuffix::USize))
                        .build()
                        .unwrap(),
                ),
            ],
        );
        lexer_test_helper(
            "1_000u32 0b1_0u8 0o_7_7u16 0x_AF_u64", // Underscores in numbers
            vec![
                Token::IntegerLiteral(
                    IntegerLiteral::builder()
                        .base(Base::Decimal)
                        .digits("1000".to_string())
                        .suffix(Some(IntegerSuffix::U32))
                        .build()
                        .unwrap(),
                ),
                Token::IntegerLiteral(
                    IntegerLiteral::builder()
                        .base(Base::Binary)
                        .digits("10".to_string())
                        .suffix(Some(IntegerSuffix::U8))
                        .build()
                        .unwrap(),
                ),
                Token::IntegerLiteral(
                    IntegerLiteral::builder()
                        .base(Base::Octal)
                        .digits("77".to_string())
                        .suffix(Some(IntegerSuffix::U16))
                        .build()
                        .unwrap(),
                ),
                Token::IntegerLiteral(
                    IntegerLiteral::builder()
                        .base(Base::Hex)
                        .digits("AF".to_string())
                        .suffix(Some(IntegerSuffix::U64))
                        .build()
                        .unwrap(),
                ),
            ],
        );
    }

    #[test]
    fn test_float_literals_with_suffixes() {
        lexer_test_helper(
            "3.14f32 1.0e-5f64 2f32 2.f32", // Note: 2f32 might be `2` (Integer) followed by `f32` (Ident)
            vec![
                Token::FloatLiteral(
                    FloatLiteral::builder()
                        .digits("3.14".to_string())
                        .exponent(None)
                        .suffix(Some(FloatSuffix::F32))
                        .build()
                        .unwrap(),
                ),
                Token::FloatLiteral(
                    FloatLiteral::builder()
                        .digits("1.0".to_string())
                        .exponent(Some(FloatExponent::Negative(5)))
                        .suffix(Some(FloatSuffix::F64))
                        .build()
                        .unwrap(),
                ),
                Token::IntegerLiteral(
                    IntegerLiteral::builder()
                        .base(Base::Decimal)
                        .digits("2".to_string())
                        .suffix(None)
                        .build()
                        .unwrap(),
                ),
                Token::F32,
                Token::IntegerLiteral(
                    IntegerLiteral::builder()
                        .base(Base::Decimal)
                        .digits("2".to_string())
                        .suffix(None)
                        .build()
                        .unwrap(),
                ),
                Token::Dot,
                Token::F32,
            ],
        );
        // Correct float with exponent only
        lexer_test_helper(
            "1e5f32 1.23e+10f64",
            vec![
                Token::FloatLiteral(
                    FloatLiteral::builder()
                        .digits("1".to_string())
                        .exponent(Some(FloatExponent::Unsigned(5)))
                        .suffix(Some(FloatSuffix::F32))
                        .build()
                        .unwrap(),
                ),
                Token::FloatLiteral(
                    FloatLiteral::builder()
                        .digits("1.23".to_string())
                        .exponent(Some(FloatExponent::Positive(10)))
                        .suffix(Some(FloatSuffix::F64))
                        .build()
                        .unwrap(),
                ),
            ],
        );
        // Test cases that might be problematic: 2.f32 - current regex doesn't handle trailing dot without digits.
        // It's `[0-9][_0-9]*\.[0-9][_0-9]*` so `2.` won't match as Float. It'll be `IntegerLiteral` + `Dot`.
        // This is a language design choice.
    }

    #[test]
    fn test_mixed_simple_literals() {
        lexer_test_helper(
            "42 3.14 0b10 0o7 0xFa 'c' \"hello\"",
            vec![
                Token::IntegerLiteral(
                    IntegerLiteral::builder()
                        .base(Base::Decimal)
                        .digits("42".to_string())
                        .suffix(None)
                        .build()
                        .unwrap(),
                ),
                Token::FloatLiteral(
                    FloatLiteral::builder()
                        .digits("3.14".to_string())
                        .exponent(None)
                        .suffix(None)
                        .build()
                        .unwrap(),
                ),
                Token::IntegerLiteral(
                    IntegerLiteral::builder()
                        .base(Base::Binary)
                        .digits("10".to_string())
                        .suffix(None)
                        .build()
                        .unwrap(),
                ),
                Token::IntegerLiteral(
                    IntegerLiteral::builder()
                        .base(Base::Octal)
                        .digits("7".to_string())
                        .suffix(None)
                        .build()
                        .unwrap(),
                ),
                Token::IntegerLiteral(
                    IntegerLiteral::builder()
                        .base(Base::Hex)
                        .digits("Fa".to_string())
                        .suffix(None)
                        .build()
                        .unwrap(),
                ),
                Token::CharLiteral("c".to_string()),
                Token::StringLiteral("hello".to_string()),
            ],
        );
    }

    #[test]
    fn test_string_and_char_literals() {
        lexer_test_helper(
            r#"'a' '\n' '\'' '\\' "hello world" "tab\tnew\nline""#,
            vec![
                Token::CharLiteral("a".to_string()),
                Token::CharLiteral("\n".to_string()),
                Token::CharLiteral("'".to_string()),
                Token::CharLiteral("\\".to_string()),
                Token::StringLiteral("hello world".to_string()),
                Token::StringLiteral("tab\tnew\nline".to_string()),
            ],
        );
    }

    #[test]
    fn test_metadata() {
        lexer_test_helper(
            "#inline #deprecated(msg) #test_attr",
            vec![
                Token::Hash,
                Token::Ident("inline".to_string()),
                Token::Hash,
                Token::Ident("deprecated".to_string()),
                Token::LParen,
                Token::Ident("msg".to_string()),
                Token::RParen,
                Token::Hash,
                Token::Ident("test_attr".to_string()),
            ],
        );
    }

    #[test]
    fn test_comments_and_whitespace() {
        lexer_test_helper(
            "  // single line comment\n let /*multi\nline\ncomment*/ x = 10",
            vec![
                Token::Newline,
                Token::Let,
                Token::Ident("x".to_string()),
                Token::Eq,
                Token::IntegerLiteral(
                    IntegerLiteral::builder()
                        .base(Base::Decimal)
                        .digits("10".to_string())
                        .suffix(None)
                        .build()
                        .unwrap(),
                ),
            ],
        );
    }

    #[test]
    fn test_complex_expression() {
        let input = "func add(a: i32, b: i32) -> i32 { return a + b; }";
        let expected = vec![
            Token::Func,
            Token::Ident("add".to_string()),
            Token::LParen,
            Token::Ident("a".to_string()),
            Token::Colon,
            Token::I32,
            Token::Comma,
            Token::Ident("b".to_string()),
            Token::Colon,
            Token::I32,
            Token::RParen,
            Token::Arrow,
            Token::I32,
            Token::LBrace,
            Token::Return,
            Token::Ident("a".to_string()),
            Token::Plus,
            Token::Ident("b".to_string()),
            Token::Semicolon,
            Token::RBrace,
        ];
        lexer_test_helper(input, expected);
    }

    #[test]
    fn test_match_statement() {
        let input = "match value { 0 => handle_zero(), _ => handle_other() }";
        let expected = vec![
            Token::Match,
            Token::Ident("value".to_string()),
            Token::LBrace,
            Token::IntegerLiteral(
                IntegerLiteral::builder()
                    .base(Base::Decimal)
                    .digits("0".to_string())
                    .suffix(None)
                    .build()
                    .unwrap(),
            ),
            Token::FatArrow,
            Token::Ident("handle_zero".to_string()),
            Token::LParen,
            Token::RParen,
            Token::Comma,
            Token::Underscore,
            Token::FatArrow,
            Token::Ident("handle_other".to_string()),
            Token::LParen,
            Token::RParen,
            Token::RBrace,
        ];
        lexer_test_helper(input, expected);
    }

    // --- Edge Case Tests ---

    #[test]
    fn test_empty_input() {
        let mut lex = Token::lexer("");
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_unrecognized_character() {
        let mut lex = Token::lexer("let $invalid;"); // $ is not tokenized
        assert_eq!(lex.next().unwrap().unwrap(), Token::Let);
        assert_eq!(lex.next().unwrap().is_err(), true); // Should be an error for $
        // The error token might be followed by valid tokens, depending on Logos' recovery
        // However, we expect the next valid token to be 'invalid' if Logos recovers.
        // For this test, we just check if it produces an error.
        assert_eq!(lex.next().unwrap().unwrap(), Token::Ident("invalid".to_string())); // Expect 'invalid'
        assert_eq!(lex.next().unwrap().unwrap(), Token::Semicolon); // Expect ';'
        assert_eq!(lex.next(), None); // Now expect no more tokens
    }

    #[test]
    fn test_string_literal_with_unclosed_quote() {
        let mut lex = Token::lexer("\"unclosed string");
        assert!(lex.next().unwrap().is_err()); // Should produce an error
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_char_literal_with_unclosed_quote() {
        let mut lex = Token::lexer(r#"'u"#);
        assert!(lex.next().unwrap().is_err()); // Should produce an error
        assert_eq!(lex.next(), None);
    }

    #[test]
    fn test_incomplete_float_literal() {
        // "1." is an IntegerLiteral then Dot, as per regex
        lexer_test_helper(
            "1. f32",
            vec![
                Token::IntegerLiteral(
                    IntegerLiteral::builder()
                        .base(Base::Decimal)
                        .digits("1".to_string())
                        .suffix(None)
                        .build()
                        .unwrap(),
                ),
                Token::Dot,
                Token::F32,
            ],
        );
    }

    #[test]
    fn test_float_vs_integer_no_suffix() {
        lexer_test_helper(
            "100 100.0 1e5 0xAf",
            vec![
                Token::IntegerLiteral(
                    IntegerLiteral::builder()
                        .base(Base::Decimal)
                        .digits("100".to_string())
                        .suffix(None)
                        .build()
                        .unwrap(),
                ),
                Token::FloatLiteral(
                    FloatLiteral::builder()
                        .digits("100.0".to_string())
                        .exponent(None)
                        .suffix(None)
                        .build()
                        .unwrap(),
                ),
                Token::FloatLiteral(
                    FloatLiteral::builder()
                        .digits("1".to_string())
                        .exponent(Some(FloatExponent::Unsigned(5)))
                        .suffix(None)
                        .build()
                        .unwrap(),
                ),
                Token::IntegerLiteral(
                    IntegerLiteral::builder()
                        .base(Base::Hex)
                        .digits("Af".to_string())
                        .suffix(None)
                        .build()
                        .unwrap(),
                ),
            ],
        );
    }

    // --- Indentation Tests ---

    fn indent_lexer_test_helper(input: &str, expected_tokens: Vec<Token>) {
        let mut lexer = IndentLexer::new(input);
        for (i, expected_token) in expected_tokens.iter().enumerate() {
            let token = lexer.next();
            assert!(token.is_some(), "Expected token at position {}, but got None", i);
            let token = token.unwrap();
            assert!(token.is_ok(), "Expected Ok token at position {}, but got Err", i);
            assert_eq!(
                token.unwrap(),
                *expected_token,
                "Token mismatch at position {}",
                i
            );
        }
        assert_eq!(lexer.next(), None, "Expected no more tokens, but got Some");
    }

    #[test]
    fn test_basic_indentation() {
        let input = "func test\n    let x\n    let y";
        let expected = vec![
            Token::Func,
            Token::Ident("test".to_string()),
            Token::Newline,
            Token::Indent,
            Token::Let,
            Token::Ident("x".to_string()),
            Token::Newline,
            Token::Let,
            Token::Ident("y".to_string()),
            Token::Dedent, // EOF dedent
        ];
        indent_lexer_test_helper(input, expected);
    }

    #[test]
    fn test_indentation_increase_and_decrease() {
        let input = "func test\n    if true\n        let x\nlet y";
        let expected = vec![
            Token::Func,
            Token::Ident("test".to_string()),
            Token::Newline,
            Token::Indent,
            Token::If,
            Token::True,
            Token::Newline,
            Token::Indent,
            Token::Let,
            Token::Ident("x".to_string()),
            Token::Newline,
            Token::Dedent,
            Token::Dedent,
            Token::Let,
            Token::Ident("y".to_string()),
        ];
        indent_lexer_test_helper(input, expected);
    }

    #[test]
    fn test_multiple_indent_levels() {
        let input = "func outer\n    func middle\n        func inner\n            let x";
        let expected = vec![
            Token::Func,
            Token::Ident("outer".to_string()),
            Token::Newline,
            Token::Indent,
            Token::Func,
            Token::Ident("middle".to_string()),
            Token::Newline,
            Token::Indent,
            Token::Func,
            Token::Ident("inner".to_string()),
            Token::Newline,
            Token::Indent,
            Token::Let,
            Token::Ident("x".to_string()),
            Token::Dedent, // EOF dedents
            Token::Dedent,
            Token::Dedent,
        ];
        indent_lexer_test_helper(input, expected);
    }

    #[test]
    fn test_dedent_multiple_levels() {
        let input = "func test\n    if true\n        if false\n            let x\nlet y";
        let expected = vec![
            Token::Func,
            Token::Ident("test".to_string()),
            Token::Newline,
            Token::Indent,
            Token::If,
            Token::True,
            Token::Newline,
            Token::Indent,
            Token::If,
            Token::False,
            Token::Newline,
            Token::Indent,
            Token::Let,
            Token::Ident("x".to_string()),
            Token::Newline,
            Token::Dedent,
            Token::Dedent,
            Token::Dedent,
            Token::Let,
            Token::Ident("y".to_string()),
        ];
        indent_lexer_test_helper(input, expected);
    }

    #[test]
    fn test_no_indentation_change() {
        let input = "let x\nlet y\nlet z";
        let expected = vec![
            Token::Let,
            Token::Ident("x".to_string()),
            Token::Newline,
            Token::Let,
            Token::Ident("y".to_string()),
            Token::Newline,
            Token::Let,
            Token::Ident("z".to_string()),
        ];
        indent_lexer_test_helper(input, expected);
    }

    #[test]
    fn test_blank_lines_ignored() {
        let input = "func test\n\n    let x\n\nlet y";
        let expected = vec![
            Token::Func,
            Token::Ident("test".to_string()),
            Token::Newline,  // Multiple newlines collapsed into one
            Token::Indent,
            Token::Let,
            Token::Ident("x".to_string()),
            Token::Newline,  // Multiple newlines collapsed into one
            Token::Dedent,
            Token::Let,
            Token::Ident("y".to_string()),
        ];
        indent_lexer_test_helper(input, expected);
    }

    #[test]
    fn test_indentation_with_expressions() {
        let input = "if x\n    let a = 1\n    let b = 2";
        let expected = vec![
            Token::If,
            Token::Ident("x".to_string()),
            Token::Newline,
            Token::Indent,
            Token::Let,
            Token::Ident("a".to_string()),
            Token::Eq,
            Token::IntegerLiteral(
                IntegerLiteral::builder()
                    .base(Base::Decimal)
                    .digits("1".to_string())
                    .suffix(None)
                    .build()
                    .unwrap(),
            ),
            Token::Newline,
            Token::Let,
            Token::Ident("b".to_string()),
            Token::Eq,
            Token::IntegerLiteral(
                IntegerLiteral::builder()
                    .base(Base::Decimal)
                    .digits("2".to_string())
                    .suffix(None)
                    .build()
                    .unwrap(),
            ),
            Token::Dedent, // EOF dedent
        ];
        indent_lexer_test_helper(input, expected);
    }
}

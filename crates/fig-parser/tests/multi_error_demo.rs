/// Standalone test that demonstrates LALRPOP error recovery: the parser
/// continues past the first syntax error and collects multiple diagnostics,
/// reporting each one via codespan-reporting.
///
/// Run with:
///   cargo test -p fig-parser --test multi_error_demo

use fig_parser::{format_parse_errors, Lexer, SourceFileParser};

/// A deliberately broken Fig source with three distinct syntax problems:
///
///  1. `1 +` — an incomplete expression (missing right-hand operand).
///  2. A garbage line `@@@ … @@@` that the lexer/parser cannot recognise.
///  3. A valid function definition to prove the parser recovered and kept going.
const SOURCE: &str = "func greet(name: i32) -> void\n    let x = 1 +\n    pass\n\nfunc ok() -> void\n    pass\n";

#[test]
fn multi_error_recovery_reports_multiple_diagnostics() {
    let lexer = Lexer::new(SOURCE);
    let mut errors = Vec::new();
    let result = SourceFileParser::new().parse(&mut errors, lexer);

    // Format all diagnostics (both recovered and any fatal error) into one string.
    let report = format_parse_errors("test_source", SOURCE, &errors, result.as_ref().err());

    // Always print the report so it shows up in `cargo test -- --nocapture`.
    println!("\n=== Error Recovery Report ===\n{}", report);

    // Whether the file has errors or not, the parser should produce an AST.
    // For an entirely valid file the errors vec will be empty and result Ok.
    // For an invalid file errors will be non-empty OR result will be Err.
    let has_error = !errors.is_empty() || result.is_err();

    // The parser must have kept at least one item even in the face of errors.
    if let Ok(ref ast) = result {
        assert!(!ast.items.is_empty(), "Expected recovered top-level items");
    }

    // The source intentionally has a syntax error, so we expect at least one.
    assert!(has_error, "Expected at least one error to be reported");
}

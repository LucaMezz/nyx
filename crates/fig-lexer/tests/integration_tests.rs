// Integration tests for the Fig lexer using datatest-stable + insta
// Each .fig file in tests/valid/ will be tested automatically
// Token snapshots are stored in .snap.yml files using insta

use datatest_stable::Utf8Path;
use fig_lexer::{diagnostics::DiagnosticError, format_lex_errors, IndentLexer};

fn lexer_test(path: &Utf8Path, contents: String) -> datatest_stable::Result<()> {
    // Tokenize the entire file
    let mut lexer = IndentLexer::new(&contents);
    let mut tokens = Vec::new();
    let mut errors = Vec::new();

    while let Some(result) = lexer.next() {
        match result {
            Ok(token) => tokens.push(token),
            Err(e) => errors.push(e),
        }
    }

    // Check for lexer errors
    if !errors.is_empty() {
        let rendered = format_lex_errors(path.as_str(), &contents, &errors);
        return Err(Box::new(DiagnosticError(rendered)));
    }

    // Create snapshot name from the test path
    // Convert path like "../../tests/valid/comments/single_line.fig" 
    // to snapshot name "comments__single_line"
    let snapshot_name = path
        .as_str()
        .trim_start_matches("../../tests/valid/")
        .trim_end_matches(".fig")
        .replace('/', "__")
        .replace('\\', "__");

    // Assert snapshot using insta with YAML format
    insta::assert_yaml_snapshot!(snapshot_name, tokens);

    Ok(())
}

datatest_stable::harness! {
    { test = lexer_test, root = "../../tests/valid", pattern = r"\.fig$" },
}

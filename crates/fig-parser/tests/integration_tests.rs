// Integration tests for the Fig parser using datatest-stable + insta
// Each .fig file in tests/valid/ will be tested automatically
// AST snapshots are stored in .snap.yml files using insta

use datatest_stable::Utf8Path;
use fig_parser::{
    format_parse_errors, DiagnosticError, Lexer, SourceFileParser,
};

fn parser_test(path: &Utf8Path, contents: String) -> datatest_stable::Result<()> {
    let lexer = Lexer::new(&contents);
    let parser = SourceFileParser::new();

    // Collect error-recovery tokens into this vector.
    let mut errors = Vec::new();

    // Parse the entire file — the grammar parameter is the errors vector.
    let result = parser.parse(&mut errors, lexer);

    // Any failure (recovered errors OR a fatal parse error) is reported as a
    // combined diagnostic so that every problem is visible at once.
    let ast = match result {
        Ok(ast) if errors.is_empty() => ast,
        Ok(_ast) => {
            // Recovered errors on a file that is supposed to be valid —
            // likely a grammar regression.  Report but only show diagnostics.
            let msg = format_parse_errors(path.as_str(), &contents, &errors, None);
            return Err(Box::new(DiagnosticError(format!(
                "recovered errors on a supposedly valid file:\n{msg}"
            ))));
        }
        Err(e) => {
            let msg = format_parse_errors(path.as_str(), &contents, &errors, Some(&e));
            return Err(Box::new(DiagnosticError(msg)));
        }
    };

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
    insta::assert_yaml_snapshot!(snapshot_name, ast);

    Ok(())
}

datatest_stable::harness! {
    { test = parser_test, root = "../../tests/valid", pattern = r"\.fig$" },
}

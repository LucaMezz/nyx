// Integration tests for the Fig parser using datatest-stable + insta
// Each .fig file in tests/valid/ will be tested automatically
// AST snapshots are stored in .snap.yml files using insta

use datatest_stable::Utf8Path;
use fig_parser::{format_parse_error, DiagnosticError, Lexer, SourceFileParser};

fn parser_test(path: &Utf8Path, contents: String) -> datatest_stable::Result<()> {
    let lexer = Lexer::new(&contents);
    let parser = SourceFileParser::new();

    // Parse the entire file
    let result = parser.parse(lexer);

    // Assert that parsing succeeded
    let ast = match result {
        Ok(ast) => ast,
        Err(e) => {
            return Err(Box::new(DiagnosticError(format_parse_error(path.as_str(), &contents, &e))))
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

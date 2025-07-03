use asmodeus_lsp::analysis::utils::{create_diagnostic, position_to_range};
use tower_lsp::lsp_types::*;

pub fn test_uri(filename: &str) -> Url {
    Url::parse(&format!("file:///test_{}.asmod", filename)).unwrap()
}

pub fn test_position(line: u32, character: u32) -> Position {
    Position { line, character }
}

pub fn test_range(start_line: u32, start_char: u32, end_char: u32) -> Range {
    position_to_range(start_line, start_char, end_char)
}

pub fn test_diagnostic(range: Range, severity: DiagnosticSeverity, message: &str) -> Diagnostic {
    create_diagnostic(range, severity, "TEST001", message.to_string())
}

pub const SAMPLE_VALID_PROGRAM: &str = r#"
start:
    POB #42
    DOD #1
    WYJSCIE
    SOB start
end:
    STP
"#;

pub const SAMPLE_INVALID_PROGRAM: &str = r#"
start:
    INVALID_INSTRUCTION #42
    POB
    SOB nonexistent_label
"#;

pub const SAMPLE_EXTENDED_PROGRAM: &str = r#"
start:
    POB #6
    MNO #7
    DZI #2
    WYJSCIE
    STP
"#;

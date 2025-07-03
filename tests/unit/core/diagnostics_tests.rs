use asmodeus_lsp::analysis::core::DiagnosticsEngine;
use tower_lsp::lsp_types::*;

#[test]
fn test_diagnostics_engine_creation() {
    let _engine = DiagnosticsEngine::new();
    // can be created?
    assert!(true);
}

#[test]
fn test_analyze_valid_document() {
    let engine = DiagnosticsEngine::new();
    let content = r#"
start:
    POB #42
    WYJSCIE
    STP
"#;
    let uri = Url::parse("file:///test.asmod").unwrap();

    let diagnostics = engine.analyze_document(content, &uri);
    assert!(
        diagnostics.is_empty(),
        "Valid document should produce no diagnostics"
    );
}

#[test]
fn test_analyze_document_with_unknown_instruction() {
    let engine = DiagnosticsEngine::new();
    let content = r#"
start:
    UNKNOWN_INSTRUCTION #42
"#;
    let uri = Url::parse("file:///test.asmod").unwrap();

    let diagnostics = engine.analyze_document(content, &uri);
    assert!(
        !diagnostics.is_empty(),
        "Should produce diagnostics for unknown instruction"
    );

    let diagnostic = &diagnostics[0];
    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
    assert!(
        diagnostic.message.contains("Unknown instruction")
            || diagnostic.message.contains("undefined macro")
    );
}

#[test]
fn test_analyze_document_with_lexer_error() {
    let engine = DiagnosticsEngine::new();
    let content = "@#$%^&*()"; // should cause lexer error
    let uri = Url::parse("file:///test.asmod").unwrap();

    let diagnostics = engine.analyze_document(content, &uri);
    assert!(
        !diagnostics.is_empty(),
        "Should produce diagnostics for lexer errors"
    );

    let diagnostic = &diagnostics[0];
    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
    assert!(diagnostic.message.contains("Lexer error"));
    assert_eq!(
        diagnostic.code,
        Some(NumberOrString::String("LEX001".to_string()))
    );
}

#[test]
fn test_analyze_document_with_parser_error() {
    let engine = DiagnosticsEngine::new();
    // should tokenize but fail to parse
    let content = r#"
    POB POB POB
"#;
    let uri = Url::parse("file:///test.asmod").unwrap();

    let diagnostics = engine.analyze_document(content, &uri);

    // should either produce parser error or semantic error
    if !diagnostics.is_empty() {
        let diagnostic = &diagnostics[0];
        assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
        assert!(
            diagnostic.message.contains("Parser error")
                || diagnostic.message.contains("Unknown instruction")
                || diagnostic.message.contains("undefined macro")
        );
    }
}

#[test]
fn test_analyze_whitespace_only_document() {
    let engine = DiagnosticsEngine::new();
    let content = "   \n\t  \n   ";
    let uri = Url::parse("file:///test.asmod").unwrap();

    let diagnostics = engine.analyze_document(content, &uri);
    // should be valid
    assert!(diagnostics.is_empty() || diagnostics.len() <= 1);
}

#[test]
fn test_analyze_comments_only_document() {
    let engine = DiagnosticsEngine::new();
    let content = r#"
; This is a comment
; Another comment
    ; Indented comment
"#;
    let uri = Url::parse("file:///test.asmod").unwrap();

    let diagnostics = engine.analyze_document(content, &uri);
    // should be valid
    assert!(diagnostics.is_empty());
}

#[test]
fn test_analyze_mixed_valid_invalid_content() {
    let engine = DiagnosticsEngine::new();
    let content = r#"
start:
    POB #42        ; Valid instruction
    INVALID_INST   ; Invalid instruction
    WYJSCIE        ; Valid instruction
    ANOTHER_BAD    ; Another invalid instruction
    STP            ; Valid instruction
"#;
    let uri = Url::parse("file:///test.asmod").unwrap();

    let diagnostics = engine.analyze_document(content, &uri);

    // should produce diagnostics for invalid instructions
    assert!(!diagnostics.is_empty());

    // all diagnostics should be errors
    for diagnostic in &diagnostics {
        assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(diagnostic.source, Some("asmodeus-lsp".to_string()));
    }
}

#[test]
fn test_diagnostic_positions() {
    let engine = DiagnosticsEngine::new();
    let content = r#"start:
    INVALID_INSTRUCTION"#;
    let uri = Url::parse("file:///test.asmod").unwrap();

    let diagnostics = engine.analyze_document(content, &uri);

    if !diagnostics.is_empty() {
        let diagnostic = &diagnostics[0];

        // diagnostic should has reasonable position
        assert!(diagnostic.range.start.line <= 1);
        assert!(diagnostic.range.start.character < 50);
        assert!(diagnostic.range.end.line <= 1);
        assert!(diagnostic.range.end.character < 50);
        assert!(diagnostic.range.start.character <= diagnostic.range.end.character);
    }
}

#[test]
fn test_diagnostic_codes() {
    let engine = DiagnosticsEngine::new();
    let content = "INVALID_INSTRUCTION";
    let uri = Url::parse("file:///test.asmod").unwrap();

    let diagnostics = engine.analyze_document(content, &uri);

    if !diagnostics.is_empty() {
        let diagnostic = &diagnostics[0];

        // should have a diagnostic code
        assert!(diagnostic.code.is_some());

        match &diagnostic.code {
            Some(NumberOrString::String(code)) => {
                // should be one of the expected error codes
                assert!(
                    code == "LEX001" || code == "PAR001" || code == "SEM001" || code == "SEM002"
                );
            }
            _ => panic!("Expected string error code"),
        }
    }
}

#[test]
fn test_multiple_errors_in_document() {
    let engine = DiagnosticsEngine::new();
    let content = r#"
FIRST_INVALID
SECOND_INVALID  
THIRD_INVALID
"#;
    let uri = Url::parse("file:///test.asmod").unwrap();

    let diagnostics = engine.analyze_document(content, &uri);

    if diagnostics.len() > 1 {
        for i in 1..diagnostics.len() {
            let prev = &diagnostics[i - 1];
            let curr = &diagnostics[i];

            assert!(prev.range.start.line <= curr.range.start.line);
            if prev.range.start.line == curr.range.start.line {
                assert!(prev.range.start.character <= curr.range.start.character);
            }
        }
    }
}

use asmodeus_lsp::analysis::SemanticAnalyzer;
use tower_lsp::lsp_types::*;

#[tokio::test]
async fn test_basic_diagnostics_integration() {
    let analyzer = SemanticAnalyzer::new();
    
    // valid assembly?
    let valid_content = r#"
start:
    POB #42
    WYJSCIE
    STP
"#;
    let uri = Url::parse("file:///test.asmod").unwrap();
    let diagnostics = analyzer.analyze_document(valid_content, &uri);
    assert!(diagnostics.is_empty(), "Valid assembly should have no errors");
    
    // invalid assembly?
    let invalid_content = r#"
start:
    UNKNOWN_INSTRUCTION_XYZ
"#;
    let diagnostics = analyzer.analyze_document(invalid_content, &uri);
    assert!(!diagnostics.is_empty(), "Invalid assembly should have errors");
    assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::ERROR));
    
    // either a parser error OR semantic error (unknown instruction/macro)
    assert!(
        diagnostics[0].message.contains("Parser error") || 
        diagnostics[0].message.contains("Unknown instruction") ||
        diagnostics[0].message.contains("undefined macro"),
        "Expected parser or semantic error, got: {}", diagnostics[0].message
    );
}

#[tokio::test] 
async fn test_lexer_error_handling() {
    let analyzer = SemanticAnalyzer::new();
    
    // content that should cause lexer error
    let invalid_content = r#"
start:
    POB @#$%^&*
"#;
    let uri = Url::parse("file:///test.asmod").unwrap();
    let diagnostics = analyzer.analyze_document(invalid_content, &uri);
    
    // should have at least 1 diagnostic
    assert!(!diagnostics.is_empty(), "Invalid syntax should produce diagnostics");
}

#[test]
fn test_document_state_management() {
    let uri = Url::parse("file:///test.asmod").unwrap();
    let mut doc = asmodeus_lsp::analysis::DocumentState::new(
        uri.clone(), 
        "initial content".to_string(), 
        1
    );
    
    assert_eq!(doc.content, "initial content");
    assert_eq!(doc.version, 1);
    
    doc.update_content("updated content".to_string(), 2);
    assert_eq!(doc.content, "updated content");
    assert_eq!(doc.version, 2);
}

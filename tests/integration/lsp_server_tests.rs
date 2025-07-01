use asmodeus_lsp::analysis::{SemanticAnalyzer, DocumentState};
use tower_lsp::lsp_types::*;

#[tokio::test]
async fn test_basic_diagnostics_integration() {
    let analyzer = SemanticAnalyzer::new();
    
    let valid_content = r#"
start:
    POB #42
    WYJSCIE
    STP
"#;
    let uri = Url::parse("file:///test.asmod").unwrap();
    let diagnostics = analyzer.analyze_document(valid_content, &uri);

    assert!(diagnostics.is_empty(), "Valid assembly should have no errors");
    
    let invalid_content = r#"
start:
    UNKNOWN_INSTRUCTION_XYZ
"#;
    let diagnostics = analyzer.analyze_document(invalid_content, &uri);
    assert!(!diagnostics.is_empty(), "Invalid assembly should have errors");
    assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::ERROR));
}

#[tokio::test] 
async fn test_lexer_error_handling() {
    let analyzer = SemanticAnalyzer::new();
    
    // should cause lexer error
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
    let mut doc = DocumentState::new(
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

#[tokio::test]
async fn test_full_workflow() {
    let analyzer = SemanticAnalyzer::new();
    let content = r#"start:
    POB #42
    DOD #1
    WYJSCIE
    SOB start
end:
    STP"#;
    let uri = Url::parse("file:///test.asmod").unwrap();
    
    // 1. analyze document
    let diagnostics = analyzer.analyze_document(content, &uri);
    assert!(diagnostics.is_empty());
    
    // 2. get completions
    let completions = analyzer.get_completions(content, Position { line: 1, character: 4 });
    assert!(!completions.is_empty());
    
    // 3. get hover info
    let hover = analyzer.get_hover_info(content, Position { line: 1, character: 6 });
    assert!(hover.is_some());
    
    // 4. get definition
    let definition = analyzer.get_definition(content, Position { line: 4, character: 8 }, &uri);
    assert!(definition.is_some());
    
    // 5. find references
    let references = analyzer.find_references(content, Position { line: 0, character: 0 }, &uri, true);
    assert_eq!(references.len(), 2); // definition + reference
    
    // 6. get symbols
    let symbols = analyzer.get_document_symbols(content);
    assert_eq!(symbols.len(), 2); // start + end
    
    // 7. get semantic tokens
    let tokens = analyzer.get_semantic_tokens(content);
    assert!(!tokens.is_empty());
}

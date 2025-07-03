use crate::analysis::language::validation::ValidationProvider; 
use tower_lsp::lsp_types::*;

#[test]
fn test_validate_symbol_usage_valid_labels() {
    let provider = ValidationProvider::new();
    let content = r#"start:
    POB #42
valid_label:
    STP"#;
    
    let diagnostics = provider.validate_symbol_usage(content);
    assert!(diagnostics.is_empty());
}

#[test]
fn test_validate_symbol_usage_invalid_label_name() {
    let provider = ValidationProvider::new();
    let content = r#"123invalid:
    POB #42"#;
    
    let diagnostics = provider.validate_symbol_usage(content);
    assert!(!diagnostics.is_empty());
    
    let diagnostic = &diagnostics[0];
    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
    assert!(diagnostic.message.contains("Invalid label name"));
}

#[test]
fn test_validate_symbol_usage_label_conflicts_with_instruction() {
    let provider = ValidationProvider::new();
    let content = r#"POB:
    DOD #42"#;
    
    let diagnostics = provider.validate_symbol_usage(content);
    assert!(!diagnostics.is_empty());
    
    let diagnostic = &diagnostics[0];
    assert!(diagnostic.message.contains("conflicts with instruction"));
}

#[test]
fn test_validate_symbol_usage_invalid_characters() {
    let provider = ValidationProvider::new();
    let content = "POB @#$%";
    
    let diagnostics = provider.validate_symbol_usage(content);
    assert!(!diagnostics.is_empty());
    
    let diagnostic = &diagnostics[0];
    assert!(diagnostic.message.contains("Invalid character"));
}

#[test]
fn test_validate_symbol_usage_comments_ignored() {
    let provider = ValidationProvider::new();
    let content = "POB #42 ; @#$% this is fine in comments";
    
    let diagnostics = provider.validate_symbol_usage(content);
    assert!(diagnostics.is_empty());
}

#[test]
fn test_validate_symbol_usage_allowed_characters() {
    let provider = ValidationProvider::new();
    let content = r#"start_123:
    POB #42
    ŁAD [100]
    SOB start_123"#;
    
    let diagnostics = provider.validate_symbol_usage(content);
    assert!(diagnostics.is_empty());
}

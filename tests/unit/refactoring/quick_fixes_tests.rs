use asmodeus_lsp::analysis::refactoring::QuickFixProvider;
use tower_lsp::lsp_types::*;

#[test]
fn test_create_quick_fix_unknown_instruction() {
    let provider = QuickFixProvider::new();
    let content = "DOX #42";
    let uri = Url::parse("file:///test.asmod").unwrap();
    
    let diagnostic = Diagnostic {
        range: Range {
            start: Position { line: 0, character: 0 },
            end: Position { line: 0, character: 3 },
        },
        severity: Some(DiagnosticSeverity::ERROR),
        message: "Unknown instruction: 'DOX'".to_string(),
        source: Some("asmodeus-lsp".to_string()),
        ..Default::default()
    };
    
    let action = provider.create_quick_fix(&diagnostic, content, &uri);
    
    assert!(action.is_some());
    match action.unwrap() {
        CodeActionOrCommand::CodeAction(action) => {
            assert_eq!(action.kind, Some(CodeActionKind::QUICKFIX));
            assert!(action.title.contains("Replace with"));
            assert!(action.title.contains("DOD"));
            assert_eq!(action.is_preferred, Some(true));
        },
        _ => panic!("Expected CodeAction"),
    }
}

#[test]
fn test_create_quick_fix_invalid_message_format() {
    let provider = QuickFixProvider::new();
    let content = "DOX #42";
    let uri = Url::parse("file:///test.asmod").unwrap();
    
    let diagnostic = Diagnostic {
        range: Range {
            start: Position { line: 0, character: 0 },
            end: Position { line: 0, character: 3 },
        },
        severity: Some(DiagnosticSeverity::ERROR),
        message: "Unknown instruction DOX".to_string(),
        source: Some("asmodeus-lsp".to_string()),
        ..Default::default()
    };
    
    let action = provider.create_quick_fix(&diagnostic, content, &uri);
    
    assert!(action.is_none()); // cannot extract instruction name without quotes
}

#[test]
fn test_create_quick_fix_no_suggestions() {
    let provider = QuickFixProvider::new();
    let content = "COMPLETELY_UNKNOWN #42";
    let uri = Url::parse("file:///test.asmod").unwrap();
    
    let diagnostic = Diagnostic {
        range: Range {
            start: Position { line: 0, character: 0 },
            end: Position { line: 0, character: 18 },
        },
        severity: Some(DiagnosticSeverity::ERROR),
        message: "Unknown instruction: 'COMPLETELY_UNKNOWN'".to_string(),
        source: Some("asmodeus-lsp".to_string()),
        ..Default::default()
    };
    
    let action = provider.create_quick_fix(&diagnostic, content, &uri);
    
    assert!(action.is_none()); // no similar instructions found
}

#[test]
fn test_create_quick_fix_unrelated_diagnostic() {
    let provider = QuickFixProvider::new();
    let content = "POB #42";
    let uri = Url::parse("file:///test.asmod").unwrap();
    
    let diagnostic = Diagnostic {
        range: Range {
            start: Position { line: 0, character: 0 },
            end: Position { line: 0, character: 3 },
        },
        severity: Some(DiagnosticSeverity::WARNING),
        message: "Some other warning".to_string(),
        source: Some("asmodeus-lsp".to_string()),
        ..Default::default()
    };
    
    let action = provider.create_quick_fix(&diagnostic, content, &uri);
    
    assert!(action.is_none()); // no quick fix for unrelated diagnostics
}

#[test]
fn test_workspace_edit_structure() {
    let provider = QuickFixProvider::new();
    let content = "DOX #42";
    let uri = Url::parse("file:///test.asmod").unwrap();
    
    let diagnostic = Diagnostic {
        range: Range {
            start: Position { line: 0, character: 0 },
            end: Position { line: 0, character: 3 },
        },
        severity: Some(DiagnosticSeverity::ERROR),
        message: "Unknown instruction: 'DOX'".to_string(),
        source: Some("asmodeus-lsp".to_string()),
        ..Default::default()
    };
    
    let action = provider.create_quick_fix(&diagnostic, content, &uri).unwrap();
    
    match action {
        CodeActionOrCommand::CodeAction(action) => {
            let edit = action.edit.unwrap();
            let changes = edit.changes.unwrap();
            
            assert!(changes.contains_key(&uri));
            let file_edits = changes.get(&uri).unwrap();
            assert_eq!(file_edits.len(), 1);
            
            let text_edit = &file_edits[0];
            assert_eq!(text_edit.range, diagnostic.range);
            assert_eq!(text_edit.new_text, "DOD");
        },
        _ => panic!("Expected CodeAction"),
    }
}

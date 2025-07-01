use asmodeus_lsp::analysis::refactoring::CodeActionsProvider;
use tower_lsp::lsp_types::*;

#[test]
fn test_get_code_actions_with_diagnostics() {
    let provider = CodeActionsProvider::new();
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
    
    let context = CodeActionContext {
        diagnostics: vec![diagnostic],
        only: None,
        trigger_kind: None,
    };
    
    let range = Range {
        start: Position { line: 0, character: 0 },
        end: Position { line: 0, character: 7 },
    };
    
    let actions = provider.get_code_actions(content, range, &uri, &context);
    
    assert!(!actions.is_empty());
    
    // should have quick fixes and refactoring actions
    let has_quick_fix = actions.iter().any(|action| {
        match action {
            CodeActionOrCommand::CodeAction(action) => {
                action.kind == Some(CodeActionKind::QUICKFIX)
            },
            _ => false,
        }
    });
    
    let has_refactor = actions.iter().any(|action| {
        match action {
            CodeActionOrCommand::CodeAction(action) => {
                action.kind == Some(CodeActionKind::REFACTOR)
            },
            _ => false,
        }
    });
    
    assert!(has_quick_fix);
    assert!(has_refactor);
}

#[test]
fn test_get_code_actions_no_diagnostics() {
    let provider = CodeActionsProvider::new();
    let content = "POB #42";
    let uri = Url::parse("file:///test.asmod").unwrap();
    
    let context = CodeActionContext {
        diagnostics: vec![],
        only: None,
        trigger_kind: None,
    };
    
    let range = Range {
        start: Position { line: 0, character: 0 },
        end: Position { line: 0, character: 7 },
    };
    
    let actions = provider.get_code_actions(content, range, &uri, &context);
    
    // should only have refactoring actions, no quick fixes
    let has_quick_fix = actions.iter().any(|action| {
        match action {
            CodeActionOrCommand::CodeAction(action) => {
                action.kind == Some(CodeActionKind::QUICKFIX)
            },
            _ => false,
        }
    });
    
    let has_refactor = actions.iter().any(|action| {
        match action {
            CodeActionOrCommand::CodeAction(action) => {
                action.kind == Some(CodeActionKind::REFACTOR)
            },
            _ => false,
        }
    });
    
    assert!(!has_quick_fix);
    assert!(has_refactor);
}

#[test]
fn test_get_code_actions_empty_content() {
    let provider = CodeActionsProvider::new();
    let content = "";
    let uri = Url::parse("file:///test.asmod").unwrap();
    
    let context = CodeActionContext {
        diagnostics: vec![],
        only: None,
        trigger_kind: None,
    };
    
    let range = Range {
        start: Position { line: 0, character: 0 },
        end: Position { line: 0, character: 0 },
    };
    
    let actions = provider.get_code_actions(content, range, &uri, &context);
    
    assert!(actions.is_empty());
}

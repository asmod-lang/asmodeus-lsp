use asmodeus_lsp::analysis::SemanticAnalyzer;
use tower_lsp::lsp_types::*;

#[tokio::test]
async fn test_complete_asmodeus_program() {
    let analyzer = SemanticAnalyzer::new();
    let content = r#"
; Fibonacci sequence calculator

main:
    POB #0          ; Load first number
    ŁAD #100        ; Store as fib_a address
    POB #1          ; Load second number  
    ŁAD #101        ; Store as fib_b address
    POB #10         ; Load counter
    ŁAD #102        ; Store counter address

loop:
    POB #102        ; Load counter
    SOZ end         ; If zero, end program
    
    POB #100        ; Load fib_a
    DOD #101        ; Add fib_b
    ŁAD #103        ; Store temp
    
    POB #101        ; Load fib_b
    ŁAD #100        ; Store as new fib_a
    
    POB #103        ; Load temp
    ŁAD #101        ; Store as new fib_b
    WYJSCIE         ; Output result
    
    POB #102        ; Load counter
    ODE #1          ; Subtract 1
    ŁAD #102        ; Store counter
    
    SOB loop        ; Jump back to loop

end:
    STP             ; Stop program
"#;

    let uri = Url::parse("file:///fibonacci.asmod").unwrap();

    // diagnostics
    let diagnostics = analyzer.analyze_document(content, &uri);
    assert!(
        diagnostics.is_empty(),
        "Complete program should have no errors"
    );

    // symbol discovery
    let symbols = analyzer.get_document_symbols(content);
    let symbol_names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();

    assert!(!symbols.is_empty(), "Should find at least one symbol");
    assert!(symbol_names.contains(&"main"));
    assert!(symbol_names.contains(&"loop"));
    assert!(symbol_names.contains(&"end"));

    // references
    let main_references = analyzer.find_references(
        content,
        Position {
            line: 3,
            character: 0,
        },
        &uri,
        true,
    );
    assert_eq!(main_references.len(), 1); // only definition

    let loop_references = analyzer.find_references(
        content,
        Position {
            line: 11,
            character: 0,
        },
        &uri,
        true,
    );
    assert_eq!(loop_references.len(), 3); // definition + reference from SOB loop

    // hover on different elements
    let hover_instruction = analyzer.get_hover_info(
        content,
        Position {
            line: 6,
            character: 4,
        },
    );
    assert!(hover_instruction.is_some());

    let hover_label = analyzer.get_hover_info(
        content,
        Position {
            line: 11,
            character: 0,
        },
    );
    assert!(hover_label.is_some());

    // semantic tokens for syntax highlighting
    let tokens = analyzer.get_semantic_tokens(content);
    assert!(!tokens.is_empty());

    // code actions
    let context = CodeActionContext {
        diagnostics: vec![],
        only: None,
        trigger_kind: None,
    };
    let range = Range {
        start: Position {
            line: 6,
            character: 0,
        },
        end: Position {
            line: 6,
            character: 15,
        },
    };
    let actions = analyzer.get_code_actions(content, range, &uri, &context);
    assert!(!actions.is_empty());
}

#[tokio::test]
async fn test_error_recovery_and_suggestions() {
    let analyzer = SemanticAnalyzer::new();
    let content = r#"
start:
    pob #42         ; lowercase instruction
    DOX #1          ; typo in instruction
    INVALID_MACRO   ; unknown macro
    SOB start       ; valid jump
"#;
    let uri = Url::parse("file:///errors.asmod").unwrap();

    // should detect errors but not crash
    let diagnostics = analyzer.analyze_document(content, &uri);
    assert!(!diagnostics.is_empty(), "Should detect errors");

    // other features still work (should) despite errors
    let symbols = analyzer.get_document_symbols(content);
    assert!(!symbols.is_empty()); // should still find 'start' label

    let completions = analyzer.get_completions(
        content,
        Position {
            line: 1,
            character: 4,
        },
    );
    assert!(!completions.is_empty()); // should still provide completions

    // code actions for error correction
    let diagnostic = Diagnostic {
        range: Range {
            start: Position {
                line: 2,
                character: 4,
            },
            end: Position {
                line: 2,
                character: 7,
            },
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
        start: Position {
            line: 2,
            character: 0,
        },
        end: Position {
            line: 2,
            character: 11,
        },
    };

    let actions = analyzer.get_code_actions(content, range, &uri, &context);

    // should suggest corrections
    let has_suggestion = actions.iter().any(|action| match action {
        CodeActionOrCommand::CodeAction(action) => {
            action.kind == Some(CodeActionKind::QUICKFIX) && action.title.contains("Replace with")
        }
        _ => false,
    });
    assert!(has_suggestion);
}

#[tokio::test]
async fn test_extended_instructions_workflow() {
    let analyzer = SemanticAnalyzer::new();
    let content = r#"
start:
    POB #6
    MNO #7          ; Extended multiplication
    DZI #2          ; Extended division  
    MOD #5          ; Extended modulo
    WYJSCIE
    STP
"#;

    // completions include extended instructions
    let completions = analyzer.get_completions(
        "",
        Position {
            line: 0,
            character: 0,
        },
    );
    let has_extended = completions
        .iter()
        .any(|c| c.label.contains("MNO") && c.label.contains("Extended"));
    assert!(has_extended);

    // hover on extended instructions
    let hover = analyzer.get_hover_info(
        content,
        Position {
            line: 3,
            character: 4,
        },
    );
    assert!(hover.is_some());

    match hover.unwrap().contents {
        HoverContents::Markup(markup) => {
            assert!(markup.value.contains("Extended"));
            assert!(markup.value.contains("--extended"));
        }
        _ => panic!("Expected markup content"),
    }

    // signature help for extended instructions
    let signature = analyzer.get_signature_help(
        "MNO ",
        Position {
            line: 0,
            character: 4,
        },
    );
    assert!(signature.is_some());

    let sig_info = &signature.unwrap().signatures[0];
    match &sig_info.documentation {
        Some(Documentation::String(doc)) => {
            assert!(doc.contains("Extended"));
        }
        _ => panic!("Expected documentation"),
    }
}

#[tokio::test]
async fn test_rename_across_complex_program() {
    let analyzer = SemanticAnalyzer::new();
    let content = r#"main:
    POB #42
    SOB calculate
calculate:
    DOD #5
    WYJSCIE
    RET
end:
    SOB calculate
    STP"#;

    let uri = Url::parse("file:///complex.asmod").unwrap();

    // rename operation
    let edit = analyzer.rename_symbol(
        content,
        Position {
            line: 3,
            character: 0,
        },
        "compute",
        &uri,
    );
    assert!(edit.is_some());

    let edit = edit.unwrap();
    let changes = edit.changes.unwrap();
    let file_edits = changes.get(&uri).unwrap();

    assert_eq!(file_edits.len(), 3);

    // definition properly renamed with colon
    let def_edit = file_edits
        .iter()
        .find(|e| e.new_text == "compute:")
        .unwrap();
    assert_eq!(def_edit.range.start.line, 3);

    // references are renamed without colon
    let ref_edits: Vec<_> = file_edits
        .iter()
        .filter(|e| e.new_text == "compute")
        .collect();
    assert_eq!(ref_edits.len(), 2);
}

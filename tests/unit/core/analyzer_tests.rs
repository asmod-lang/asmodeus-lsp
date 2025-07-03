use asmodeus_lsp::analysis::utils::position_to_range;
use asmodeus_lsp::analysis::SemanticAnalyzer;
use tower_lsp::lsp_types::*;

#[test]
fn test_semantic_analyzer_creation() {
    let _analyzer = SemanticAnalyzer::new();
    // can be created?
    assert!(true);
}

#[test]
fn test_analyzer_completions_delegation() {
    let analyzer = SemanticAnalyzer::new();
    let completions = analyzer.get_completions(
        "",
        Position {
            line: 0,
            character: 0,
        },
    );

    assert!(!completions.is_empty());

    // should contain instruction completions
    let instruction_names: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();

    assert!(instruction_names.iter().any(|&name| name.contains("POB")));
}

#[test]
fn test_analyzer_hover_delegation() {
    let analyzer = SemanticAnalyzer::new();
    let hover = analyzer.get_hover_info(
        "POB #42",
        Position {
            line: 0,
            character: 1,
        },
    );

    assert!(hover.is_some());
}

#[test]
fn test_analyzer_definition_delegation() {
    let analyzer = SemanticAnalyzer::new();
    let content = "start:\n    SOB start";
    let uri = Url::parse("file:///test.asmod").unwrap();

    let definition = analyzer.get_definition(
        content,
        Position {
            line: 1,
            character: 8,
        },
        &uri,
    );
    assert!(definition.is_some());
}

#[test]
fn test_analyzer_references_delegation() {
    let analyzer = SemanticAnalyzer::new();
    let content = "start:\n    SOB start";
    let uri = Url::parse("file:///test.asmod").unwrap();

    let references = analyzer.find_references(
        content,
        Position {
            line: 0,
            character: 0,
        },
        &uri,
        true,
    );
    assert_eq!(references.len(), 2); // definition + reference
}

#[test]
fn test_analyzer_symbols_delegation() {
    let analyzer = SemanticAnalyzer::new();
    let content = "start:\n    POB #42\nloop:\n    STP";

    let symbols = analyzer.get_document_symbols(content);
    assert_eq!(symbols.len(), 2);
}

#[test]
fn test_analyzer_semantic_tokens_delegation() {
    let analyzer = SemanticAnalyzer::new();
    let content = "POB #42 ; comment";

    let tokens = analyzer.get_semantic_tokens(content);
    assert!(!tokens.is_empty());
}

#[test]
fn test_analyzer_signature_help_delegation() {
    let analyzer = SemanticAnalyzer::new();
    let content = "POB ";

    let signature = analyzer.get_signature_help(
        content,
        Position {
            line: 0,
            character: 4,
        },
    );
    assert!(signature.is_some());
}

#[test]
fn test_analyzer_code_actions_delegation() {
    let analyzer = SemanticAnalyzer::new();
    let content = "POB #42";
    let uri = Url::parse("file:///test.asmod").unwrap();

    let context = CodeActionContext {
        diagnostics: vec![],
        only: None,
        trigger_kind: None,
    };

    let range = position_to_range(0, 0, 7); // Używamy funkcji utility

    let actions = analyzer.get_code_actions(content, range, &uri, &context);
    assert!(!actions.is_empty());
}

#[test]
fn test_analyzer_rename_delegation() {
    let analyzer = SemanticAnalyzer::new();
    let content = "start:\n    SOB start";
    let uri = Url::parse("file:///test.asmod").unwrap();

    let edit = analyzer.rename_symbol(
        content,
        Position {
            line: 0,
            character: 2,
        },
        "begin",
        &uri,
    );
    assert!(edit.is_some());
}


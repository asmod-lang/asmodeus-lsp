use asmodeus_lsp::analysis::language::NavigationProvider;
use tower_lsp::lsp_types::*;

#[test]
fn test_goto_definition_for_label() {
    let provider = NavigationProvider::new();
    let content = r#"start:
    POB #42
    SOB start"#;
    let uri = Url::parse("file:///test.asmod").unwrap();
    
    let definition = provider.get_definition(content, Position { line: 2, character: 8 }, &uri);
    
    assert!(definition.is_some());
    match definition.unwrap() {
        GotoDefinitionResponse::Scalar(location) => {
            assert_eq!(location.uri, uri);
            assert_eq!(location.range.start.line, 0);
            assert_eq!(location.range.start.character, 0);
        },
        _ => panic!("Expected scalar location"),
    }
}

#[test]
fn test_goto_definition_for_nonexistent_label() {
    let provider = NavigationProvider::new();
    let content = "POB #42";
    let uri = Url::parse("file:///test.asmod").unwrap();
    
    let definition = provider.get_definition(content, Position { line: 0, character: 1 }, &uri);
    
    assert!(definition.is_none());
}

#[test]
fn test_find_references_with_declaration() {
    let provider = NavigationProvider::new();
    let content = r#"start:
    POB #42
    SOB start
loop:
    DOD start
    SOB loop"#;
    let uri = Url::parse("file:///test.asmod").unwrap();
    
    let references = provider.find_references(content, Position { line: 0, character: 0 }, &uri, true);
    
    assert_eq!(references.len(), 3); // definition + 2 references
    
    // all references should have valid URI
    for reference in &references {
        assert_eq!(reference.uri, uri);
    }
}

#[test]
fn test_find_references_without_declaration() {
    let provider = NavigationProvider::new();
    let content = r#"start:
    POB #42
    SOB start
    DOD start"#;
    let uri = Url::parse("file:///test.asmod").unwrap();
    
    let references = provider.find_references(content, Position { line: 0, character: 0 }, &uri, false);
    
    assert_eq!(references.len(), 2); // tylko references, bez definition
}

#[test]
fn test_find_references_for_nonexistent_symbol() {
    let provider = NavigationProvider::new();
    let content = "POB #42";
    let uri = Url::parse("file:///test.asmod").unwrap();
    
    // outside line range
    let references = provider.find_references(content, Position { line: 5, character: 0 }, &uri, true);
    
    assert!(references.is_empty());
}

#[test]
fn test_find_references_invalid_position() {
    let provider = NavigationProvider::new();
    let content = "POB #42";
    let uri = Url::parse("file:///test.asmod").unwrap();
    
    let references = provider.find_references(content, Position { line: 5, character: 0 }, &uri, true);
    
    assert!(references.is_empty());
}

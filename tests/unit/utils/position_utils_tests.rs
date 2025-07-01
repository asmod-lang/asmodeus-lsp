use asmodeus_lsp::analysis::utils::*;
use tower_lsp::lsp_types::*;

#[test]
fn test_position_to_range() {
    let range = position_to_range(5, 10, 15);
    assert_eq!(range.start.line, 5);
    assert_eq!(range.start.character, 10);
    assert_eq!(range.end.line, 5);
    assert_eq!(range.end.character, 15);
}

#[test]
fn test_word_range() {
    let range = word_range(2, 5, 10);
    assert_eq!(range.start.line, 2);
    assert_eq!(range.start.character, 5);
    assert_eq!(range.end.line, 2);
    assert_eq!(range.end.character, 10);
}

#[test]
fn test_is_valid_position() {
    let content = "line1\nline2\nline3";
    
    assert!(is_valid_position(content, Position { line: 0, character: 0 }));
    assert!(is_valid_position(content, Position { line: 1, character: 3 }));
    assert!(is_valid_position(content, Position { line: 2, character: 5 }));
    
    assert!(!is_valid_position(content, Position { line: 3, character: 0 })); // beyond lines
    assert!(!is_valid_position(content, Position { line: 0, character: 10 })); // beyond line length
}

#[test]
fn test_get_line_at_position() {
    let content = "line1\nline2\nline3";
    
    assert_eq!(get_line_at_position(content, Position { line: 0, character: 0 }).unwrap(), "line1");
    assert_eq!(get_line_at_position(content, Position { line: 1, character: 0 }).unwrap(), "line2");
    assert_eq!(get_line_at_position(content, Position { line: 2, character: 0 }).unwrap(), "line3");
    
    assert!(get_line_at_position(content, Position { line: 3, character: 0 }).is_none());
}

#[test]
fn test_create_location() {
    let uri = Url::parse("file:///test.asmod").unwrap();
    let range = Range {
        start: Position { line: 0, character: 0 },
        end: Position { line: 0, character: 5 },
    };
    
    let location = create_location(&uri, range);
    assert_eq!(location.uri, uri);
    assert_eq!(location.range, range);
}

#[test]
fn test_create_word_location() {
    let uri = Url::parse("file:///test.asmod").unwrap();
    let location = create_word_location(&uri, 2, 10, 5);
    
    assert_eq!(location.uri, uri);
    assert_eq!(location.range.start.line, 2);
    assert_eq!(location.range.start.character, 10);
    assert_eq!(location.range.end.line, 2);
    assert_eq!(location.range.end.character, 15);
}

#[test]
fn test_create_diagnostic() {
    let range = Range {
        start: Position { line: 0, character: 0 },
        end: Position { line: 0, character: 5 },
    };
    
    let diagnostic = create_diagnostic(
        range,
        DiagnosticSeverity::ERROR,
        "TEST001",
        "Test message".to_string(),
    );
    
    assert_eq!(diagnostic.range, range);
    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
    assert_eq!(diagnostic.code, Some(NumberOrString::String("TEST001".to_string())));
    assert_eq!(diagnostic.message, "Test message");
    assert_eq!(diagnostic.source, Some("asmodeus-lsp".to_string()));
}

#[test]
fn test_create_semantic_diagnostic() {
    let diagnostic = create_semantic_diagnostic(
        5, 10, 3, "SEM001", "Semantic error".to_string()
    );
    
    assert_eq!(diagnostic.range.start.line, 4); // line - 1
    assert_eq!(diagnostic.range.start.character, 9); // column - 1
    assert_eq!(diagnostic.range.end.line, 4);
    assert_eq!(diagnostic.range.end.character, 13); // column + length
    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
}

#[test]
fn test_is_label_definition_location() {
    let content = r#"start:
    POB #42
    SOB start"#;
    
    let uri = Url::parse("file:///test.asmod").unwrap();
    
    // on label definition 
    let def_location = Location {
        uri: uri.clone(),
        range: Range {
            start: Position { line: 0, character: 0 },
            end: Position { line: 0, character: 5 },
        },
    };
    assert!(is_label_definition_location(content, &def_location));
    
    // on label reference 
    let ref_location = Location {
        uri,
        range: Range {
            start: Position { line: 2, character: 8 },
            end: Position { line: 2, character: 13 },
        },
    };
    assert!(!is_label_definition_location(content, &ref_location));
}

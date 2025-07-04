use asmodeus_lsp::analysis::language::SymbolProvider;
use tower_lsp::lsp_types::*;

#[test]
fn test_get_document_symbols() {
    let provider = SymbolProvider::new();
    let content = r#"start:
    POB #42
loop:
    SOB start
end:
    STP"#;

    let symbols = provider.get_document_symbols(content);

    assert_eq!(symbols.len(), 3);

    let symbol_names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
    assert!(symbol_names.contains(&"start"));
    assert!(symbol_names.contains(&"loop"));
    assert!(symbol_names.contains(&"end"));

    // whether all symbols are labels
    for symbol in &symbols {
        assert_eq!(symbol.kind, SymbolKind::FUNCTION);
    }
}

#[test]
fn test_symbols_with_indentation() {
    let provider = SymbolProvider::new();
    let content = r#"  start:
    POB #42
    loop:
        SOB start"#;

    let symbols = provider.get_document_symbols(content);

    assert_eq!(symbols.len(), 2);

    let start_symbol = symbols.iter().find(|s| s.name == "start").unwrap();
    assert_eq!(start_symbol.location.range.start.character, 2); // leading whitespace

    let loop_symbol = symbols.iter().find(|s| s.name == "loop").unwrap();
    assert_eq!(loop_symbol.location.range.start.character, 4);
}

#[test]
fn test_invalid_label_names_ignored() {
    let provider = SymbolProvider::new();
    let content = r#"123invalid:
    POB #42
valid_label:
    STP
:
    DOD #1"#;

    let symbols = provider.get_document_symbols(content);

    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].name, "valid_label");
}

#[test]
#[allow(deprecated)]
fn test_filter_workspace_symbols() {
    let provider = SymbolProvider::new();
    let uri = Url::parse("file:///test.asmod").unwrap();

    let mut symbols = vec![
        SymbolInformation {
            name: "start".to_string(),
            kind: SymbolKind::FUNCTION,
            tags: None,
            deprecated: None,
            location: Location {
                uri: Url::parse("file:///dummy").unwrap(),
                range: Range::default(),
            },
            container_name: None,
        },
        SymbolInformation {
            name: "loop".to_string(),
            kind: SymbolKind::FUNCTION,
            tags: None,
            deprecated: None,
            location: Location {
                uri: Url::parse("file:///dummy").unwrap(),
                range: Range::default(),
            },
            container_name: None,
        },
    ];

    provider.filter_workspace_symbols(&mut symbols, "sta", &uri);

    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].name, "start");
    assert_eq!(symbols[0].location.uri, uri);
}

use asmodeus_lsp::analysis::language::HoverProvider;
use tower_lsp::lsp_types::*;

#[test]
fn test_hover_on_instruction() {
    let provider = HoverProvider::new();
    let content = "POB #42";
    let hover = provider.get_hover_info(content, Position { line: 0, character: 1 });
    
    assert!(hover.is_some());
    let hover = hover.unwrap();
    
    match hover.contents {
        HoverContents::Markup(markup) => {
            assert_eq!(markup.kind, MarkupKind::Markdown);
            assert!(markup.value.contains("POB"));
            assert!(markup.value.contains("Load"));
            assert!(markup.value.contains("Memory"));
        },
        _ => panic!("Expected markup content"),
    }
    
    assert!(hover.range.is_some());
}

#[test]
fn test_hover_on_label_definition() {
    let provider = HoverProvider::new();
    let content = r#"start:
    POB #42
    SOB start"#;
    
    let hover = provider.get_hover_info(content, Position { line: 0, character: 2 });
    
    assert!(hover.is_some());
    let hover = hover.unwrap();
    
    match hover.contents {
        HoverContents::Markup(markup) => {
            assert!(markup.value.contains("start"));
            assert!(markup.value.contains("Label"));
            assert!(markup.value.contains("Line 1"));
        },
        _ => panic!("Expected markup content"),
    }
}

#[test]
fn test_hover_on_label_reference() {
    let provider = HoverProvider::new();
    let content = r#"start:
    POB #42
    SOB start"#;
    
    let hover = provider.get_hover_info(content, Position { line: 2, character: 8 });
    
    assert!(hover.is_some());
    let hover = hover.unwrap();
    
    match hover.contents {
        HoverContents::Markup(markup) => {
            assert!(markup.value.contains("start"));
            assert!(markup.value.contains("Label"));
        },
        _ => panic!("Expected markup content"),
    }
}

#[test]
fn test_hover_on_extended_instruction() {
    let provider = HoverProvider::new();
    let content = "MNO #5";
    let hover = provider.get_hover_info(content, Position { line: 0, character: 1 });
    
    assert!(hover.is_some());
    let hover = hover.unwrap();
    
    match hover.contents {
        HoverContents::Markup(markup) => {
            assert!(markup.value.contains("MNO"));
            assert!(markup.value.contains("Extended"));
            assert!(markup.value.contains("--extended"));
        },
        _ => panic!("Expected markup content"),
    }
}

#[test]
fn test_hover_on_whitespace_returns_none() {
    let provider = HoverProvider::new();
    let content = "POB #42";
    let hover = provider.get_hover_info(content, Position { line: 0, character: 3 });
    
    assert!(hover.is_none());
}

#[test]
fn test_hover_beyond_content_returns_none() {
    let provider = HoverProvider::new();
    let content = "POB #42";
    let hover = provider.get_hover_info(content, Position { line: 1, character: 0 });
    
    assert!(hover.is_none());
}

#[test]
fn test_hover_on_unknown_word_returns_none() {
    let provider = HoverProvider::new();
    let content = "unknown_word";
    let hover = provider.get_hover_info(content, Position { line: 0, character: 5 });
    
    assert!(hover.is_none());
}

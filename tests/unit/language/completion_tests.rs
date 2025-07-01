use asmodeus_lsp::analysis::language::CompletionProvider;
use tower_lsp::lsp_types::*;

#[test]
fn test_empty_content_returns_instructions() {
    let provider = CompletionProvider::new();
    let completions = provider.get_completions("", Position { line: 0, character: 0 });
    
    assert!(!completions.is_empty());
    
    let instruction_names: Vec<&str> = completions.iter()
        .map(|c| c.label.as_str())
        .collect();
    
    assert!(instruction_names.iter().any(|&name| name.contains("POB")));
    assert!(instruction_names.iter().any(|&name| name.contains("DOD")));
    assert!(instruction_names.iter().any(|&name| name.contains("STP")));
}

#[test]
fn test_instruction_context() {
    let provider = CompletionProvider::new();
    let content = "start:\n    ";
    let completions = provider.get_completions(content, Position { line: 1, character: 4 });
    
    assert!(!completions.is_empty());
    
    // should return instruction
    let has_instructions = completions.iter()
        .any(|c| c.kind == Some(CompletionItemKind::KEYWORD));
    assert!(has_instructions);
}

#[test]
fn test_operand_context() {
    let provider = CompletionProvider::new();
    let content = "POB ";
    let completions = provider.get_completions(content, Position { line: 0, character: 4 });
    
    // should return operands
    let has_immediate = completions.iter()
        .any(|c| c.label.contains("immediate"));
    assert!(has_immediate);
}

#[test]
fn test_completion_snippets() {
    let provider = CompletionProvider::new();
    let completions = provider.get_completions("", Position { line: 0, character: 0 });
    
    // instructions should have snippets
    let pob_completion = completions.iter()
        .find(|c| c.label.contains("POB"))
        .unwrap();
    
    assert!(pob_completion.insert_text.is_some());
    assert_eq!(pob_completion.insert_text_format, Some(InsertTextFormat::SNIPPET));
}

#[test]
fn test_extended_instructions_marked() {
    let provider = CompletionProvider::new();
    let completions = provider.get_completions("", Position { line: 0, character: 0 });
    
    let extended_completion = completions.iter()
        .find(|c| c.label.contains("MNO"))
        .unwrap();
    
    assert!(extended_completion.label.contains("Extended"));
}

#[test]
fn test_completion_sorting() {
    let provider = CompletionProvider::new();
    let completions = provider.get_completions("", Position { line: 0, character: 0 });
    
    // standard instructions should have higher priority than extended ones
    let standard_completion = completions.iter()
        .find(|c| c.label == "POB")
        .unwrap();
    
    let extended_completion = completions.iter()
        .find(|c| c.label.contains("MNO"))
        .unwrap();
    
    assert!(standard_completion.sort_text < extended_completion.sort_text);
}

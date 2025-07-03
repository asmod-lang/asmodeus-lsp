use asmodeus_lsp::analysis::language::SemanticTokensProvider;

#[test]
fn test_basic_semantic_tokens() {
    let provider = SemanticTokensProvider::new();
    let content = r#"start:
    POB #42    ; load immediate
    SOB start  ; jump to start"#;

    let tokens = provider.get_semantic_tokens(content);

    assert!(!tokens.is_empty());

    // should be in delta encoding format
    let mut current_line = 0;

    for token in &tokens {
        current_line += token.delta_line;

        // positions should be valid
        assert!(current_line < 3);
        assert!(token.length > 0);
    }
}

#[test]
fn test_instruction_tokens() {
    let provider = SemanticTokensProvider::new();
    let content = "POB #42";

    let tokens = provider.get_semantic_tokens(content);

    // should have tokens for: POB, #, 42
    assert!(tokens.len() >= 3);

    // first token should be instruction (KEYWORD = 0)
    assert_eq!(tokens[0].token_type, 0);
}

#[test]
fn test_comment_tokens() {
    let provider = SemanticTokensProvider::new();
    let content = "POB #42 ; this is a comment";

    let tokens = provider.get_semantic_tokens(content);

    // should include comment token (COMMENT = 4)
    let has_comment = tokens.iter().any(|t| t.token_type == 4);
    assert!(has_comment);
}

#[test]
fn test_label_tokens() {
    let provider = SemanticTokensProvider::new();
    let content = r#"start:
    SOB start"#;

    let tokens = provider.get_semantic_tokens(content);

    // should include label token (FUNCTION = 1)
    let label_tokens: Vec<_> = tokens.iter().filter(|t| t.token_type == 1).collect();
    assert_eq!(label_tokens.len(), 2); // definicja + referencja
}

#[test]
fn test_operator_tokens() {
    let provider = SemanticTokensProvider::new();
    let content = "POB #42 [100]";

    let tokens = provider.get_semantic_tokens(content);

    // should include operands token (OPERATOR = 3)
    let operator_tokens: Vec<_> = tokens.iter().filter(|t| t.token_type == 3).collect();
    assert!(operator_tokens.len() >= 3); // #, [, ]
}

#[test]
fn test_number_tokens() {
    let provider = SemanticTokensProvider::new();
    let content = "POB #42 0xFF 0b1010";

    let tokens = provider.get_semantic_tokens(content);

    // should include number token (NUMBER = 2)
    let number_tokens: Vec<_> = tokens.iter().filter(|t| t.token_type == 2).collect();
    assert!(number_tokens.len() >= 3); // 42, 0xFF, 0b1010
}

#[test]
fn test_empty_content() {
    let provider = SemanticTokensProvider::new();
    let tokens = provider.get_semantic_tokens("");

    assert!(tokens.is_empty());
}

#[test]
fn test_whitespace_only() {
    let provider = SemanticTokensProvider::new();
    let tokens = provider.get_semantic_tokens("   \n\t  \n   ");

    assert!(tokens.is_empty());
}

use asmodeus_lsp::analysis::refactoring::SuggestionProvider;

#[test]
fn test_find_similar_instructions_basic() {
    let provider = SuggestionProvider::new();
    
    let suggestions = provider.find_similar_instructions("DOX");
    assert!(suggestions.contains(&"DOD".to_string()));
    
    let suggestions = provider.find_similar_instructions("PO");
    assert!(suggestions.contains(&"POB".to_string()));
    
    let suggestions = provider.find_similar_instructions("ST");
    assert!(suggestions.contains(&"STP".to_string()));
}

#[test]
fn test_find_similar_instructions_case_insensitive() {
    let provider = SuggestionProvider::new();
    
    let suggestions = provider.find_similar_instructions("pob");
    assert!(suggestions.contains(&"POB".to_string()));
    
    let suggestions = provider.find_similar_instructions("dod");
    assert!(suggestions.contains(&"DOD".to_string()));
}

#[test]
fn test_find_similar_instructions_no_matches() {
    let provider = SuggestionProvider::new();
    
    let suggestions = provider.find_similar_instructions("COMPLETELY_DIFFERENT");
    assert!(suggestions.is_empty());
}

#[test]
fn test_suggest_common_fixes() {
    let provider = SuggestionProvider::new();
    
    let suggestions = provider.suggest_common_fixes("DOX");
    assert!(!suggestions.is_empty());
    assert!(suggestions.contains(&"DOD".to_string()));
    
    let suggestions = provider.suggest_common_fixes("pob");
    assert!(suggestions.contains(&"POB".to_string()));
}

#[test]
fn test_suggest_common_typos() {
    let provider = SuggestionProvider::new();
    
    let suggestions = provider.suggest_common_fixes("pob");
    assert!(suggestions.contains(&"POB".to_string()));
    
    let suggestions = provider.suggest_common_fixes("lad");
    assert!(suggestions.contains(&"ŁAD".to_string()));
    
    let suggestions = provider.suggest_common_fixes("wejscie");
    assert!(suggestions.contains(&"WEJSCIE".to_string()));
}

#[test]
fn test_suggest_l_to_l_kreskowane() {
    let provider = SuggestionProvider::new();
    
    let suggestions = provider.suggest_common_fixes("LAD");
    assert!(suggestions.contains(&"ŁAD".to_string()));
}

#[test]
fn test_suggest_alternative_instructions() {
    let provider = SuggestionProvider::new();
    
    let suggestions = provider.suggest_alternative_instructions("add");
    assert!(suggestions.contains(&"DOD".to_string()));
    
    let suggestions = provider.suggest_alternative_instructions("subtract");
    assert!(suggestions.contains(&"ODE".to_string()));
    
    let suggestions = provider.suggest_alternative_instructions("jump");
    assert!(suggestions.contains(&"SOB".to_string()));
    assert!(suggestions.contains(&"SOM".to_string()));
    assert!(suggestions.contains(&"SOZ".to_string()));
    
    let suggestions = provider.suggest_alternative_instructions("load");
    assert!(suggestions.contains(&"POB".to_string()));
    
    let suggestions = provider.suggest_alternative_instructions("store");
    assert!(suggestions.contains(&"ŁAD".to_string()));
}

#[test]
fn test_suggest_alternative_instructions_case_insensitive() {
    let provider = SuggestionProvider::new();
    
    let suggestions = provider.suggest_alternative_instructions("ADD");
    assert!(suggestions.contains(&"DOD".to_string()));
    
    let suggestions = provider.suggest_alternative_instructions("Jump");
    assert!(suggestions.contains(&"SOB".to_string()));
}

#[test]
fn test_suggest_alternative_instructions_unknown_context() {
    let provider = SuggestionProvider::new();
    
    let suggestions = provider.suggest_alternative_instructions("unknown_operation");
    assert!(suggestions.is_empty());
}

#[test]
fn test_is_valid_suggestion() {
    let provider = SuggestionProvider::new();
    
    assert!(provider.is_valid_suggestion("POB"));
    assert!(provider.is_valid_suggestion("DOD"));
    assert!(provider.is_valid_suggestion("STP"));
    assert!(!provider.is_valid_suggestion("INVALID"));
}

#[test]
fn test_suggestions_no_duplicates() {
    let provider = SuggestionProvider::new();
    
    // multiple methods might suggest the same instruction
    let suggestions = provider.suggest_common_fixes("dod");
    
    // count occurrences of "DOD"
    let dod_count = suggestions.iter().filter(|&s| s == "DOD").count();
    assert_eq!(dod_count, 1); // should appear only once
}

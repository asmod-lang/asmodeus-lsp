use asmodeus_lsp::analysis::utils::*;

#[test]
fn test_is_word_char() {
    assert!(is_word_char('a'));
    assert!(is_word_char('Z'));
    assert!(is_word_char('5'));
    assert!(is_word_char('_'));
    assert!(is_word_char('Ł'));
    
    assert!(!is_word_char(' '));
    assert!(!is_word_char('#'));
    assert!(!is_word_char(':'));
    assert!(!is_word_char('['));
}

#[test]
fn test_get_word_at_position() {
    let line = "    POB #42    ; comment";
    
    // "POB"
    let result = get_word_at_position(line, 5).unwrap();
    assert_eq!(result.0, "POB");
    assert_eq!(result.1, 4);
    assert_eq!(result.2, 7);
    
    // "42"
    let result = get_word_at_position(line, 10).unwrap();
    assert_eq!(result.0, "42");
    assert_eq!(result.1, 9);
    assert_eq!(result.2, 11);
    
    // space - should return None
    assert!(get_word_at_position(line, 3).is_none());
    
    // beyond line length
    assert!(get_word_at_position(line, 100).is_none());
}

#[test]
fn test_is_whole_word_match() {
    let line = "POB start POBtest";
    
    assert!(is_whole_word_match(line, 0, "POB"));
    assert!(is_whole_word_match(line, 4, "start"));
    assert!(!is_whole_word_match(line, 10, "POB")); // part of "POBtest"
}

#[test]
fn test_is_label_declaration() {
    assert!(is_label_declaration("start:", 0, "start"));
    assert!(is_label_declaration("  loop:", 2, "loop"));
    assert!(!is_label_declaration("start", 0, "start"));
    assert!(!is_label_declaration("SOB start", 4, "start"));
}

#[test]
fn test_is_valid_symbol_name() {
    assert!(is_valid_symbol_name("start"));
    assert!(is_valid_symbol_name("loop_1"));
    assert!(is_valid_symbol_name("_private"));
    
    assert!(!is_valid_symbol_name(""));
    assert!(!is_valid_symbol_name("123invalid"));
    assert!(!is_valid_symbol_name("invalid-name"));
    assert!(!is_valid_symbol_name("invalid name"));
}

#[test]
fn test_could_be_label() {
    assert!(could_be_label("start"));
    assert!(could_be_label("loop1"));
    assert!(could_be_label("myLabel"));
    
    assert!(!could_be_label("POB"));
    assert!(!could_be_label("START"));
    assert!(!could_be_label("LOOP_1"));
}

#[test]
fn test_find_word_occurrences() {
    let content = r#"start:
    POB #42
    SOB start
loop:
    DOD start
    SOB loop"#;
    
    let occurrences = find_word_occurrences(content, "start");
    assert_eq!(occurrences.len(), 3); // definition + 2 references
    
    let occurrences = find_word_occurrences(content, "loop");
    assert_eq!(occurrences.len(), 2); // definition + 1 reference
    
    let occurrences = find_word_occurrences(content, "nonexistent");
    assert_eq!(occurrences.len(), 0);
}

#[test]
fn test_find_label_definition() {
    let content = r#"start:
    POB #42
loop:
    SOB start"#;
    
    let def = find_label_definition("start", content).unwrap();
    assert_eq!(def.0, 0); // line 0
    assert_eq!(def.1, 0); // column 0
    assert_eq!(def.2, 5); // end column
    
    let def = find_label_definition("loop", content).unwrap();
    assert_eq!(def.0, 2); // line 2
    
    assert!(find_label_definition("nonexistent", content).is_none());
}

#[test]
fn test_get_label_info() {
    let content = r#"start:
    POB #42
loop:
    SOB start"#;
    
    let info = get_label_info("start", content).unwrap();
    assert!(info.contains("start"));
    assert!(info.contains("Line 1"));
    assert!(info.contains("start:"));
    
    assert!(get_label_info("nonexistent", content).is_none());
}

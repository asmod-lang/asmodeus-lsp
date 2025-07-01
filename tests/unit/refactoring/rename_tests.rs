use asmodeus_lsp::analysis::refactoring::RenameProvider;
use tower_lsp::lsp_types::*;

#[test]
fn test_rename_symbol_basic() {
    let provider = RenameProvider::new();
    let content = r#"start:
    POB #42
    SOB start"#;
    let uri = Url::parse("file:///test.asmod").unwrap();
    
    let edit = provider.rename_symbol(content, Position { line: 0, character: 2 }, "begin", &uri);
    
    assert!(edit.is_some());
    let edit = edit.unwrap();
    
    let changes = edit.changes.unwrap();
    let file_edits = changes.get(&uri).unwrap();
    
    assert_eq!(file_edits.len(), 2); // definition + reference
    
    // should include colon
    assert_eq!(file_edits[0].new_text, "begin:");
    
    // should not include colon
    assert_eq!(file_edits[1].new_text, "begin");
}

#[test]
fn test_rename_symbol_invalid_new_name() {
    let provider = RenameProvider::new();
    let content = "start:\n    POB #42";
    let uri = Url::parse("file:///test.asmod").unwrap();
    
    // invalid name starting with number
    let edit = provider.rename_symbol(content, Position { line: 0, character: 2 }, "123invalid", &uri);
    assert!(edit.is_none());
    
    // invalid name with special characters
    let edit = provider.rename_symbol(content, Position { line: 0, character: 2 }, "invalid-name", &uri);
    assert!(edit.is_none());
}

#[test]
fn test_rename_instruction_denied() {
    let provider = RenameProvider::new();
    let content = "POB #42";
    let uri = Url::parse("file:///test.asmod").unwrap();
    
    let edit = provider.rename_symbol(content, Position { line: 0, character: 1 }, "newname", &uri);
    assert!(edit.is_none()); // cannot rename instructions
}

#[test]
fn test_rename_to_instruction_name_denied() {
    let provider = RenameProvider::new();
    let content = "start:\n    POB #42";
    let uri = Url::parse("file:///test.asmod").unwrap();
    
    let edit = provider.rename_symbol(content, Position { line: 0, character: 2 }, "POB", &uri);
    assert!(edit.is_none()); // cannot rename to instruction name
}

#[test]
fn test_get_rename_range_valid_symbol() {
    let provider = RenameProvider::new();
    let content = "start:\n    POB #42";
    
    let range = provider.get_rename_range(content, Position { line: 0, character: 2 });
    
    assert!(range.is_some());
    let range = range.unwrap();
    assert_eq!(range.start.line, 0);
    assert_eq!(range.start.character, 0);
    assert_eq!(range.end.line, 0);
    assert_eq!(range.end.character, 5);
}

#[test]
fn test_get_rename_range_instruction_denied() {
    let provider = RenameProvider::new();
    let content = "POB #42";
    
    let range = provider.get_rename_range(content, Position { line: 0, character: 1 });
    assert!(range.is_none());
}

#[test]
fn test_validate_new_name() {
    let provider = RenameProvider::new();
    
    // valid names
    assert!(provider.validate_new_name("validName").is_ok());
    assert!(provider.validate_new_name("valid_name").is_ok());
    assert!(provider.validate_new_name("_private").is_ok());
    assert!(provider.validate_new_name("name123").is_ok());
    
    // invalid names
    assert!(provider.validate_new_name("123invalid").is_err());
    assert!(provider.validate_new_name("invalid-name").is_err());
    assert!(provider.validate_new_name("invalid name").is_err());
    assert!(provider.validate_new_name("").is_err());
    
    // instruction names
    assert!(provider.validate_new_name("POB").is_err());
    assert!(provider.validate_new_name("DOD").is_err());
    
    // reserved words
    assert!(provider.validate_new_name("AK").is_err());
    assert!(provider.validate_new_name("PC").is_err());
    assert!(provider.validate_new_name("SP").is_err());
}

#[test]
fn test_check_name_conflicts() {
    let provider = RenameProvider::new();
    let content = r#"existing:
    POB #42
another:
    STP"#;
    
    // should find conflict
    let conflicts = provider.check_name_conflicts(content, "existing", None);
    assert_eq!(conflicts.len(), 1);
    
    // should not find conflict for new name
    let conflicts = provider.check_name_conflicts(content, "newname", None);
    assert_eq!(conflicts.len(), 0);
    
    // should exclude specified position
    let exclude_pos = Position { line: 0, character: 0 };
    let conflicts = provider.check_name_conflicts(content, "existing", Some(exclude_pos));
    assert_eq!(conflicts.len(), 0);
}

#[test]
fn test_rename_multiple_references() {
    let provider = RenameProvider::new();
    let content = r#"start:
    POB #42
    SOB start
loop:
    DOD start
    SOB loop
    SOB start"#;
    let uri = Url::parse("file:///test.asmod").unwrap();
    
    let edit = provider.rename_symbol(content, Position { line: 0, character: 2 }, "begin", &uri);
    
    assert!(edit.is_some());
    let edit = edit.unwrap();
    
    let changes = edit.changes.unwrap();
    let file_edits = changes.get(&uri).unwrap();
    
    assert_eq!(file_edits.len(), 4); // 1 definition + 3 references
}

use asmodeus_lsp::analysis::refactoring::RefactoringActionsProvider;
use tower_lsp::lsp_types::*;

#[test]
fn test_create_uppercase_action() {
    let provider = RefactoringActionsProvider::new();
    let content = "pob #42";
    let uri = Url::parse("file:///test.asmod").unwrap();

    let range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 3,
        },
    };

    let actions = provider.get_refactoring_actions(content, range, &uri);

    let uppercase_action = actions.iter().find(|action| match action {
        CodeActionOrCommand::CodeAction(action) => action.title == "Convert to uppercase",
        _ => false,
    });

    assert!(uppercase_action.is_some());

    match uppercase_action.unwrap() {
        CodeActionOrCommand::CodeAction(action) => {
            assert_eq!(action.kind, Some(CodeActionKind::REFACTOR));

            let edit = action.edit.as_ref().unwrap();
            let changes = edit.changes.as_ref().unwrap();
            let file_edits = changes.get(&uri).unwrap();

            assert_eq!(file_edits[0].new_text, "POB");
        }
        _ => panic!("Expected CodeAction"),
    }
}

#[test]
fn test_create_add_comment_action() {
    let provider = RefactoringActionsProvider::new();
    let content = "POB #42";
    let uri = Url::parse("file:///test.asmod").unwrap();

    let range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 7,
        },
    };

    let actions = provider.get_refactoring_actions(content, range, &uri);

    let comment_action = actions.iter().find(|action| match action {
        CodeActionOrCommand::CodeAction(action) => action.title == "Add comment",
        _ => false,
    });

    assert!(comment_action.is_some());

    match comment_action.unwrap() {
        CodeActionOrCommand::CodeAction(action) => {
            let edit = action.edit.as_ref().unwrap();
            let changes = edit.changes.as_ref().unwrap();
            let file_edits = changes.get(&uri).unwrap();

            assert!(file_edits[0].new_text.contains("TODO: Add comment"));
        }
        _ => panic!("Expected CodeAction"),
    }
}

#[test]
fn test_create_format_instruction_action() {
    let provider = RefactoringActionsProvider::new();
    let content = "    POB #42";
    let uri = Url::parse("file:///test.asmod").unwrap();

    let range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 11,
        },
    };

    let actions = provider.get_refactoring_actions(content, range, &uri);

    // should NOT suggest formatting for already formatted content
    let format_action = actions.iter().find(|action| match action {
        CodeActionOrCommand::CodeAction(action) => action.title == "Format instruction",
        _ => false,
    });

    assert!(format_action.is_none()); // should be None
}

#[test]
fn test_create_format_instruction_action_with_unformatted_content() {
    let provider = RefactoringActionsProvider::new();
    let content = "POB #42"; // no indentation
    let uri = Url::parse("file:///test.asmod").unwrap();

    let range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 7,
        },
    };

    let actions = provider.get_refactoring_actions(content, range, &uri);

    let format_action = actions.iter().find(|action| match action {
        CodeActionOrCommand::CodeAction(action) => action.title == "Format instruction",
        _ => false,
    });

    assert!(format_action.is_some());
}

#[test]
fn test_create_add_label_action() {
    let provider = RefactoringActionsProvider::new();
    let content = "POB #42";
    let uri = Url::parse("file:///test.asmod").unwrap();

    let range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 7,
        },
    };

    let actions = provider.get_refactoring_actions(content, range, &uri);

    let label_action = actions.iter().find(|action| match action {
        CodeActionOrCommand::CodeAction(action) => action.title == "Add label above instruction",
        _ => false,
    });

    assert!(label_action.is_some());

    match label_action.unwrap() {
        CodeActionOrCommand::CodeAction(action) => {
            let edit = action.edit.as_ref().unwrap();
            let changes = edit.changes.as_ref().unwrap();
            let file_edits = changes.get(&uri).unwrap();

            assert_eq!(file_edits[0].new_text, "label:\n");
        }
        _ => panic!("Expected CodeAction"),
    }
}

#[test]
fn test_no_actions_for_empty_range() {
    let provider = RefactoringActionsProvider::new();
    let content = "";
    let uri = Url::parse("file:///test.asmod").unwrap();

    let range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 0,
        },
    };

    let actions = provider.get_refactoring_actions(content, range, &uri);

    assert!(actions.is_empty());
}

#[test]
fn test_no_uppercase_action_for_already_uppercase() {
    let provider = RefactoringActionsProvider::new();
    let content = "POB #42";
    let uri = Url::parse("file:///test.asmod").unwrap();

    let range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 3,
        },
    };

    let actions = provider.get_refactoring_actions(content, range, &uri);

    let uppercase_action = actions.iter().find(|action| match action {
        CodeActionOrCommand::CodeAction(action) => action.title == "Convert to uppercase",
        _ => false,
    });

    assert!(uppercase_action.is_none());
}

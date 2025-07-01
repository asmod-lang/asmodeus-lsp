use tower_lsp::lsp_types::*;
use std::collections::HashMap;
use crate::analysis::utils::InstructionDatabase;

#[derive(Debug)]
pub struct RefactoringActionsProvider {
    instruction_db: InstructionDatabase,
}

impl RefactoringActionsProvider {
    pub fn new() -> Self {
        Self {
            instruction_db: InstructionDatabase::new(),
        }
    }

    pub fn get_refactoring_actions(&self, content: &str, range: Range, uri: &Url) -> Vec<CodeActionOrCommand> {
        let mut actions = Vec::new();
        
        // "Convert to uppercase" 
        if let Some(action) = self.create_uppercase_action(content, range, uri) {
            actions.push(action);
        }
        
        // "Add comment" 
        if let Some(action) = self.create_add_comment_action(content, range, uri) {
            actions.push(action);
        }

        // "Format instruction"
        if let Some(action) = self.create_format_instruction_action(content, range, uri) {
            actions.push(action);
        }

        // "Add label"
        if let Some(action) = self.create_add_label_action(content, range, uri) {
            actions.push(action);
        }
        
        actions
    }

    fn create_uppercase_action(&self, content: &str, range: Range, uri: &Url) -> Option<CodeActionOrCommand> {
        let lines: Vec<&str> = content.lines().collect();
        let line_num = range.start.line as usize;
        
        if line_num >= lines.len() {
            return None;
        }
        
        let line = lines[line_num];
        let start_char = range.start.character as usize;
        let end_char = range.end.character as usize;
        
        if start_char >= line.len() || end_char > line.len() {
            return None;
        }
        
        let selected_text = &line[start_char..end_char];
        
        if self.instruction_db.is_valid_instruction(&selected_text.to_uppercase()) && selected_text != selected_text.to_uppercase() {
            let edit = TextEdit {
                range,
                new_text: selected_text.to_uppercase(),
            };
            
            let mut changes = HashMap::new();
            changes.insert(uri.clone(), vec![edit]);
            
            return Some(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Convert to uppercase".to_string(),
                kind: Some(CodeActionKind::REFACTOR),
                diagnostics: None,
                edit: Some(WorkspaceEdit {
                    changes: Some(changes),
                    document_changes: None,
                    change_annotations: None,
                }),
                command: None,
                is_preferred: Some(false),
                disabled: None,
                data: None,
            }));
        }
        
        None
    }

    fn create_add_comment_action(&self, content: &str, range: Range, uri: &Url) -> Option<CodeActionOrCommand> {
        let lines: Vec<&str> = content.lines().collect();
        let line_num = range.start.line as usize;
        
        if line_num >= lines.len() {
            return None;
        }
        
        let line = lines[line_num];
        
        // add comment if line does not have it
        if !line.contains(';') && !line.trim().is_empty() {
            let edit = TextEdit {
                range: Range {
                    start: Position {
                        line: range.start.line,
                        character: line.len() as u32,
                    },
                    end: Position {
                        line: range.start.line,
                        character: line.len() as u32,
                    },
                },
                new_text: "    ; TODO: Add comment".to_string(),
            };
            
            let mut changes = HashMap::new();
            changes.insert(uri.clone(), vec![edit]);
            
            return Some(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Add comment".to_string(),
                kind: Some(CodeActionKind::REFACTOR),
                diagnostics: None,
                edit: Some(WorkspaceEdit {
                    changes: Some(changes),
                    document_changes: None,
                    change_annotations: None,
                }),
                command: None,
                is_preferred: Some(false),
                disabled: None,
                data: None,
            }));
        }
        
        None
    }

    fn create_format_instruction_action(&self, content: &str, range: Range, uri: &Url) -> Option<CodeActionOrCommand> {
        let lines: Vec<&str> = content.lines().collect();
        let line_num = range.start.line as usize;
        
        if line_num >= lines.len() {
            return None;
        }
        
        let line = lines[line_num];
        let trimmed = line.trim();
        
        // check for line if it has instruction to format
        if !trimmed.is_empty() && !line.starts_with("    ") {
            let formatted_line = format!("    {}", trimmed); // 4 spaces indentation
            
            let edit = TextEdit {
                range: Range {
                    start: Position {
                        line: range.start.line,
                        character: 0,
                    },
                    end: Position {
                        line: range.start.line,
                        character: line.len() as u32,
                    },
                },
                new_text: formatted_line,
            };
            
            let mut changes = HashMap::new();
            changes.insert(uri.clone(), vec![edit]);
            
            return Some(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Format instruction".to_string(),
                kind: Some(CodeActionKind::REFACTOR),
                diagnostics: None,
                edit: Some(WorkspaceEdit {
                    changes: Some(changes),
                    document_changes: None,
                    change_annotations: None,
                }),
                command: None,
                is_preferred: Some(false),
                disabled: None,
                data: None,
            }));
        }
        
        None
    }

    fn create_add_label_action(&self, content: &str, range: Range, uri: &Url) -> Option<CodeActionOrCommand> {
        let lines: Vec<&str> = content.lines().collect();
        let line_num = range.start.line as usize;
        
        if line_num >= lines.len() {
            return None;
        }
        
        let line = lines[line_num];
        let trimmed = line.trim();
        
        // check if line has instruction or does not have label
        if !trimmed.is_empty() && !trimmed.contains(':') {
            let words: Vec<&str> = trimmed.split_whitespace().collect();
            if !words.is_empty() && self.instruction_db.is_valid_instruction(words[0]) {
                let edit = TextEdit {
                    range: Range {
                        start: Position {
                            line: range.start.line,
                            character: 0,
                        },
                        end: Position {
                            line: range.start.line,
                            character: 0,
                        },
                    },
                    new_text: "label:\n".to_string(),
                };
                
                let mut changes = HashMap::new();
                changes.insert(uri.clone(), vec![edit]);
                
                return Some(CodeActionOrCommand::CodeAction(CodeAction {
                    title: "Add label above".to_string(),
                    kind: Some(CodeActionKind::REFACTOR),
                    diagnostics: None,
                    edit: Some(WorkspaceEdit {
                        changes: Some(changes),
                        document_changes: None,
                        change_annotations: None,
                    }),
                    command: None,
                    is_preferred: Some(false),
                    disabled: None,
                    data: None,
                }));
            }
        }
        
        None
    }
}

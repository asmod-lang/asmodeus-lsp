use tower_lsp::lsp_types::*;
use std::collections::HashMap;
use crate::analysis::refactoring::SuggestionProvider;

#[derive(Debug)]
pub struct QuickFixProvider {
    suggestion_provider: SuggestionProvider,
}

impl QuickFixProvider {
    pub fn new() -> Self {
        Self {
            suggestion_provider: SuggestionProvider::new(),
        }
    }

    pub fn create_quick_fix(&self, diagnostic: &Diagnostic, content: &str, uri: &Url) -> Option<CodeActionOrCommand> {
        if diagnostic.message.contains("Unknown instruction") {
            return self.suggest_instruction_correction(diagnostic, content, uri);
        }
        
        if diagnostic.message.contains("undefined macro") {
            return self.suggest_instruction_correction(diagnostic, content, uri);
        }
        
        None
    }

    fn suggest_instruction_correction(&self, diagnostic: &Diagnostic, content: &str, uri: &Url) -> Option<CodeActionOrCommand> {
        let message = &diagnostic.message;
        let start_quote = message.find('\'')?;
        let end_quote = message.rfind('\'')?;
        if start_quote >= end_quote {
            return None;
        }
        
        let unknown_instruction = &message[start_quote + 1..end_quote];
        
        let suggestions = self.suggestion_provider.find_similar_instructions(unknown_instruction);
        
        if let Some(suggestion) = suggestions.first() {
            let edit = TextEdit {
                range: diagnostic.range,
                new_text: suggestion.clone(),
            };
            
            let mut changes = HashMap::new();
            changes.insert(uri.clone(), vec![edit]);
            
            return Some(CodeActionOrCommand::CodeAction(CodeAction {
                title: format!("Replace with '{}'", suggestion),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: Some(vec![diagnostic.clone()]),
                edit: Some(WorkspaceEdit {
                    changes: Some(changes),
                    document_changes: None,
                    change_annotations: None,
                }),
                command: None,
                is_preferred: Some(true),
                disabled: None,
                data: None,
            }));
        }
        
        None
    }
}

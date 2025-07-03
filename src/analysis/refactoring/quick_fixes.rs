use crate::analysis::refactoring::SuggestionProvider;
use std::collections::HashMap;
use tower_lsp::lsp_types::*;

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

    pub fn create_quick_fix(
        &self,
        diagnostic: &Diagnostic,
        content: &str,
        uri: &Url,
    ) -> Option<CodeActionOrCommand> {
        if diagnostic.message.contains("Unknown instruction") {
            return self.suggest_instruction_correction(diagnostic, content, uri);
        }

        if diagnostic.message.contains("undefined macro") {
            return self.suggest_instruction_correction(diagnostic, content, uri);
        }

        None
    }

    fn suggest_instruction_correction(
        &self,
        diagnostic: &Diagnostic,
        content: &str,
        uri: &Url,
    ) -> Option<CodeActionOrCommand> {
        let message = &diagnostic.message;
        let start_quote = message.find('\'')?;
        let end_quote = message.rfind('\'')?;
        if start_quote >= end_quote {
            return None;
        }

        let unknown_instruction = &message[start_quote + 1..end_quote];

        let mut suggestions = self
            .suggestion_provider
            .suggest_common_fixes(unknown_instruction);

        if suggestions.is_empty() {
            if let Some(context) = self.extract_context_from_content(content, diagnostic) {
                let context_suggestions = self
                    .suggestion_provider
                    .suggest_alternative_instructions(&context);

                for suggestion in context_suggestions {
                    if self.suggestion_provider.is_valid_suggestion(&suggestion) {
                        suggestions.push(suggestion);
                    }
                }
            }
        }

        if let Some(suggestion) = suggestions
            .into_iter()
            .find(|s| self.suggestion_provider.is_valid_suggestion(s))
        {
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

    fn extract_context_from_content(
        &self,
        content: &str,
        diagnostic: &Diagnostic,
    ) -> Option<String> {
        let lines: Vec<&str> = content.lines().collect();
        let line_index = diagnostic.range.start.line as usize;

        if line_index >= lines.len() {
            return None;
        }

        let line = lines[line_index];

        if let Some(comment_pos) = line.find(';') {
            let comment = &line[comment_pos + 1..].trim().to_lowercase();

            if comment.contains("add") || comment.contains("plus") || comment.contains("dodaj") {
                return Some("add".to_string());
            } else if comment.contains("sub")
                || comment.contains("minus")
                || comment.contains("odejmij")
            {
                return Some("subtract".to_string());
            } else if comment.contains("load")
                || comment.contains("pobierz")
                || comment.contains("get")
            {
                return Some("load".to_string());
            } else if comment.contains("store")
                || comment.contains("save")
                || comment.contains("zapisz")
            {
                return Some("store".to_string());
            } else if comment.contains("jump")
                || comment.contains("goto")
                || comment.contains("skok")
            {
                return Some("jump".to_string());
            } else if comment.contains("stop")
                || comment.contains("halt")
                || comment.contains("koniec")
            {
                return Some("stop".to_string());
            } else if comment.contains("input")
                || comment.contains("read")
                || comment.contains("wejście")
            {
                return Some("input".to_string());
            } else if comment.contains("output")
                || comment.contains("print")
                || comment.contains("wyjście")
            {
                return Some("output".to_string());
            } else if comment.contains("multiply")
                || comment.contains("mul")
                || comment.contains("mnożenie")
            {
                return Some("multiply".to_string());
            } else if comment.contains("divide")
                || comment.contains("div")
                || comment.contains("dzielenie")
            {
                return Some("divide".to_string());
            }
        }

        let context_lines = self.get_surrounding_context(&lines, line_index);
        for context_line in context_lines {
            if context_line.contains("POB") || context_line.contains("WEJSCIE") {
                return Some("load".to_string());
            } else if context_line.contains("ŁAD") || context_line.contains("WYJSCIE") {
                return Some("store".to_string());
            } else if context_line.contains("DOD") {
                return Some("add".to_string());
            } else if context_line.contains("ODE") {
                return Some("subtract".to_string());
            } else if context_line.contains("SOB")
                || context_line.contains("SOM")
                || context_line.contains("SOZ")
            {
                return Some("jump".to_string());
            }
        }

        None
    }

    fn get_surrounding_context<'a>(
        &self,
        lines: &'a Vec<&'a str>,
        line_index: usize,
    ) -> Vec<&'a str> {
        let mut context = Vec::new();

        if line_index > 0 {
            context.push(lines[line_index - 1]);
        }

        if line_index + 1 < lines.len() {
            context.push(lines[line_index + 1]);
        }

        context
    }
}

use crate::analysis::language::NavigationProvider;
use crate::analysis::utils::{
    get_line_at_position, get_word_at_position, is_label_definition_location, is_valid_position,
    is_valid_symbol_name, word_range, InstructionDatabase,
};
use std::collections::HashMap;
use tower_lsp::lsp_types::*;

#[derive(Debug)]
pub struct RenameProvider {
    instruction_db: InstructionDatabase,
    navigation_provider: NavigationProvider,
}

impl RenameProvider {
    pub fn new() -> Self {
        Self {
            instruction_db: InstructionDatabase::new(),
            navigation_provider: NavigationProvider::new(),
        }
    }

    /// store symbol in code
    pub fn rename_symbol(
        &self,
        content: &str,
        position: Position,
        new_name: &str,
        uri: &Url,
    ) -> Option<WorkspaceEdit> {
        if !is_valid_position(content, position) {
            return None;
        }

        let current_line = get_line_at_position(content, position)?;
        let cursor_pos = position.character as usize;

        let word_info = get_word_at_position(current_line, cursor_pos)?;
        let (old_name, _, _) = word_info;

        if let Err(_) = self.validate_new_name(new_name) {
            return None;
        }

        if self.instruction_db.is_valid_instruction(&old_name) {
            return None;
        }

        let conflicts = self.check_name_conflicts(content, new_name, Some(position));
        if !conflicts.is_empty() {
            return None;
        }

        let references = self
            .navigation_provider
            .find_references(content, position, uri, true);

        if references.is_empty() {
            return None;
        }

        // edit text for all references
        let mut edits = Vec::new();
        for reference in references {
            // whether is label definition (need : after)
            let edit_text = if is_label_definition_location(content, &reference) {
                format!("{}:", new_name)
            } else {
                new_name.to_string()
            };

            edits.push(TextEdit {
                range: reference.range,
                new_text: edit_text,
            });
        }

        let mut changes = HashMap::new();
        changes.insert(uri.clone(), edits);

        Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        })
    }

    pub fn get_rename_range(&self, content: &str, position: Position) -> Option<Range> {
        if !is_valid_position(content, position) {
            return None;
        }

        let current_line = get_line_at_position(content, position)?;
        let cursor_pos = position.character as usize;

        let word_info = get_word_at_position(current_line, cursor_pos)?;
        let (word, start_pos, end_pos) = word_info;

        if self.instruction_db.is_valid_instruction(&word) {
            return None;
        }

        if is_valid_symbol_name(&word) {
            Some(word_range(position.line, start_pos, end_pos))
        } else {
            None
        }
    }

    pub fn validate_new_name(&self, new_name: &str) -> Result<(), String> {
        if !is_valid_symbol_name(new_name) {
            return Err("Invalid symbol name. Must start with letter or underscore and contain only alphanumeric characters and underscores.".to_string());
        }

        if self.instruction_db.is_valid_instruction(new_name) {
            return Err(format!(
                "Name '{}' conflicts with instruction name.",
                new_name
            ));
        }

        if self.is_reserved_word(new_name) {
            return Err(format!("Name '{}' is a reserved word.", new_name));
        }

        Ok(())
    }

    fn is_reserved_word(&self, word: &str) -> bool {
        let reserved_words = [
            "AK",    // Akumulator
            "PC",    // Program Counter
            "SP",    // Stack Pointer
            "IR",    // Instruction Register
            "MAR",   // Memory Address Register
            "MBR",   // Memory Buffer Register
            "FLAGS", // Flags
        ];

        reserved_words.contains(&word.to_uppercase().as_str())
    }

    pub fn check_name_conflicts(
        &self,
        content: &str,
        new_name: &str,
        exclude_position: Option<Position>,
    ) -> Vec<Position> {
        let mut conflicts = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // label definitions
            if let Some(colon_pos) = trimmed.find(':') {
                let label_name = &trimmed[..colon_pos];
                if label_name == new_name {
                    let position = Position {
                        line: line_num as u32,
                        character: (line.len() - trimmed.len()) as u32,
                    };

                    // skip position that is being changed
                    if let Some(exclude_pos) = exclude_position {
                        if position.line != exclude_pos.line
                            || position.character != exclude_pos.character
                        {
                            conflicts.push(position);
                        }
                    } else {
                        conflicts.push(position);
                    }
                }
            }
        }

        conflicts
    }
}

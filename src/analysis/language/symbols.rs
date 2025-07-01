use tower_lsp::lsp_types::*;
use crate::analysis::utils::{is_valid_symbol_name, InstructionDatabase};

#[derive(Debug)]
pub struct SymbolProvider {
    instruction_db: InstructionDatabase,
}

impl SymbolProvider {
    pub fn new() -> Self {
        Self {
            instruction_db: InstructionDatabase::new(),
        }
    }

    pub fn get_document_symbols(&self, content: &str) -> Vec<SymbolInformation> {
        let mut symbols = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        
        for (line_num, line) in lines.iter().enumerate() {
            println!("Processing line {}: '{}'", line_num, line);
            let trimmed = line.trim();
            
            // search for labels
            if let Some(colon_pos) = trimmed.find(':') {
                let label_name = &trimmed[..colon_pos];

                if !label_name.is_empty() && is_valid_symbol_name(label_name) {
                    symbols.push(SymbolInformation {
                        name: label_name.to_string(),
                        kind: SymbolKind::FUNCTION,
                        tags: None,
                        deprecated: None,
                        location: Location {
                            uri: Url::parse("file:///dummy").unwrap(),
                            range: Range {
                                start: Position {
                                    line: line_num as u32,
                                    character: (line.len() - trimmed.len()) as u32, // leading whitespace
                                },
                                end: Position {
                                    line: line_num as u32,
                                    character: (line.len() - trimmed.len() + colon_pos) as u32,
                                },
                            },
                        },
                        container_name: None,
                    });
                }
            }
        }
        
        symbols
    }

    /// based on query
    pub fn filter_workspace_symbols(&self, symbols: &mut Vec<SymbolInformation>, query: &str, uri: &Url) {
        // update URI for all symbols
        for symbol in symbols.iter_mut() {
            if symbol.name.to_lowercase().contains(&query.to_lowercase()) {
                symbol.location.uri = uri.clone();
            }
        }

        // only those that matches query
        symbols.retain(|symbol| symbol.name.to_lowercase().contains(&query.to_lowercase()));
    }

    pub fn get_rename_range(&self, content: &str, position: Position) -> Option<Range> {
        let lines: Vec<&str> = content.lines().collect();
        
        if position.line as usize >= lines.len() {
            return None;
        }

        let current_line = lines[position.line as usize];
        let cursor_pos = position.character as usize;

        let word_info = crate::analysis::utils::get_word_at_position(current_line, cursor_pos)?;
        let (word, start_pos, end_pos) = word_info;
        
        // only labels renaming are valid
        if self.instruction_db.is_valid_instruction(&word) {
            return None;
        }
        
        if is_valid_symbol_name(&word) {
            Some(Range {
                start: Position {
                    line: position.line,
                    character: start_pos as u32,
                },
                end: Position {
                    line: position.line,
                    character: end_pos as u32,
                },
            })
        } else {
            None
        }
    }
}

use crate::analysis::utils::{create_location, is_valid_symbol_name, word_range};
use tower_lsp::lsp_types::*;

#[derive(Debug)]
pub struct SymbolProvider {}

impl SymbolProvider {
    pub fn new() -> Self {
        Self {}
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
                    let label_start = line.len() - trimmed.len(); // leading whitespace
                    let range = word_range(line_num as u32, label_start, label_start + colon_pos);

                    symbols.push(SymbolInformation {
                        name: label_name.to_string(),
                        kind: SymbolKind::FUNCTION,
                        tags: None,
                        deprecated: None,
                        location: create_location(&Url::parse("file:///dummy").unwrap(), range),
                        container_name: None,
                    });
                }
            }
        }

        symbols
    }

    /// based on query
    pub fn filter_workspace_symbols(
        &self,
        symbols: &mut Vec<SymbolInformation>,
        query: &str,
        uri: &Url,
    ) {
        // update URI for all symbols
        for symbol in symbols.iter_mut() {
            if symbol.name.to_lowercase().contains(&query.to_lowercase()) {
                symbol.location.uri = uri.clone();
            }
        }

        // only those that matches query
        symbols.retain(|symbol| symbol.name.to_lowercase().contains(&query.to_lowercase()));
    }
}

use crate::analysis::utils::{
    coordinates_to_location, create_word_location, find_label_definition, find_word_occurrences,
    get_line_at_position, get_word_at_position, is_label_declaration, is_valid_position,
    is_whole_word_match,
};
use tower_lsp::lsp_types::*;

#[derive(Debug)]
pub struct NavigationProvider {}

impl NavigationProvider {
    pub fn new() -> Self {
        Self {}
    }

    /// go-to-definition functionality
    pub fn get_definition(
        &self,
        content: &str,
        position: Position,
        uri: &Url,
    ) -> Option<GotoDefinitionResponse> {
        if !is_valid_position(content, position) {
            return None;
        }

        let current_line = get_line_at_position(content, position)?;
        let cursor_pos = position.character as usize;

        let word_info = get_word_at_position(current_line, cursor_pos)?;
        let (word, _, _) = word_info;

        self.find_label_definition(&word, content, uri)
    }

    fn find_label_definition(
        &self,
        label: &str,
        content: &str,
        uri: &Url,
    ) -> Option<GotoDefinitionResponse> {
        if let Some((line_num, start_pos, _end_pos)) = find_label_definition(label, content) {
            let location = create_word_location(uri, line_num, start_pos, label.len());
            return Some(GotoDefinitionResponse::Scalar(location));
        }
        None
    }

    pub fn find_references(
        &self,
        content: &str,
        position: Position,
        uri: &Url,
        include_declaration: bool,
    ) -> Vec<Location> {
        let mut locations = Vec::new();

        if !is_valid_position(content, position) {
            return locations; // empty list
        }

        let current_line = match get_line_at_position(content, position) {
            Some(line) => line,
            None => return locations, // empty list
        };

        let cursor_pos = position.character as usize;

        let word_info = match get_word_at_position(current_line, cursor_pos) {
            Some(info) => info,
            None => return locations, // empty list
        };

        let (word, _, _) = word_info;

        // find ALL occurrences
        let occurrences = find_word_occurrences(content, &word);
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, start_pos, end_pos) in occurrences {
            if line_num >= lines.len() {
                continue;
            }

            let line = lines[line_num];

            if !is_whole_word_match(line, start_pos, &word) {
                continue;
            }

            // skip definition if not needed
            if !include_declaration && is_label_declaration(line, start_pos, &word) {
                continue;
            }

            if end_pos == start_pos + word.len() {
                locations.push(create_word_location(uri, line_num, start_pos, word.len()));
            } else {
                locations.push(coordinates_to_location(uri, line_num, start_pos, end_pos));
            }
        }

        locations
    }
}

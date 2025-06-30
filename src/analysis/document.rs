use serde::{Deserialize, Serialize};
use tower_lsp::lsp_types::*;

#[derive(Debug, Clone)]
pub struct DocumentState {
    pub uri: Url,
    pub content: String,
    pub version: i32,
    pub diagnostics: Vec<Diagnostic>,
}

impl DocumentState {
    pub fn new(uri: Url, content: String, version: i32) -> Self {
        Self {
            uri,
            content,
            version,
            diagnostics: Vec::new(),
        }
    }

    pub fn update_content(&mut self, content: String, version: i32) {
        self.content = content;
        self.version = version;
        self.diagnostics.clear(); // clear old diagnostics
    }

    pub fn set_diagnostics(&mut self, diagnostics: Vec<Diagnostic>) {
        self.diagnostics = diagnostics;
    }
}

#[derive(Debug, Clone)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

impl From<tower_lsp::lsp_types::Position> for Position {
    fn from(pos: tower_lsp::lsp_types::Position) -> Self {
        Self {
            line: pos.line,
            character: pos.character,
        }
    }
}

impl From<Position> for tower_lsp::lsp_types::Position {
    fn from(pos: Position) -> Self {
        Self {
            line: pos.line,
            character: pos.character,
        }
    }
}

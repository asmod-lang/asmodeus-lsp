use tower_lsp::lsp_types::*;

/// (line, character) to Range
pub fn position_to_range(line: u32, start_char: u32, end_char: u32) -> Range {
    Range {
        start: Position {
            line,
            character: start_char,
        },
        end: Position {
            line,
            character: end_char,
        },
    }
}

/// creates Range from word position in line
pub fn word_range(line_num: u32, start_pos: usize, end_pos: usize) -> Range {
    Range {
        start: Position {
            line: line_num,
            character: start_pos as u32,
        },
        end: Position {
            line: line_num,
            character: end_pos as u32,
        },
    }
}

/// whether position is in valid range in line
pub fn is_valid_position(content: &str, position: Position) -> bool {
    let lines: Vec<&str> = content.lines().collect();

    if position.line as usize >= lines.len() {
        return false;
    }

    let line = lines[position.line as usize];
    position.character as usize <= line.len()
}

pub fn get_line_at_position(content: &str, position: Position) -> Option<&str> {
    let lines: Vec<&str> = content.lines().collect();

    if position.line as usize >= lines.len() {
        return None;
    }

    Some(lines[position.line as usize])
}

/// from URI and Range
pub fn create_location(uri: &Url, range: Range) -> Location {
    Location {
        uri: uri.clone(),
        range,
    }
}

/// from URI and word position
pub fn create_word_location(
    uri: &Url,
    line_num: usize,
    start_pos: usize,
    word_len: usize,
) -> Location {
    Location {
        uri: uri.clone(),
        range: Range {
            start: Position {
                line: line_num as u32,
                character: start_pos as u32,
            },
            end: Position {
                line: line_num as u32,
                character: (start_pos + word_len) as u32,
            },
        },
    }
}

/// converts coordinates (line_num, start_pos, end_pos) to Location
pub fn coordinates_to_location(
    uri: &Url,
    line_num: usize,
    start_pos: usize,
    end_pos: usize,
) -> Location {
    Location {
        uri: uri.clone(),
        range: Range {
            start: Position {
                line: line_num as u32,
                character: start_pos as u32,
            },
            end: Position {
                line: line_num as u32,
                character: end_pos as u32,
            },
        },
    }
}

pub fn create_diagnostic(
    range: Range,
    severity: DiagnosticSeverity,
    code: &str,
    message: String,
) -> Diagnostic {
    Diagnostic {
        range,
        severity: Some(severity),
        code: Some(NumberOrString::String(code.to_string())),
        source: Some("asmodeus-lsp".to_string()),
        message,
        related_information: None,
        tags: None,
        code_description: None,
        data: None,
    }
}

/// for parser (parseid) / lexer (lexariel) error with default position
pub fn create_parser_diagnostic(message: String, error_type: &str) -> Diagnostic {
    let range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 1,
        },
    };
    create_diagnostic(range, DiagnosticSeverity::ERROR, error_type, message)
}

/// for semantic error with specified position
pub fn create_semantic_diagnostic(
    line: usize,
    column: usize,
    length: usize,
    code: &str,
    message: String,
) -> Diagnostic {
    let range = Range {
        start: Position {
            line: (line.saturating_sub(1)) as u32,
            character: (column.saturating_sub(1)) as u32,
        },
        end: Position {
            line: (line.saturating_sub(1)) as u32,
            character: (column + length) as u32,
        },
    };
    create_diagnostic(range, DiagnosticSeverity::ERROR, code, message)
}

/// checks whether there is : after the symbol
pub fn is_label_definition_location(content: &str, location: &Location) -> bool {
    let lines: Vec<&str> = content.lines().collect();
    let line_num = location.range.start.line as usize;

    if line_num >= lines.len() {
        return false;
    }

    let line = lines[line_num];
    let end_pos = location.range.end.character as usize;

    if end_pos < line.len() {
        line.chars().nth(end_pos) == Some(':')
    } else {
        false
    }
}

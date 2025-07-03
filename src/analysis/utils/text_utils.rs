/// may be part of word / identifier
pub fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == 'Ł'
}

/// finds words at position in line
/// (word, start_position, end_position) or None if not found
pub fn get_word_at_position(line: &str, cursor_pos: usize) -> Option<(String, usize, usize)> {
    if cursor_pos > line.len() {
        return None;
    }

    let chars: Vec<char> = line.chars().collect();

    // is word char?
    if cursor_pos < chars.len() && !is_word_char(chars[cursor_pos]) {
        return None;
    }

    let mut start = cursor_pos;
    while start > 0 && is_word_char(chars.get(start - 1).copied().unwrap_or(' ')) {
        start -= 1;
    }

    let mut end = cursor_pos;
    while end < chars.len() && is_word_char(chars.get(end).copied().unwrap_or(' ')) {
        end += 1;
    }

    if start == end {
        return None;
    }

    let word: String = chars[start..end].iter().collect();
    Some((word, start, end))
}

pub fn is_whole_word_match(line: &str, pos: usize, word: &str) -> bool {
    let chars: Vec<char> = line.chars().collect();

    // check char before word
    if pos > 0 {
        if let Some(prev_char) = chars.get(pos - 1) {
            if is_word_char(*prev_char) {
                return false;
            }
        }
    }

    // check char after word
    let end_pos = pos + word.len();
    if let Some(next_char) = chars.get(end_pos) {
        if is_word_char(*next_char) {
            return false;
        }
    }

    true
}

/// (has : after)
pub fn is_label_declaration(line: &str, pos: usize, word: &str) -> bool {
    let after_word_pos = pos + word.len();
    line.chars().nth(after_word_pos) == Some(':')
}

pub fn is_valid_symbol_name(name: &str) -> bool {
    !name.is_empty()
        && name.chars().all(|c| c.is_alphanumeric() || c == '_')
        && !name.chars().next().unwrap().is_ascii_digit()
}

pub fn could_be_label(word: &str) -> bool {
    !word
        .chars()
        .all(|c| c.is_uppercase() || c.is_ascii_digit() || c == '_')
}

pub fn find_word_occurrences(content: &str, word: &str) -> Vec<(usize, usize, usize)> {
    let mut occurrences = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        let mut search_pos = 0;
        while let Some(pos) = line[search_pos..].find(word) {
            let actual_pos = search_pos + pos;

            if is_whole_word_match(line, actual_pos, word) {
                occurrences.push((line_num, actual_pos, actual_pos + word.len()));
            }
            search_pos = actual_pos + word.len();
        }
    }

    occurrences
}

pub fn find_label_definition(label: &str, content: &str) -> Option<(usize, usize, usize)> {
    let lines: Vec<&str> = content.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("{}:", label)) {
            let label_start = line.len() - trimmed.len(); // position after leading whitespace
            return Some((line_num, label_start, label_start + label.len()));
        }
    }
    None
}

pub fn get_label_info(word: &str, content: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("{}:", word)) {
            return Some(format!(
                "**Label:** `{}`\n\n**Defined at:** Line {}\n\n**Definition:** `{}`",
                word,
                line_num + 1,
                trimmed
            ));
        }
    }
    None
}

use tower_lsp::lsp_types::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct RawSemanticToken {
    line: u32,
    start_char: u32,
    length: u32,
    token_type: u32,
    token_modifiers: u32,
}

#[derive(Debug)]
pub struct SemanticAnalyzer;

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze_document(&self, content: &str, _uri: &Url) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // lexariel
        match lexariel::tokenize(content) {
            Ok(tokens) => {
                // parseid
                match parseid::parse(tokens) {
                    Ok(ast) => {
                        // semantic validation
                        diagnostics.extend(self.validate_semantics(&ast));
                    }
                    Err(parse_error) => {
                        diagnostics.push(self.parser_error_to_diagnostic(parse_error));
                    }
                }
            }
            Err(lex_error) => {
                diagnostics.push(self.lexer_error_to_diagnostic(lex_error));
            }
        }

        diagnostics
    }

    pub fn get_completions(&self, content: &str, position: Position) -> Vec<CompletionItem> {
        let mut completions = Vec::new();

        // empty content 
        if content.is_empty() {
            completions.extend(self.get_instruction_completions());
            return completions;
        }

        // current line content for context analysis
        let lines: Vec<&str> = content.lines().collect();
        
        if position.line as usize >= lines.len() {
            // position is beyond existing lines - suggest instructions for new line
            completions.extend(self.get_instruction_completions());
            return completions;
        }

        let current_line = lines[position.line as usize];
        let cursor_pos = position.character as usize;
        
        // get text before cursor for context
        let text_before_cursor = if cursor_pos <= current_line.len() {
            &current_line[..cursor_pos]
        } else {
            current_line
        };

        let completion_context = self.analyze_completion_context(text_before_cursor);

        match completion_context {
            CompletionContext::Instruction => {
                completions.extend(self.get_instruction_completions());
            }
            CompletionContext::Operand => {
                completions.extend(self.get_operand_completions(content));
            }
            CompletionContext::Label => {
                completions.extend(self.get_label_completions(content));
            }
            CompletionContext::Unknown => {
                // default
                completions.extend(self.get_instruction_completions());
            }
        }

        completions
}

    fn analyze_completion_context(&self, text_before_cursor: &str) -> CompletionContext {
        let trimmed = text_before_cursor.trim();
        
        // line is empty or ends with label: -> suggest instructions
        if trimmed.is_empty() || trimmed.ends_with(':') {
            return CompletionContext::Instruction;
        }

        // after an instruction -> suggest operands
        let words: Vec<&str> = trimmed.split_whitespace().collect();
        if words.len() >= 1 {
            let first_word = words[0];
            if self.is_valid_instruction(first_word) && words.len() == 1 {
                return CompletionContext::Operand;
            }
        }

        // in the middle of typing something -> check context
        if trimmed.chars().all(|c| c.is_uppercase() || c == '_') {
            return CompletionContext::Instruction;
        }

        CompletionContext::Unknown
    }

    fn get_instruction_completions(&self) -> Vec<CompletionItem> {
        let instructions = [
            // Arithmetic instructions
            ("DOD", "Add", "DOD ${1:operand}", "Add value to accumulator: (AK) + (operand) → AK"),
            ("ODE", "Subtract", "ODE ${1:operand}", "Subtract value from accumulator: (AK) - (operand) → AK"),
            
            // Memory instructions
            ("POB", "Load", "POB ${1:operand}", "Load value into accumulator: (operand) → AK"),
            ("ŁAD", "Store", "ŁAD ${1:address}", "Store accumulator to memory: (AK) → (address)"),
            
            // Control flow instructions
            ("SOB", "Jump", "SOB ${1:label}", "Unconditional jump to label"),
            ("SOM", "Jump if negative", "SOM ${1:label}", "Jump to label if AK < 0"),
            ("SOZ", "Jump if zero", "SOZ ${1:label}", "Jump to label if AK = 0"),
            ("STP", "Stop", "STP", "Stop program execution"),
            
            // Stack instructions
            ("SDP", "Push", "SDP", "Push accumulator to stack: (AK) → stack"),
            ("PZS", "Pop", "PZS", "Pop from stack to accumulator: stack → AK"),
            
            // Interrupt instructions
            ("DNS", "Disable interrupts", "DNS", "Disable interrupt handling"),
            ("CZM", "Clear interrupt mask", "CZM", "Clear interrupt mask register"),
            ("MSK", "Set interrupt mask", "MSK ${1:mask}", "Set interrupt mask register"),
            ("PWR", "Return from interrupt", "PWR", "Return from interrupt handler"),
            
            // I/O instructions
            ("WEJSCIE", "Input", "WEJSCIE", "Input value from user"),
            ("WYJSCIE", "Output", "WYJSCIE", "Output accumulator value"),
        ];

        let mut completions = Vec::new();

        for (name, kind, snippet, description) in instructions {
            completions.push(CompletionItem {
                label: name.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(kind.to_string()),
                documentation: Some(Documentation::String(description.to_string())),
                insert_text: Some(snippet.to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                sort_text: Some(format!("1_{}", name)), // priority sorting
                ..Default::default()
            });
        }

        // Extended instructions
        let extended_instructions = [
            ("MNO", "Multiply", "MNO ${1:operand}", "Multiply: (AK) * (operand) → AK [Extended]"),
            ("DZI", "Divide", "DZI ${1:operand}", "Divide: (AK) / (operand) → AK [Extended]"),
            ("MOD", "Modulo", "MOD ${1:operand}", "Modulo: (AK) % (operand) → AK [Extended]"),
        ];

        for (name, kind, snippet, description) in extended_instructions {
            completions.push(CompletionItem {
                label: format!("{} (Extended)", name),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(kind.to_string()),
                documentation: Some(Documentation::String(description.to_string())),
                insert_text: Some(snippet.to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                sort_text: Some(format!("2_{}", name)), // lower priority
                ..Default::default()
            });
        }

        completions
    }

    fn get_operand_completions(&self, _content: &str) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "#immediate".to_string(),
                kind: Some(CompletionItemKind::CONSTANT),
                detail: Some("Immediate value".to_string()),
                documentation: Some(Documentation::String("Use immediate value (e.g., #42, #0xFF)".to_string())),
                insert_text: Some("#${1:value}".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            // TODO: label completions
        ]
    }

    fn get_label_completions(&self, _content: &str) -> Vec<CompletionItem> {
        // TODO: parsing content for existing labels
        Vec::new()
    }

    fn is_valid_instruction(&self, word: &str) -> bool {
        let valid_instructions = [
            "DOD", "ODE", "LAD", "POB", "SOB", "SOM", "STP", 
            "DNS", "PZS", "SDP", "CZM", "MSK", "PWR", 
            "WEJSCIE", "WYJSCIE", "SOZ", "MNO", "DZI", "MOD"
        ];
        valid_instructions.contains(&word)
    }

    fn validate_semantics(&self, ast: &parseid::Program) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for element in &ast.elements {
            match element {
                parseid::ProgramElement::Instruction(instruction) => {
                    if let Err(diagnostic) = self.validate_instruction(instruction) {
                        diagnostics.push(diagnostic);
                    }
                }
                parseid::ProgramElement::MacroCall(macro_call) => {
                    if let Err(diagnostic) = self.validate_macro_call(macro_call) {
                        diagnostics.push(diagnostic);
                    }
                }
                _ => {}
            }
        }

        diagnostics
    }

    fn validate_instruction(&self, instruction: &parseid::Instruction) -> Result<(), Diagnostic> {
        let opcode_str = &instruction.opcode;
        
        let valid_instructions = [
            "DOD", "ODE", "LAD", "POB", "SOB", "SOM", "STP", 
            "DNS", "PZS", "SDP", "CZM", "MSK", "PWR", 
            "WEJSCIE", "WYJSCIE", "SOZ", "MNO", "DZI", "MOD"
        ];

        if !valid_instructions.contains(&opcode_str.as_str()) {
            return Err(Diagnostic {
                range: Range {
                    start: Position {
                        line: (instruction.line.saturating_sub(1)) as u32,
                        character: (instruction.column.saturating_sub(1)) as u32,
                    },
                    end: Position {
                        line: (instruction.line.saturating_sub(1)) as u32,
                        character: (instruction.column + opcode_str.len()) as u32,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("SEM001".to_string())),
                source: Some("asmodeus-lsp".to_string()),
                message: format!("Unknown instruction: '{}'", opcode_str),
                related_information: None,
                tags: None,
                code_description: None,
                data: None,
            });
        }

        Ok(())
    }

    fn validate_macro_call(&self, macro_call: &parseid::MacroCall) -> Result<(), Diagnostic> {
        let name = &macro_call.name;
        
        if name.chars().all(|c| c.is_uppercase() || c == '_') {
            return Err(Diagnostic {
                range: Range {
                    start: Position {
                        line: (macro_call.line.saturating_sub(1)) as u32,
                        character: (macro_call.column.saturating_sub(1)) as u32,
                    },
                    end: Position {
                        line: (macro_call.line.saturating_sub(1)) as u32,
                        character: (macro_call.column + name.len()) as u32,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("SEM002".to_string())),
                source: Some("asmodeus-lsp".to_string()),
                message: format!("Unknown instruction or undefined macro: '{}'", name),
                related_information: None,
                tags: None,
                code_description: None,
                data: None,
            });
        }

        Ok(())
    }

    fn lexer_error_to_diagnostic(&self, error: lexariel::LexerError) -> Diagnostic {
        Diagnostic {
            range: Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 0, character: 1 },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String("LEX001".to_string())),
            source: Some("asmodeus-lsp".to_string()),
            message: format!("Lexer error: {}", error),
            related_information: None,
            tags: None,
            code_description: None,
            data: None,
        }
    }

    fn parser_error_to_diagnostic(&self, error: parseid::ParserError) -> Diagnostic {
        Diagnostic {
            range: Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 0, character: 1 },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String("PAR001".to_string())),
            source: Some("asmodeus-lsp".to_string()),
            message: format!("Parser error: {}", error),
            related_information: None,
            tags: None,
            code_description: None,
            data: None,
        }
    }

fn get_label_info(&self, word: &str, content: &str) -> Option<String> {
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

pub fn get_definition(&self, content: &str, position: Position, uri: &Url) -> Option<GotoDefinitionResponse> {
    let lines: Vec<&str> = content.lines().collect();
    
    if position.line as usize >= lines.len() {
        return None;
    }

    let current_line = lines[position.line as usize];
    let cursor_pos = position.character as usize;

    let word_info = self.get_word_at_position(current_line, cursor_pos)?;
    let (word, _, _) = word_info;

    self.find_label_definition(&word, content, uri)
}

fn find_label_definition(&self, label: &str, content: &str, uri: &Url) -> Option<GotoDefinitionResponse> {
    let lines: Vec<&str> = content.lines().collect();
    
    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("{}:", label)) {
            let location = Location {
                uri: uri.clone(),
                range: Range {
                    start: Position {
                        line: line_num as u32,
                        character: 0,
                    },
                    end: Position {
                        line: line_num as u32,
                        character: trimmed.len() as u32,
                    },
                },
            };
            return Some(GotoDefinitionResponse::Scalar(location));
        }
    }
    None
}

    
pub fn get_hover_info(&self, content: &str, position: Position) -> Option<Hover> {
    let lines: Vec<&str> = content.lines().collect();
    
    if position.line as usize >= lines.len() {
        return None;
    }

    let current_line = lines[position.line as usize];
    let cursor_pos = position.character as usize;

    let word_info = self.get_word_at_position(current_line, cursor_pos)?;
    let (word, start_pos, end_pos) = word_info;

    // instruction info first
    let hover_content = if let Some(instruction_info) = self.get_instruction_info(&word) {
        instruction_info
    } else if let Some(label_info) = self.get_label_info(&word, content) {
        label_info
    } else {
        return None;
    };

    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: hover_content,
        }),
        range: Some(Range {
            start: Position {
                line: position.line,
                character: start_pos as u32,
            },
            end: Position {
                line: position.line,
                character: end_pos as u32,
            },
        }),
    })
}

fn get_word_at_position(&self, line: &str, cursor_pos: usize) -> Option<(String, usize, usize)> {
    if cursor_pos > line.len() {
        return None;
    }

    let chars: Vec<char> = line.chars().collect();
    
    let mut start = cursor_pos;
    while start > 0 && self.is_word_char(chars.get(start - 1).copied().unwrap_or(' ')) {
        start -= 1;
    }

    let mut end = cursor_pos;
    while end < chars.len() && self.is_word_char(chars.get(end).copied().unwrap_or(' ')) {
        end += 1;
    }

    if start == end {
        return None;
    }

    let word: String = chars[start..end].iter().collect();
    Some((word, start, end))
}

fn is_word_char(&self, c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == 'Ł'
}

fn get_instruction_info(&self, word: &str) -> Option<String> {
    match word {
        "DOD" => Some("**DOD** - Add\n\n**Operation:** `(AK) + (operand) → AK`\n\n**Description:** Adds the value to the accumulator.".to_string()),
        "ODE" => Some("**ODE** - Subtract\n\n**Operation:** `(AK) - (operand) → AK`\n\n**Description:** Subtracts the value from the accumulator.".to_string()),
        "POB" => Some("**POB** - Load\n\n**Operation:** `(operand) → AK`\n\n**Description:** Loads a value into the accumulator.".to_string()),
        "ŁAD" | "LAD" => Some("**ŁAD** - Store\n\n**Operation:** `(AK) → (address)`\n\n**Description:** Stores accumulator to memory.".to_string()),
        "SOB" => Some("**SOB** - Jump\n\n**Operation:** Unconditional jump\n\n**Description:** Jumps to the specified label.".to_string()),
        "SOM" => Some("**SOM** - Jump if negative\n\n**Operation:** Jump if `AK < 0`\n\n**Description:** Conditional jump when accumulator is negative.".to_string()),
        "SOZ" => Some("**SOZ** - Jump if zero\n\n**Operation:** Jump if `AK = 0`\n\n**Description:** Conditional jump when accumulator is zero.".to_string()),
        "STP" => Some("**STP** - Stop\n\n**Operation:** Halt execution\n\n**Description:** Stops the program.".to_string()),
        "WEJSCIE" => Some("**WEJSCIE** - Input\n\n**Description:** Reads user input into accumulator.".to_string()),
        "WYJSCIE" => Some("**WYJSCIE** - Output\n\n**Description:** Outputs accumulator value.".to_string()),
        "MNO" => Some("**MNO** - Multiply *(Extended)*\n\n**Operation:** `(AK) * (operand) → AK`\n\n**Note:** Requires `--extended` flag.".to_string()),
        "DZI" => Some("**DZI** - Divide *(Extended)*\n\n**Operation:** `(AK) / (operand) → AK`\n\n**Note:** Requires `--extended` flag.".to_string()),
        "MOD" => Some("**MOD** - Modulo *(Extended)*\n\n**Operation:** `(AK) % (operand) → AK`\n\n**Note:** Requires `--extended` flag.".to_string()),
        _ => None,
    }
}

pub fn find_references(&self, content: &str, position: Position, uri: &Url, include_declaration: bool) -> Vec<Location> {
    let mut locations = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    
    if position.line as usize >= lines.len() {
        return locations;
    }

    let current_line = lines[position.line as usize];
    let cursor_pos = position.character as usize;

    let word_info = match self.get_word_at_position(current_line, cursor_pos) {
        Some(info) => info,
        None => return locations,
    };
    let (word, _, _) = word_info;

    // all occurrences of this symbol
    for (line_num, line) in lines.iter().enumerate() {
        let mut search_pos = 0;
        while let Some(pos) = line[search_pos..].find(&word) {
            let actual_pos = search_pos + pos;
            
            // whole word match
            if self.is_whole_word_match(line, actual_pos, &word) {
                // skip declaration if not requested
                if !include_declaration && self.is_label_declaration(line, actual_pos, &word) {
                    search_pos = actual_pos + word.len();
                    continue;
                }

                locations.push(Location {
                    uri: uri.clone(),
                    range: Range {
                        start: Position {
                            line: line_num as u32,
                            character: actual_pos as u32,
                        },
                        end: Position {
                            line: line_num as u32,
                            character: (actual_pos + word.len()) as u32,
                        },
                    },
                });
            }
            search_pos = actual_pos + word.len();
        }
    }

    locations
}

fn is_whole_word_match(&self, line: &str, pos: usize, word: &str) -> bool {
    let chars: Vec<char> = line.chars().collect();
    
    // character before
    if pos > 0 {
        if let Some(prev_char) = chars.get(pos - 1) {
            if self.is_word_char(*prev_char) {
                return false;
            }
        }
    }
    
    // character after
    let end_pos = pos + word.len();
    if let Some(next_char) = chars.get(end_pos) {
        if self.is_word_char(*next_char) {
            return false;
        }
    }
    
    true
}

fn is_label_declaration(&self, line: &str, pos: usize, word: &str) -> bool {
    let after_word_pos = pos + word.len();
    line.chars().nth(after_word_pos) == Some(':')
}

pub fn get_document_symbols(&self, content: &str) -> Vec<SymbolInformation> {
    let mut symbols = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    
    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        // labels
        if let Some(colon_pos) = trimmed.find(':') {
            let label_name = &trimmed[..colon_pos];
            if !label_name.is_empty() && label_name.chars().all(|c| self.is_word_char(c)) {
                symbols.push(SymbolInformation {
                    name: label_name.to_string(),
                    kind: SymbolKind::FUNCTION, // as functions
                    tags: None,
                    deprecated: None,
                    location: Location {
                        uri: Url::parse("file:///dummy").unwrap(),
                        range: Range {
                            start: Position {
                                line: line_num as u32,
                                character: 0,
                            },
                            end: Position {
                                line: line_num as u32,
                                character: trimmed.len() as u32,
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


pub fn get_semantic_tokens(&self, content: &str) -> Vec<SemanticToken> {
    let mut tokens = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    
    for (line_num, line) in lines.iter().enumerate() {
        let line_tokens = self.tokenize_line(line, line_num as u32);
        tokens.extend(line_tokens);
    }
    
    // to delta encoding
    self.encode_semantic_tokens(tokens)
}

fn tokenize_line(&self, line: &str, line_num: u32) -> Vec<RawSemanticToken> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        if chars[i].is_whitespace() {
            i += 1;
            continue;
        }
        
        // comment
        if chars[i] == ';' {
            tokens.push(RawSemanticToken {
                line: line_num,
                start_char: i as u32,
                length: (chars.len() - i) as u32,
                token_type: 4, // COMMENT
                token_modifiers: 0,
            });
            break;
        }
        
        // operators
        if chars[i] == '#' {
            tokens.push(RawSemanticToken {
                line: line_num,
                start_char: i as u32,
                length: 1,
                token_type: 3, // OPERATOR
                token_modifiers: 0,
            });
            i += 1;
            continue;
        }
        
        if chars[i] == '[' || chars[i] == ']' {
            tokens.push(RawSemanticToken {
                line: line_num,
                start_char: i as u32,
                length: 1,
                token_type: 3, // OPERATOR
                token_modifiers: 0,
            });
            i += 1;
            continue;
        }
        
        // word/number
        let start = i;
        while i < chars.len() && self.is_word_char(chars[i]) {
            i += 1;
        }
        
        if start < i {
            let word: String = chars[start..i].iter().collect();
            let token_type = self.classify_token(&word, line, start);
            
            tokens.push(RawSemanticToken {
                line: line_num,
                start_char: start as u32,
                length: (i - start) as u32,
                token_type,
                token_modifiers: 0,
            });
        } else {
            i += 1; // skip unknown char
        }
    }
    
    tokens
}

fn classify_token(&self, word: &str, line: &str, position: usize) -> u32 {
    // number
    if word.chars().all(|c| c.is_ascii_digit()) || 
       word.starts_with("0x") || word.starts_with("0b") {
        return 2; // NUMBER
    }
    
    // instruction
    if self.is_valid_instruction(word) {
        return 0; // KEYWORD
    }
    
    // label (followed by :)
    let after_word = position + word.len();
    if let Some(next_char) = line.chars().nth(after_word) {
        if next_char == ':' {
            return 1; // FUNCTION (label definition)
        }
    }
    
    // label reference
    if self.could_be_label(word) {
        return 1; // FUNCTION (label reference)
    }
    
    2 // default to NUMBER/IDENTIFIER
}

fn could_be_label(&self, word: &str) -> bool {
    !word.chars().all(|c| c.is_uppercase() || c.is_ascii_digit() || c == '_')
}

fn encode_semantic_tokens(&self, tokens: Vec<RawSemanticToken>) -> Vec<SemanticToken> {
    let mut encoded = Vec::new();
    let mut prev_line = 0;
    let mut prev_char = 0;
    
    for token in tokens {
        let delta_line = token.line - prev_line;
        let delta_char = if delta_line == 0 {
            token.start_char - prev_char
        } else {
            token.start_char
        };
        
        encoded.push(SemanticToken {
            delta_line,
            delta_start: delta_char,
            length: token.length,
            token_type: token.token_type,
            token_modifiers_bitset: token.token_modifiers,
        });
        
        prev_line = token.line;
        prev_char = token.start_char;
    }
    
    encoded
}


pub fn get_code_actions(&self, content: &str, range: Range, uri: &Url, context: &CodeActionContext) -> Vec<CodeActionOrCommand> {
    let mut actions = Vec::new();
    
    // quick fixes for diagnostics
    for diagnostic in &context.diagnostics {
        if let Some(action) = self.create_quick_fix(diagnostic, content, uri) {
            actions.push(action);
        }
    }
    
    actions.extend(self.get_refactoring_actions(content, range, uri));
    
    actions
}

fn create_quick_fix(&self, diagnostic: &Diagnostic, content: &str, uri: &Url) -> Option<CodeActionOrCommand> {
    // unknown instruction errors
    if diagnostic.message.contains("Unknown instruction") {
        return self.suggest_instruction_correction(diagnostic, content, uri);
    }
    
    // undefined macro errors  
    if diagnostic.message.contains("undefined macro") {
        return self.suggest_instruction_correction(diagnostic, content, uri);
    }
    
    None
}

fn suggest_instruction_correction(&self, diagnostic: &Diagnostic, content: &str, uri: &Url) -> Option<CodeActionOrCommand> {
    // unknown instruction from diagnostic message
    let message = &diagnostic.message;
    let start_quote = message.find('\'')?;
    let end_quote = message.rfind('\'')?;
    if start_quote >= end_quote {
        return None;
    }
    
    let unknown_instruction = &message[start_quote + 1..end_quote];
    
    // similar instructions
    let suggestions = self.find_similar_instructions(unknown_instruction);
    
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

fn find_similar_instructions(&self, unknown: &str) -> Vec<String> {
    let valid_instructions = [
        "DOD", "ODE", "ŁAD", "POB", "SOB", "SOM", "STP", 
        "DNS", "PZS", "SDP", "CZM", "MSK", "PWR", 
        "WEJSCIE", "WYJSCIE", "SOZ", "MNO", "DZI", "MOD"
    ];
    
    let mut suggestions = Vec::new();
    
    // instructions with similar spelling (levenshtein)
    for &instruction in &valid_instructions {
        let distance = self.levenshtein_distance(unknown.to_uppercase().as_str(), instruction);
        if distance <= 2 { // 2 character differences
            suggestions.push(instruction.to_string());
        }
    }
    
    // similarity (distance)
    suggestions.sort_by_key(|s| self.levenshtein_distance(unknown.to_uppercase().as_str(), s));
    
    suggestions
}

fn levenshtein_distance(&self, a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();
    
    let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];
    
    // first row and column
    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    for j in 0..=b_len {
        matrix[0][j] = j;
    }
    
    // matrix
    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i - 1][j] + 1,     // deletion
                    matrix[i][j - 1] + 1      // insertion
                ),
                matrix[i - 1][j - 1] + cost   // substitution
            );
        }
    }
    
    matrix[a_len][b_len]
}

fn get_refactoring_actions(&self, content: &str, range: Range, uri: &Url) -> Vec<CodeActionOrCommand> {
    let mut actions = Vec::new();
    
    // "Convert to uppercase" 
    if let Some(action) = self.create_uppercase_action(content, range, uri) {
        actions.push(action);
    }
    
    // "Add comment" 
    if let Some(action) = self.create_add_comment_action(content, range, uri) {
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
    
    // lowercase instruction
    if self.is_valid_instruction(&selected_text.to_uppercase()) && selected_text != selected_text.to_uppercase() {
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
    
    // add comment if line doesnt have one
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

}

#[derive(Debug, Clone)]
enum CompletionContext {
    Instruction,
    Operand,
    Label,
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_assembly_no_diagnostics() {
        let analyzer = SemanticAnalyzer::new();
        let content = r#"
start:
    POB #42
    WYJSCIE
    STP
"#;
        let uri = Url::parse("file:///test.asmod").unwrap();
        let diagnostics = analyzer.analyze_document(content, &uri);
        
        assert!(diagnostics.is_empty(), "Valid assembly should produce no diagnostics");
    }

    #[test]
    fn test_invalid_syntax_produces_diagnostic() {
        let analyzer = SemanticAnalyzer::new();
        
        let content = r#"
start:
    INVALID_INSTRUCTION
"#;
        let uri = Url::parse("file:///test.asmod").unwrap();
        let diagnostics = analyzer.analyze_document(content, &uri);
        
        assert!(!diagnostics.is_empty(), "Invalid syntax should produce diagnostics");
        assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::ERROR));
        assert!(diagnostics[0].message.contains("Unknown instruction") || 
                diagnostics[0].message.contains("undefined macro"));
    }

    #[test]
    fn test_definitely_invalid_syntax() {
        let analyzer = SemanticAnalyzer::new();
        
        let content = "@@@ invalid content $$$";
        let uri = Url::parse("file:///test.asmod").unwrap();
        let diagnostics = analyzer.analyze_document(content, &uri);
        
        assert!(!diagnostics.is_empty(), "Definitely invalid syntax should produce diagnostics");
    }

    #[test]
    fn test_instruction_completions() {
        let analyzer = SemanticAnalyzer::new();
        let content = "";
        let position = Position { line: 0, character: 0 };

        let completions = analyzer.get_completions(content, position);

        assert!(!completions.is_empty(), "Should return instruction completions");

        // if common instructions are present
        let instruction_names: Vec<&str> = completions.iter()
            .map(|c| c.label.as_str())
            .collect();

        assert!(instruction_names.iter().any(|&name| name.contains("POB")));
        assert!(instruction_names.iter().any(|&name| name.contains("DOD")));
        assert!(instruction_names.iter().any(|&name| name.contains("STP")));
    }

#[test]
fn test_hover_instruction() {
    let analyzer = SemanticAnalyzer::new();
    let content = "POB #42";
    let position = Position { line: 0, character: 1 };
    
    let hover = analyzer.get_hover_info(content, position);
    
    assert!(hover.is_some(), "Should return hover info for POB instruction");
}

#[test]
fn test_goto_definition() {
    let analyzer = SemanticAnalyzer::new();
    let content = r#"start:
    POB #42
    SOB start"#;
    let position = Position { line: 2, character: 8 }; // 'start' w SOB
    let uri = Url::parse("file:///test.asmod").unwrap();
    
    let definition = analyzer.get_definition(content, position, &uri);
    
    assert!(definition.is_some(), "Should find label definition");
}

#[test]
fn test_find_references() {
    let analyzer = SemanticAnalyzer::new();
    let content = r#"start:
    POB #42
    SOB start
loop:
    DOD start
    SOB loop"#;
    let position = Position { line: 0, character: 0 }; // na definicji 'start'
    let uri = Url::parse("file:///test.asmod").unwrap();
    
    let references = analyzer.find_references(content, position, &uri, true);
    
    assert_eq!(references.len(), 3, "Should find 3 references to 'start'");
}

#[test]
fn test_document_symbols() {
    let analyzer = SemanticAnalyzer::new();
    let content = r#"start:
    POB #42
loop:
    SOB start"#;
    
    let symbols = analyzer.get_document_symbols(content);
    
    assert_eq!(symbols.len(), 2, "Should find 2 symbols");
    assert!(symbols.iter().any(|s| s.name == "start"));
    assert!(symbols.iter().any(|s| s.name == "loop"));
}

#[test]
fn test_semantic_tokens() {
    let analyzer = SemanticAnalyzer::new();
    let content = r#"start:
    POB #42    ; load immediate
    SOB start  ; jump to start"#;
    
    let tokens = analyzer.get_semantic_tokens(content);
    
    assert!(!tokens.is_empty(), "Should generate semantic tokens");
}

#[test]
fn test_code_actions_instruction_suggestion() {
    let analyzer = SemanticAnalyzer::new();
    let content = "DOX #42"; // typo in DOD
    let uri = Url::parse("file:///test.asmod").unwrap();
    
    // diagnostic for unknown instruction
    let diagnostic = Diagnostic {
        range: Range {
            start: Position { line: 0, character: 0 },
            end: Position { line: 0, character: 3 },
        },
        severity: Some(DiagnosticSeverity::ERROR),
        message: "Unknown instruction: 'DOX'".to_string(),
        source: Some("asmodeus-lsp".to_string()),
        ..Default::default()
    };
    
    let context = CodeActionContext {
        diagnostics: vec![diagnostic],
        only: None,
        trigger_kind: None,
    };
    
    let actions = analyzer.get_code_actions(content, Range::default(), &uri, &context);
    
    assert!(!actions.is_empty(), "Should suggest code actions for typos");
}

}

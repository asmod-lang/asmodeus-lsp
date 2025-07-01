use tower_lsp::lsp_types::*;

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
}

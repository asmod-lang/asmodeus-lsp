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

    fn validate_semantics(&self, ast: &parseid::Program) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for element in &ast.elements {
            match element {
                parseid::ProgramElement::Instruction(instruction) => {
                    // instruction is valid?
                    if let Err(diagnostic) = self.validate_instruction(instruction) {
                        diagnostics.push(diagnostic);
                    }
                }
                parseid::ProgramElement::MacroCall(macro_call) => {
                    // invalid instruction being parsed as macro?
                    if let Err(diagnostic) = self.validate_macro_call(macro_call) {
                        diagnostics.push(diagnostic);
                    }
                }
                _ => {
                    // other elements (labels, data) are fine for now
                }
            }
        }

        diagnostics
    }

    fn validate_instruction(&self, instruction: &parseid::Instruction) -> Result<(), Diagnostic> {
        // instruction opcode
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
        // assume all macro calls that look like instructions are invalid
        let name = &macro_call.name;
        
        // if it looks like an instruction but isnt valid -> report error
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
        
        // that should definitely fail at lexer level
        let content = "@@@ invalid content $$$";
        let uri = Url::parse("file:///test.asmod").unwrap();
        let diagnostics = analyzer.analyze_document(content, &uri);
        
        assert!(!diagnostics.is_empty(), "Definitely invalid syntax should produce diagnostics");
    }
}

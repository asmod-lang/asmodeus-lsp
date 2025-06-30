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
                    Ok(_ast) => {
                        // success - no syntax errors
                        // TODO: semantic validation here later
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

    fn lexer_error_to_diagnostic(&self, error: lexariel::LexerError) -> Diagnostic {
        // use default position
        Diagnostic {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 1,
                },
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
        // use default position
        Diagnostic {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 1,
                },
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
    }
}

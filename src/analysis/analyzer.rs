use crate::error::{LspError, LspResult};
use lexariel::{tokenize, LexerError};
use parseid::{parse, ParserError};
use tower_lsp::lsp_types::*;

pub struct SemanticAnalyzer;

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze_document(&self, content: &str, uri: &Url) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // 1: lexariel
        match tokenize(content) {
            Ok(tokens) => {
                // 2: parseid
                match parse(tokens) {
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

    fn lexer_error_to_diagnostic(&self, error: LexerError) -> Diagnostic {
        Diagnostic {
            range: Range {
                start: Position {
                    line: (error.line().saturating_sub(1)) as u32, // LSP 0-based
                    character: (error.column().saturating_sub(1)) as u32,
                },
                end: Position {
                    line: (error.line().saturating_sub(1)) as u32,
                    character: error.column() as u32, // end one character after start
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

    fn parser_error_to_diagnostic(&self, error: ParserError) -> Diagnostic {
        // position from parser error if available
        let (line, column) = match &error {
            ParserError::UnexpectedToken { line, column, .. } => (*line, *column),
            ParserError::UnexpectedEndOfInput { line, column } => (*line, *column),
            _ => (1, 1), // default position fallback 
        };

        Diagnostic {
            range: Range {
                start: Position {
                    line: (line.saturating_sub(1)) as u32,
                    character: (column.saturating_sub(1)) as u32,
                },
                end: Position {
                    line: (line.saturating_sub(1)) as u32,
                    character: column as u32,
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

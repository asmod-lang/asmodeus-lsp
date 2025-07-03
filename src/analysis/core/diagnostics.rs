use crate::analysis::language::ValidationProvider;
use crate::analysis::utils::{
    create_parser_diagnostic, 
};
use tower_lsp::lsp_types::*;

#[derive(Debug)]
pub struct DiagnosticsEngine {
    validation_provider: ValidationProvider,
}

impl DiagnosticsEngine {
    pub fn new() -> Self {
        Self {
            validation_provider: ValidationProvider::new(),
        }
    }

    /// returns diagnostics
    pub fn analyze_document(&self, content: &str, _uri: &Url) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        match lexariel::tokenize(content) {
            Ok(tokens) => {
                // tokens into AST
                match parseid::parse(tokens) {
                    Ok(ast) => {
                        diagnostics.extend(self.validation_provider.validate_semantics(&ast));
                    }
                    Err(parse_error) => {
                        diagnostics.push(self.parser_error_to_diagnostic(parse_error));
                    }
                }
            }
            Err(lexer_error) => {
                diagnostics.push(self.lexer_error_to_diagnostic(lexer_error));
            }
        }

        diagnostics
    }

    fn lexer_error_to_diagnostic(&self, error: lexariel::LexerError) -> Diagnostic {
        create_parser_diagnostic(format!("Lexer error: {}", error), "LEX001")
    }

    fn parser_error_to_diagnostic(&self, error: parseid::ParserError) -> Diagnostic {
        create_parser_diagnostic(format!("Parser error: {}", error), "PAR001")
    }
}

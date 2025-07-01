use tower_lsp::lsp_types::*;
use crate::analysis::utils::{InstructionDatabase, create_parser_diagnostic, create_semantic_diagnostic};

#[derive(Debug)]
pub struct DiagnosticsEngine {
    instruction_db: InstructionDatabase,
}

impl DiagnosticsEngine {
    pub fn new() -> Self {
        Self {
            instruction_db: InstructionDatabase::new(),
        }
    }

    /// returns diagnostics
    pub fn analyze_document(&self, content: &str, _uri: &Url) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        match lexariel::tokenize(content) {
            Ok(tokens) => {
                match parseid::parse(tokens) {
                    Ok(ast) => {
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

    /// AST semantic validation
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

    /// single instruction
    fn validate_instruction(&self, instruction: &parseid::Instruction) -> Result<(), Diagnostic> {
        let opcode_str = &instruction.opcode;
        
        if !self.instruction_db.is_valid_instruction(opcode_str) {
            return Err(create_semantic_diagnostic(
                instruction.line,
                instruction.column,
                opcode_str.len(),
                "SEM001",
                format!("Unknown instruction: '{}'", opcode_str),
            ));
        }

        Ok(())
    }

    fn validate_macro_call(&self, macro_call: &parseid::MacroCall) -> Result<(), Diagnostic> {
        let name = &macro_call.name;
        
        if name.chars().all(|c| c.is_uppercase() || c == '_') {
            return Err(create_semantic_diagnostic(
                macro_call.line,
                macro_call.column,
                name.len(),
                "SEM002",
                format!("Unknown instruction or undefined macro: '{}'", name),
            ));
        }

        Ok(())
    }

    fn lexer_error_to_diagnostic(&self, error: lexariel::LexerError) -> Diagnostic {
        create_parser_diagnostic(
            format!("Lexer error: {}", error),
            "LEX001",
        )
    }

    fn parser_error_to_diagnostic(&self, error: parseid::ParserError) -> Diagnostic {
        create_parser_diagnostic(
            format!("Parser error: {}", error),
            "PAR001",
        )
    }
}

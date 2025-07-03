use tower_lsp::lsp_types::*;
use crate::analysis::utils::{InstructionDatabase, create_semantic_diagnostic};

#[derive(Debug)]
pub struct ValidationProvider {
    instruction_db: InstructionDatabase,
}

impl ValidationProvider {
    pub fn new() -> Self {
        Self {
            instruction_db: InstructionDatabase::new(),
        }
    }

    /// AST semantic
    pub fn validate_semantics(&self, ast: &parseid::Program) -> Vec<Diagnostic> {
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

        // external validation - whether instruction has proper operands 
        if let Some(instruction_info) = self.instruction_db.get_instruction(opcode_str) {
            self.validate_instruction_operands(instruction, instruction_info)?;
        }

        Ok(())
    }

    fn validate_instruction_operands(
        &self, 
        instruction: &parseid::Instruction, 
        instruction_info: &crate::analysis::utils::InstructionInfo
    ) -> Result<(), Diagnostic> {
        match instruction_info.operand_type {
            crate::analysis::utils::OperandType::None => {
                // should not have operands
                if instruction.operand.is_some() {
                    return Err(create_semantic_diagnostic(
                        instruction.line,
                        instruction.column + instruction.opcode.len(),
                        1,
                        "SEM003",
                        format!("Instruction '{}' does not take operands", instruction.opcode),
                    ));
                }
            },
            _ => {
                // should have operands 
                if instruction.operand.is_none() {
                    return Err(create_semantic_diagnostic(
                        instruction.line,
                        instruction.column + instruction.opcode.len(),
                        1,
                        "SEM004",
                        format!("Instruction '{}' requires an operand", instruction.opcode),
                    ));
                }
            }
        }

        Ok(())
    }

    fn validate_macro_call(&self, macro_call: &parseid::MacroCall) -> Result<(), Diagnostic> {
        let name = &macro_call.name;
        
        // whether is not an instruction used as macro
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

    pub fn validate_symbol_usage(&self, content: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            // forbidden symbols check
            if let Some(diagnostic) = self.check_invalid_characters(line, line_num) {
                diagnostics.push(diagnostic);
            }

            // valid label naming check
            if let Some(diagnostic) = self.check_label_naming(line, line_num) {
                diagnostics.push(diagnostic);
            }
        }

        diagnostics
    }

    fn check_invalid_characters(&self, line: &str, line_num: usize) -> Option<Diagnostic> {
        // outside comments
        if let Some(comment_pos) = line.find(';') {
            let code_part = &line[..comment_pos];
            self.find_invalid_chars_in_code(code_part, line_num)
        } else {
            self.find_invalid_chars_in_code(line, line_num)
        }
    }

    fn find_invalid_chars_in_code(&self, code: &str, line_num: usize) -> Option<Diagnostic> {
        for (char_pos, ch) in code.char_indices() {
            if !ch.is_alphanumeric() && ch != '_' && ch != 'Ł' && ch != ' ' && ch != '\t' 
                && ch != '#' && ch != '[' && ch != ']' && ch != ':' {
                return Some(create_semantic_diagnostic(
                    line_num + 1,
                    char_pos + 1,
                    1,
                    "SEM005",
                    format!("Invalid character '{}' in code", ch),
                ));
            }
        }
        None
    }

    fn check_label_naming(&self, line: &str, line_num: usize) -> Option<Diagnostic> {
        let trimmed = line.trim();
        
        if let Some(colon_pos) = trimmed.find(':') {
            let label_name = &trimmed[..colon_pos];
            
            // valid label naming check
            if !crate::analysis::utils::is_valid_symbol_name(label_name) {
                let label_start = line.len() - trimmed.len();
                return Some(create_semantic_diagnostic(
                    line_num + 1,
                    label_start + 1,
                    label_name.len(),
                    "SEM006",
                    format!("Invalid label name '{}'. Labels must start with a letter or underscore and contain only alphanumeric characters and underscores", label_name),
                ));
            }

            // whether label name is instruction name as well
            if self.instruction_db.is_valid_instruction(label_name) {
                let label_start = line.len() - trimmed.len();
                return Some(create_semantic_diagnostic(
                    line_num + 1,
                    label_start + 1,
                    label_name.len(),
                    "SEM007",
                    format!("Label name '{}' conflicts with instruction name", label_name),
                ));
            }
        }

        None
    }
}

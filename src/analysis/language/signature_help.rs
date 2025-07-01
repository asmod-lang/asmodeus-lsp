use tower_lsp::lsp_types::*;
use crate::analysis::utils::{InstructionDatabase, get_line_at_position, is_valid_position};

#[derive(Debug)]
pub struct SignatureHelpProvider {
    instruction_db: InstructionDatabase,
}

impl SignatureHelpProvider {
    pub fn new() -> Self {
        Self {
            instruction_db: InstructionDatabase::new(),
        }
    }

    pub fn get_signature_help(&self, content: &str, position: Position) -> Option<SignatureHelp> {
        if !is_valid_position(content, position) {
            return None;
        }

        let current_line = get_line_at_position(content, position)?;
        let cursor_pos = position.character as usize;
        
        let text_before_cursor = if cursor_pos <= current_line.len() {
            &current_line[..cursor_pos]
        } else {
            current_line
        };
        
        // instruction at the start of the line
        let trimmed = text_before_cursor.trim_start();
        let words: Vec<&str> = trimmed.split_whitespace().collect();
        
        if words.is_empty() {
            return None;
        }
        
        let instruction = words[0];
        if !self.instruction_db.is_valid_instruction(instruction) {
            return None;
        }
        
        // for instruction
        let signature_info = self.get_instruction_signature(instruction)?;
        
        Some(SignatureHelp {
            signatures: vec![signature_info],
            active_signature: Some(0),
            active_parameter: self.get_active_parameter(text_before_cursor, instruction),
        })
    }

    /// for instruction
    fn get_instruction_signature(&self, instruction: &str) -> Option<SignatureInformation> {
        let instruction_info = self.instruction_db.get_instruction(instruction)?;
        
        match instruction {
            "DOD" => Some(SignatureInformation {
                label: "DOD operand".to_string(),
                documentation: Some(Documentation::String("Add operand to accumulator: (AK) + (operand) → AK".to_string())),
                parameters: Some(vec![
                    ParameterInformation {
                        label: ParameterLabel::Simple("operand".to_string()),
                        documentation: Some(Documentation::String("Memory address, immediate value (#42), or label".to_string())),
                    }
                ]),
                active_parameter: None,
            }),
            "ODE" => Some(SignatureInformation {
                label: "ODE operand".to_string(),
                documentation: Some(Documentation::String("Subtract operand from accumulator: (AK) - (operand) → AK".to_string())),
                parameters: Some(vec![
                    ParameterInformation {
                        label: ParameterLabel::Simple("operand".to_string()),
                        documentation: Some(Documentation::String("Memory address, immediate value (#42), or label".to_string())),
                    }
                ]),
                active_parameter: None,
            }),
            "POB" => Some(SignatureInformation {
                label: "POB operand".to_string(),
                documentation: Some(Documentation::String("Load operand into accumulator: (operand) → AK".to_string())),
                parameters: Some(vec![
                    ParameterInformation {
                        label: ParameterLabel::Simple("operand".to_string()),
                        documentation: Some(Documentation::String("Memory address, immediate value (#42), or label".to_string())),
                    }
                ]),
                active_parameter: None,
            }),
            "ŁAD" | "LAD" => Some(SignatureInformation {
                label: "ŁAD address".to_string(),
                documentation: Some(Documentation::String("Store accumulator to address: (AK) → (address)".to_string())),
                parameters: Some(vec![
                    ParameterInformation {
                        label: ParameterLabel::Simple("address".to_string()),
                        documentation: Some(Documentation::String("Memory address or label where to store AK".to_string())),
                    }
                ]),
                active_parameter: None,
            }),
            "SOB" | "SOM" | "SOZ" => Some(SignatureInformation {
                label: format!("{} label", instruction),
                documentation: Some(Documentation::String(match instruction {
                    "SOB" => "Unconditional jump to label".to_string(),
                    "SOM" => "Jump to label if AK < 0".to_string(),
                    "SOZ" => "Jump to label if AK = 0".to_string(),
                    _ => "Jump instruction".to_string(),
                })),
                parameters: Some(vec![
                    ParameterInformation {
                        label: ParameterLabel::Simple("label".to_string()),
                        documentation: Some(Documentation::String("Target label or address to jump to".to_string())),
                    }
                ]),
                active_parameter: None,
            }),
            "MSK" => Some(SignatureInformation {
                label: "MSK mask".to_string(),
                documentation: Some(Documentation::String("Set interrupt mask register".to_string())),
                parameters: Some(vec![
                    ParameterInformation {
                        label: ParameterLabel::Simple("mask".to_string()),
                        documentation: Some(Documentation::String("8-bit interrupt mask value (0-255)".to_string())),
                    }
                ]),
                active_parameter: None,
            }),
            "MNO" => Some(SignatureInformation {
                label: "MNO operand".to_string(),
                documentation: Some(Documentation::String("Multiply accumulator by operand: (AK) * (operand) → AK [Extended]. Requires --extended flag".to_string())),
                parameters: Some(vec![
                    ParameterInformation {
                        label: ParameterLabel::Simple("operand".to_string()),
                        documentation: Some(Documentation::String("Value to multiply with. Requires --extended flag".to_string())),
                    }
                ]),
                active_parameter: None,
            }),
            "DZI" => Some(SignatureInformation {
                label: "DZI operand".to_string(),
                documentation: Some(Documentation::String("Divide accumulator by operand: (AK) / (operand) → AK [Extended]".to_string())),
                parameters: Some(vec![
                    ParameterInformation {
                        label: ParameterLabel::Simple("operand".to_string()),
                        documentation: Some(Documentation::String("Divisor value. Requires --extended flag".to_string())),
                    }
                ]),
                active_parameter: None,
            }),
            "MOD" => Some(SignatureInformation {
                label: "MOD operand".to_string(),
                documentation: Some(Documentation::String("Modulo operation: (AK) % (operand) → AK [Extended]".to_string())),
                parameters: Some(vec![
                    ParameterInformation {
                        label: ParameterLabel::Simple("operand".to_string()),
                        documentation: Some(Documentation::String("Modulo value. Requires --extended flag".to_string())),
                    }
                ]),
                active_parameter: None,
            }),
            _ => {
                match instruction_info.operand_type {
                    crate::analysis::utils::OperandType::None => Some(SignatureInformation {
                        label: instruction.to_string(),
                        documentation: Some(Documentation::String(instruction_info.description.to_string())),
                        parameters: None,
                        active_parameter: None,
                    }),
                    _ => Some(SignatureInformation {
                        label: format!("{} operand", instruction),
                        documentation: Some(Documentation::String(instruction_info.description.to_string())),
                        parameters: Some(vec![
                            ParameterInformation {
                                label: ParameterLabel::Simple("operand".to_string()),
                                documentation: Some(Documentation::String("Required operand".to_string())),
                            }
                        ]),
                        active_parameter: None,
                    }),
                }
            }
        }
    }

    /// based on the cursor position
    fn get_active_parameter(&self, text_before_cursor: &str, instruction: &str) -> Option<u32> {
        let words: Vec<&str> = text_before_cursor.split_whitespace().collect();
        
        // instruction + at least 1 space -> parameter mode
        if words.len() >= 2 || (words.len() == 1 && text_before_cursor.ends_with(' ')) {
            Some(0) // first parameter
        } else {
            None // still instruction
        }
    }
}

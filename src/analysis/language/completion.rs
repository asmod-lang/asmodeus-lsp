use tower_lsp::lsp_types::*;
use crate::analysis::utils::{InstructionDatabase, InstructionCategory, get_line_at_position, is_valid_position};

#[derive(Debug, Clone)]
pub enum CompletionContext {
    Instruction,
    Operand,
    Label,
    Unknown,
}

#[derive(Debug)]
pub struct CompletionProvider {
    instruction_db: InstructionDatabase,
}

impl CompletionProvider {
    pub fn new() -> Self {
        Self {
            instruction_db: InstructionDatabase::new(),
        }
    }

    pub fn get_completions(&self, content: &str, position: Position) -> Vec<CompletionItem> {
        let mut completions = Vec::new();

        if content.is_empty() {
            completions.extend(self.get_instruction_completions());
            return completions;
        }

        if !is_valid_position(content, position) {
            completions.extend(self.get_instruction_completions());
            return completions;
        }

        let current_line = match get_line_at_position(content, position) {
            Some(line) => line,
            None => {
                completions.extend(self.get_instruction_completions());
                return completions;
            }
        };

        let cursor_pos = position.character as usize;
        
        // get text before cursor for context analysis
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
                // defaults to instructions
                completions.extend(self.get_instruction_completions());
            }
        }

        completions
    }

    fn analyze_completion_context(&self, text_before_cursor: &str) -> CompletionContext {
        let trimmed = text_before_cursor.trim();
        
        // line is empty or ends with label ":" -> suggest instructions
        if trimmed.is_empty() || trimmed.ends_with(':') {
            return CompletionContext::Instruction;
        }

        // after instructions -> suggest operands
        let words: Vec<&str> = trimmed.split_whitespace().collect();
        if words.len() >= 1 {
            let first_word = words[0];
            if self.instruction_db.is_valid_instruction(first_word) && words.len() == 1 {
                return CompletionContext::Operand;
            }
        }

        // during typing -> check context 
        if trimmed.chars().all(|c| c.is_uppercase() || c == '_') {
            return CompletionContext::Instruction;
        }

        CompletionContext::Unknown
    }

    fn get_instruction_completions(&self) -> Vec<CompletionItem> {
        let mut completions = Vec::new();

        for instruction in self.instruction_db.get_all_instructions() {
            let (snippet, kind_text) = match instruction.category {
                InstructionCategory::Arithmetic => (
                    format!("{} ${{1:operand}}", instruction.name),
                    "Arithmetic"
                ),
                InstructionCategory::Memory => (
                    format!("{} ${{1:operand}}", instruction.name),
                    "Memory"
                ),
                InstructionCategory::ControlFlow => {
                    if instruction.name == "STP" {
                        (instruction.name.to_string(), "Control Flow")
                    } else {
                        (format!("{} ${{1:label}}", instruction.name), "Control Flow")
                    }
                },
                InstructionCategory::Stack => (
                    instruction.name.to_string(),
                    "Stack"
                ),
                InstructionCategory::Interrupt => {
                    if instruction.name == "MSK" {
                        (format!("{} ${{1:mask}}", instruction.name), "Interrupt")
                    } else {
                        (instruction.name.to_string(), "Interrupt")
                    }
                },
                InstructionCategory::InputOutput => (
                    instruction.name.to_string(),
                    "I/O"
                ),
            };

            let label = if instruction.is_extended {
                format!("{} (Extended)", instruction.name)
            } else {
                instruction.name.to_string()
            };

            let sort_priority = if instruction.is_extended { "2" } else { "1" };

            completions.push(CompletionItem {
                label,
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(kind_text.to_string()),
                documentation: Some(Documentation::String(instruction.description.to_string())),
                insert_text: Some(snippet),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                sort_text: Some(format!("{}_{}", sort_priority, instruction.name)),
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
            // TODO: label completions with content
        ]
    }

    fn get_label_completions(&self, _content: &str) -> Vec<CompletionItem> {
        // TODO: parsing content for existing labels
        Vec::new()
    }
}

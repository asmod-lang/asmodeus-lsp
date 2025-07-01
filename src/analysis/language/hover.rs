use tower_lsp::lsp_types::*;
use crate::analysis::utils::{
    InstructionDatabase, 
    get_line_at_position, 
    is_valid_position, 
    get_word_at_position,
    get_label_info,
    word_range
};

#[derive(Debug)]
pub struct HoverProvider {
    instruction_db: InstructionDatabase,
}

impl HoverProvider {
    pub fn new() -> Self {
        Self {
            instruction_db: InstructionDatabase::new(),
        }
    }

    pub fn get_hover_info(&self, content: &str, position: Position) -> Option<Hover> {
        if !is_valid_position(content, position) {
            return None;
        }

        let current_line = get_line_at_position(content, position)?;
        let cursor_pos = position.character as usize;

        let word_info = get_word_at_position(current_line, cursor_pos)?;
        let (word, start_pos, end_pos) = word_info;

        if word.trim().is_empty() {
            return None;
        }

        // first check instruction info
        let hover_content = if let Some(instruction_info) = self.get_instruction_info(&word) {
            instruction_info
        } else if let Some(label_info) = get_label_info(&word, content) {
            label_info
        } else {
            return None;
        };

        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: hover_content,
            }),
            range: Some(word_range(position.line, start_pos, end_pos)),
        })
    }

    fn get_instruction_info(&self, word: &str) -> Option<String> {
    let instruction = self.instruction_db.get_instruction(word)?;
    
    let extended_note = if instruction.is_extended {
        " *(Extended)*"
    } else {
        ""
    };

    let category_info = match instruction.category {
        crate::analysis::utils::InstructionCategory::Arithmetic => "**Category:** Arithmetic",
        crate::analysis::utils::InstructionCategory::Memory => "**Category:** Memory",
        crate::analysis::utils::InstructionCategory::ControlFlow => "**Category:** Control Flow",
        crate::analysis::utils::InstructionCategory::Stack => "**Category:** Stack",
        crate::analysis::utils::InstructionCategory::Interrupt => "**Category:** Interrupt",
        crate::analysis::utils::InstructionCategory::InputOutput => "**Category:** I/O",
    };

    let operand_info = match instruction.operand_type {
        crate::analysis::utils::OperandType::None => "",
        crate::analysis::utils::OperandType::Single => "\n\n**Operand:** Memory address, immediate value (#42), or label",
        crate::analysis::utils::OperandType::Address => "\n\n**Operand:** Memory address or label",
        crate::analysis::utils::OperandType::Label => "\n\n**Operand:** Target label or address",
        crate::analysis::utils::OperandType::Immediate => "\n\n**Operand:** Immediate value (#42)",
    };

    let extended_info = if instruction.is_extended {
        "\n\n**Note:** Requires `--extended` flag"
    } else {
        ""
    };

    let result = format!(
        "**{}**{}\n\n**Operation:** `{}`\n\n{}\n\n**Description:** {}{}{}",
        instruction.name,
        extended_note,
        instruction.operation,
        category_info,
        instruction.description,
        operand_info,
        extended_info
    );
    
    Some(result)
}
}

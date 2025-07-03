use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct InstructionInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub operation: &'static str,
    pub category: InstructionCategory,
    pub operand_type: OperandType,
    pub is_extended: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstructionCategory {
    Arithmetic,
    Memory,
    ControlFlow,
    Stack,
    Interrupt,
    InputOutput,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperandType {
    None,
    AddressOrLabelOnly,
    LabelOnly,
    ImmediateOnly,
    Flexible,
}

#[derive(Debug)]
pub struct InstructionDatabase {
    instructions: HashMap<&'static str, InstructionInfo>,
}

impl InstructionDatabase {
    pub fn new() -> Self {
        let mut db = Self {
            instructions: HashMap::new(),
        };
        db.initialize();
        db
    }

    fn initialize(&mut self) {
        let instructions = [
            // Arithmetic instructions
            InstructionInfo {
                name: "DOD",
                description: "Add value to accumulator: (AK) + (operand) → AK",
                operation: "(AK) + (operand) → AK",
                category: InstructionCategory::Arithmetic,
                operand_type: OperandType::Flexible,
                is_extended: false,
            },
            InstructionInfo {
                name: "ODE",
                description: "Subtract value from accumulator: (AK) - (operand) → AK",
                operation: "(AK) - (operand) → AK",
                category: InstructionCategory::Arithmetic,
                operand_type: OperandType::Flexible,
                is_extended: false,
            },
            // Memory instructions
            InstructionInfo {
                name: "POB",
                description: "Load value into accumulator: (operand) → AK",
                operation: "(operand) → AK",
                category: InstructionCategory::Memory,
                operand_type: OperandType::Flexible,
                is_extended: false,
            },
            InstructionInfo {
                name: "ŁAD",
                description: "Store accumulator to memory: (AK) → (address)",
                operation: "(AK) → (address)",
                category: InstructionCategory::Memory,
                operand_type: OperandType::AddressOrLabelOnly,
                is_extended: false,
            },
            // Control flow instructions
            InstructionInfo {
                name: "SOB",
                description: "Unconditional jump to label",
                operation: "Unconditional jump",
                category: InstructionCategory::ControlFlow,
                operand_type: OperandType::LabelOnly,
                is_extended: false,
            },
            InstructionInfo {
                name: "SOM",
                description: "Jump to label if AK < 0",
                operation: "Jump if AK < 0",
                category: InstructionCategory::ControlFlow,
                operand_type: OperandType::LabelOnly,
                is_extended: false,
            },
            InstructionInfo {
                name: "SOZ",
                description: "Jump to label if AK = 0",
                operation: "Jump if AK = 0",
                category: InstructionCategory::ControlFlow,
                operand_type: OperandType::LabelOnly,
                is_extended: false,
            },
            InstructionInfo {
                name: "STP",
                description: "Stop program execution",
                operation: "Halt execution",
                category: InstructionCategory::ControlFlow,
                operand_type: OperandType::None,
                is_extended: false,
            },
            // Stack instructions
            InstructionInfo {
                name: "SDP",
                description: "Push accumulator to stack: (AK) → stack",
                operation: "(AK) → stack",
                category: InstructionCategory::Stack,
                operand_type: OperandType::None,
                is_extended: false,
            },
            InstructionInfo {
                name: "PZS",
                description: "Pop from stack to accumulator: stack → AK",
                operation: "stack → AK",
                category: InstructionCategory::Stack,
                operand_type: OperandType::None,
                is_extended: false,
            },
            // Interrupt instructions
            InstructionInfo {
                name: "DNS",
                description: "Disable interrupt handling",
                operation: "Disable interrupts",
                category: InstructionCategory::Interrupt,
                operand_type: OperandType::None,
                is_extended: false,
            },
            InstructionInfo {
                name: "CZM",
                description: "Clear interrupt mask register",
                operation: "Clear interrupt mask",
                category: InstructionCategory::Interrupt,
                operand_type: OperandType::None,
                is_extended: false,
            },
            InstructionInfo {
                name: "MSK",
                description: "Set interrupt mask register",
                operation: "Set interrupt mask",
                category: InstructionCategory::Interrupt,
                operand_type: OperandType::ImmediateOnly,
                is_extended: false,
            },
            InstructionInfo {
                name: "PWR",
                description: "Return from interrupt handler",
                operation: "Return from interrupt",
                category: InstructionCategory::Interrupt,
                operand_type: OperandType::None,
                is_extended: false,
            },
            // I/O instructions
            InstructionInfo {
                name: "WEJSCIE",
                description: "Input value from user",
                operation: "Input → AK",
                category: InstructionCategory::InputOutput,
                operand_type: OperandType::None,
                is_extended: false,
            },
            InstructionInfo {
                name: "WYJSCIE",
                description: "Output accumulator value",
                operation: "Output AK",
                category: InstructionCategory::InputOutput,
                operand_type: OperandType::None,
                is_extended: false,
            },
            // Extended instructions
            InstructionInfo {
                name: "MNO",
                description: "Multiply: (AK) * (operand) → AK [Extended]",
                operation: "(AK) * (operand) → AK",
                category: InstructionCategory::Arithmetic,
                operand_type: OperandType::Flexible,
                is_extended: true,
            },
            InstructionInfo {
                name: "DZI",
                description: "Divide: (AK) / (operand) → AK [Extended]",
                operation: "(AK) / (operand) → AK",
                category: InstructionCategory::Arithmetic,
                operand_type: OperandType::Flexible,
                is_extended: true,
            },
            InstructionInfo {
                name: "MOD",
                description: "Modulo: (AK) % (operand) → AK [Extended]",
                operation: "(AK) % (operand) → AK",
                category: InstructionCategory::Arithmetic,
                operand_type: OperandType::Flexible,
                is_extended: true,
            },
        ];

        for instruction in instructions {
            self.instructions.insert(instruction.name, instruction);
        }
    }

    pub fn get_instruction(&self, name: &str) -> Option<&InstructionInfo> {
        self.instructions.get(name)
    }

    pub fn is_valid_instruction(&self, name: &str) -> bool {
        self.instructions.contains_key(name)
    }

    pub fn get_all_instructions(&self) -> Vec<&InstructionInfo> {
        self.instructions.values().collect()
    }

    pub fn get_instructions_by_category(
        &self,
        category: InstructionCategory,
    ) -> Vec<&InstructionInfo> {
        self.instructions
            .values()
            .filter(|info| info.category == category)
            .collect()
    }

    pub fn find_similar_instructions(&self, unknown: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        let unknown_upper = unknown.to_uppercase();

        for &instruction_name in self.instructions.keys() {
            let distance = levenshtein_distance(&unknown_upper, instruction_name);
            if distance <= 2 {
                // max 2 character differences
                suggestions.push(instruction_name.to_string());
            }
        }

        // by similarity (distance)
        suggestions.sort_by_key(|s| levenshtein_distance(&unknown_upper, s));
        suggestions
    }
}

fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

    // first row and column
    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    for j in 0..=b_len {
        matrix[0][j] = j;
    }

    // matrix
    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i - 1][j] + 1, // deletion
                    matrix[i][j - 1] + 1, // insertion
                ),
                matrix[i - 1][j - 1] + cost, // substitution
            );
        }
    }

    matrix[a_len][b_len]
}

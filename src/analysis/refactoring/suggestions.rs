use crate::analysis::utils::InstructionDatabase;

#[derive(Debug)]
pub struct SuggestionProvider {
    instruction_db: InstructionDatabase,
}

impl SuggestionProvider {
    pub fn new() -> Self {
        Self {
            instruction_db: InstructionDatabase::new(),
        }
    }

    pub fn find_similar_instructions(&self, unknown: &str) -> Vec<String> {
        self.instruction_db.find_similar_instructions(unknown)
    }

    pub fn suggest_common_fixes(&self, unknown: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        suggestions.extend(self.find_similar_instructions(unknown));
        suggestions.extend(self.suggest_common_typos(unknown));
        
        suggestions.sort();
        suggestions.dedup();
        
        suggestions
    }

    fn suggest_common_typos(&self, unknown: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        let unknown_lower = unknown.to_lowercase();
        
        let common_typos = [
            ("pob", "POB"),
            ("dod", "DOD"),
            ("ode", "ODE"),
            ("sob", "SOB"),
            ("som", "SOM"),
            ("soz", "SOZ"),
            ("stp", "STP"),
            ("lad", "ŁAD"),
            ("ład", "ŁAD"),
            ("wejscie", "WEJSCIE"),
            ("wyjscie", "WYJSCIE"),
            ("mno", "MNO"),
            ("dzi", "DZI"),
            ("mod", "MOD"),
            ("sdp", "SDP"),
            ("pzs", "PZS"),
            ("dns", "DNS"),
            ("czm", "CZM"),
            ("msk", "MSK"),
            ("pwr", "PWR"),
        ];

        for (typo, correct) in &common_typos {
            if unknown_lower == *typo {
                suggestions.push(correct.to_string());
            }
        }

        // L/Ł
        if unknown.contains('L') && !unknown.contains('Ł') {
            let with_l_kreskowane = unknown.replace('L', "Ł");
            if self.instruction_db.is_valid_instruction(&with_l_kreskowane) {
                suggestions.push(with_l_kreskowane);
            }
        }

        suggestions
    }

    pub fn suggest_alternative_instructions(&self, context: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        match context.to_lowercase().as_str() {
            "add" | "plus" | "sum" => suggestions.push("DOD".to_string()),
            "subtract" | "minus" | "sub" => suggestions.push("ODE".to_string()),
            "load" | "get" | "fetch" => suggestions.push("POB".to_string()),
            "store" | "save" | "put" => suggestions.push("ŁAD".to_string()),
            "jump" | "goto" | "branch" => {
                suggestions.extend(vec!["SOB".to_string(), "SOM".to_string(), "SOZ".to_string()]);
            },
            "stop" | "halt" | "end" => suggestions.push("STP".to_string()),
            "input" | "read" => suggestions.push("WEJSCIE".to_string()),
            "output" | "print" | "write" => suggestions.push("WYJSCIE".to_string()),
            "multiply" | "mul" => suggestions.push("MNO".to_string()),
            "divide" | "div" => suggestions.push("DZI".to_string()),
            "modulo" | "remainder" => suggestions.push("MOD".to_string()),
            _ => {}
        }
        
        suggestions
    }

    pub fn is_valid_suggestion(&self, suggestion: &str) -> bool {
        self.instruction_db.is_valid_instruction(suggestion)
    }
}

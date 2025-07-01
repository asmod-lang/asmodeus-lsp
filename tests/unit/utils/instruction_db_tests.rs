use asmodeus_lsp::analysis::utils::{InstructionDatabase, InstructionCategory};

#[test]
fn test_instruction_lookup() {
    let db = InstructionDatabase::new();
    
    assert!(db.is_valid_instruction("POB"));
    assert!(db.is_valid_instruction("DOD"));
    assert!(db.is_valid_instruction("STP"));
    assert!(!db.is_valid_instruction("INVALID"));
    
    let pob_info = db.get_instruction("POB").unwrap();
    assert_eq!(pob_info.name, "POB");
    assert_eq!(pob_info.category, InstructionCategory::Memory);
    assert!(!pob_info.is_extended);
}

#[test]
fn test_extended_instructions() {
    let db = InstructionDatabase::new();
    
    assert!(db.is_valid_instruction("MNO"));
    assert!(db.is_valid_instruction("DZI"));
    assert!(db.is_valid_instruction("MOD"));
    
    let mno_info = db.get_instruction("MNO").unwrap();
    assert!(mno_info.is_extended);
    assert_eq!(mno_info.category, InstructionCategory::Arithmetic);
}

#[test]
fn test_similar_instructions() {
    let db = InstructionDatabase::new();
    
    let suggestions = db.find_similar_instructions("DOX");
    assert!(suggestions.contains(&"DOD".to_string()));
    
    let suggestions = db.find_similar_instructions("PO");
    assert!(suggestions.contains(&"POB".to_string()));
    
    let suggestions = db.find_similar_instructions("INVALID_LONG_NAME");
    assert!(suggestions.is_empty());
}

#[test]
fn test_category_filtering() {
    let db = InstructionDatabase::new();
    
    let arithmetic = db.get_instructions_by_category(InstructionCategory::Arithmetic);
    assert!(arithmetic.iter().any(|i| i.name == "DOD"));
    assert!(arithmetic.iter().any(|i| i.name == "ODE"));
    assert!(arithmetic.iter().any(|i| i.name == "MNO"));
    
    let control_flow = db.get_instructions_by_category(InstructionCategory::ControlFlow);
    assert!(control_flow.iter().any(|i| i.name == "SOB"));
    assert!(control_flow.iter().any(|i| i.name == "STP"));
    
    let io = db.get_instructions_by_category(InstructionCategory::InputOutput);
    assert!(io.iter().any(|i| i.name == "WEJSCIE"));
    assert!(io.iter().any(|i| i.name == "WYJSCIE"));
}

#[test]
fn test_all_instructions_covered() {
    let db = InstructionDatabase::new();
    let all_instructions = db.get_all_instructions();
    
    let instruction_names: Vec<&str> = all_instructions.iter().map(|i| i.name).collect();
    
    assert!(instruction_names.contains(&"POB"));
    assert!(instruction_names.contains(&"DOD"));
    assert!(instruction_names.contains(&"ODE"));
    assert!(instruction_names.contains(&"ŁAD"));
    assert!(instruction_names.contains(&"SOB"));
    assert!(instruction_names.contains(&"SOM"));
    assert!(instruction_names.contains(&"SOZ"));
    assert!(instruction_names.contains(&"STP"));
    assert!(instruction_names.contains(&"WEJSCIE"));
    assert!(instruction_names.contains(&"WYJSCIE"));
}

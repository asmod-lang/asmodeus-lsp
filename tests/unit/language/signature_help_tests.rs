use asmodeus_lsp::analysis::language::SignatureHelpProvider;
use tower_lsp::lsp_types::*;

#[test]
fn test_signature_help_for_pob() {
    let provider = SignatureHelpProvider::new();
    let content = "POB ";
    let signature = provider.get_signature_help(content, Position { line: 0, character: 4 });
    
    assert!(signature.is_some());
    let signature = signature.unwrap();
    
    assert_eq!(signature.signatures.len(), 1);
    assert_eq!(signature.active_signature, Some(0));
    assert_eq!(signature.active_parameter, Some(0));
    
    let sig_info = &signature.signatures[0];
    assert!(sig_info.label.contains("POB"));
    assert!(sig_info.parameters.is_some());
    
    let params = sig_info.parameters.as_ref().unwrap();
    assert_eq!(params.len(), 1);
    
    match &params[0].label {
        ParameterLabel::Simple(label) => assert_eq!(label, "operand"),
        _ => panic!("Expected simple label"),
    }
}

#[test]
fn test_signature_help_for_jump_instruction() {
    let provider = SignatureHelpProvider::new();
    let content = "SOB ";
    let signature = provider.get_signature_help(content, Position { line: 0, character: 4 });
    
    assert!(signature.is_some());
    let signature = signature.unwrap();
    
    let sig_info = &signature.signatures[0];
    assert!(sig_info.label.contains("SOB"));
    assert!(sig_info.label.contains("label"));
    
    match &sig_info.documentation {
        Some(Documentation::String(doc)) => {
            assert!(doc.contains("Unconditional jump"));
        },
        _ => panic!("Expected string documentation"),
    }
}

#[test]
fn test_signature_help_for_no_operand_instruction() {
    let provider = SignatureHelpProvider::new();
    let content = "STP ";
    let signature = provider.get_signature_help(content, Position { line: 0, character: 4 });
    
    assert!(signature.is_some());
    let signature = signature.unwrap();
    
    let sig_info = &signature.signatures[0];
    assert!(sig_info.label.contains("STP"));
}

#[test]
fn test_signature_help_extended_instruction() {
    let provider = SignatureHelpProvider::new();
    let content = "MNO ";
    let signature = provider.get_signature_help(content, Position { line: 0, character: 4 });
    
    assert!(signature.is_some());
    let signature = signature.unwrap();
    
    let sig_info = &signature.signatures[0];
    assert!(sig_info.label.contains("MNO"));
    
    match &sig_info.documentation {
        Some(Documentation::String(doc)) => {
            assert!(doc.contains("Extended"));
            assert!(doc.contains("--extended"));
        },
        _ => panic!("Expected string documentation"),
    }
}

#[test]
fn test_signature_help_invalid_instruction() {
    let provider = SignatureHelpProvider::new();
    let content = "INVALID ";
    let signature = provider.get_signature_help(content, Position { line: 0, character: 8 });
    
    assert!(signature.is_none());
}

#[test]
fn test_signature_help_while_typing_instruction() {
    let provider = SignatureHelpProvider::new();
    let content = "PO";
    let signature = provider.get_signature_help(content, Position { line: 0, character: 2 });
    
    assert!(signature.is_none());
}

#[test]
fn test_signature_help_with_multiple_words() {
    let provider = SignatureHelpProvider::new();
    let content = "POB #42";
    let signature = provider.get_signature_help(content, Position { line: 0, character: 7 });
    
    assert!(signature.is_some());
    let signature = signature.unwrap();
    assert_eq!(signature.active_parameter, Some(0));
}

#[test]
fn test_signature_help_beyond_content() {
    let provider = SignatureHelpProvider::new();
    let content = "POB #42";
    let signature = provider.get_signature_help(content, Position { line: 1, character: 0 });
    
    assert!(signature.is_none());
}

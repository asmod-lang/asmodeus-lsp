use tower_lsp::lsp_types::*;
use crate::analysis::utils::{InstructionDatabase, is_word_char, could_be_label, classify_token};

#[derive(Debug, Clone)]
struct RawSemanticToken {
    line: u32,
    start_char: u32,
    length: u32,
    token_type: u32,
    token_modifiers: u32,
}

#[derive(Debug)]
pub struct SemanticTokensProvider {
    instruction_db: InstructionDatabase,
}

impl SemanticTokensProvider {
    pub fn new() -> Self {
        Self {
            instruction_db: InstructionDatabase::new(),
        }
    }

    pub fn get_semantic_tokens(&self, content: &str) -> Vec<SemanticToken> {
        let mut tokens = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        
        for (line_num, line) in lines.iter().enumerate() {
            let line_tokens = self.tokenize_line(line, line_num as u32);
            tokens.extend(line_tokens);
        }
        
        // convert to delta encoding
        self.encode_semantic_tokens(tokens)
    }

    fn tokenize_line(&self, line: &str, line_num: u32) -> Vec<RawSemanticToken> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            if chars[i].is_whitespace() {
                i += 1;
                continue;
            }
            
            // comment 
            if chars[i] == ';' {
                tokens.push(RawSemanticToken {
                    line: line_num,
                    start_char: i as u32,
                    length: (chars.len() - i) as u32,
                    token_type: 4, // COMMENT
                    token_modifiers: 0,
                });
                break;
            }
            
            // operands
            if chars[i] == '#' {
                tokens.push(RawSemanticToken {
                    line: line_num,
                    start_char: i as u32,
                    length: 1,
                    token_type: 3, // OPERATOR
                    token_modifiers: 0,
                });
                i += 1;
                continue;
            }
            
            if chars[i] == '[' || chars[i] == ']' {
                tokens.push(RawSemanticToken {
                    line: line_num,
                    start_char: i as u32,
                    length: 1,
                    token_type: 3, // OPERATOR
                    token_modifiers: 0,
                });
                i += 1;
                continue;
            }
            
            // word/number
            let start = i;
            while i < chars.len() && is_word_char(chars[i]) {
                i += 1;
            }
            
            if start < i {
                let word: String = chars[start..i].iter().collect();
                let token_type = self.classify_token(&word, line, start);
                
                tokens.push(RawSemanticToken {
                    line: line_num,
                    start_char: start as u32,
                    length: (i - start) as u32,
                    token_type,
                    token_modifiers: 0,
                });
            } else {
                i += 1; // unknown symbol
            }
        }
        
        tokens
    }

    /// based on content and context
    fn classify_token(&self, word: &str, line: &str, position: usize) -> u32 {
        // number 
        if word.chars().all(|c| c.is_ascii_digit()) || 
           word.starts_with("0x") || word.starts_with("0b") {
            return 2; // NUMBER
        }
        
        // instruction
        if self.instruction_db.is_valid_instruction(word) {
            return 0; // KEYWORD
        }
        
        // label (after which is : )
        let after_word = position + word.len();
        if let Some(next_char) = line.chars().nth(after_word) {
            if next_char == ':' {
                return 1; // FUNCTION (label definition)
            }
        }
        
        // label reference
        if could_be_label(word) {
            return 1; // FUNCTION (label reference)
        }
        
        2 // defaults to NUMBER/IDENTIFIER
    }

    /// raw tokens to delta encoding format (LSP requirement)
    fn encode_semantic_tokens(&self, tokens: Vec<RawSemanticToken>) -> Vec<SemanticToken> {
        let mut encoded = Vec::new();
        let mut prev_line = 0;
        let mut prev_char = 0;
        
        for token in tokens {
            let delta_line = token.line - prev_line;
            let delta_char = if delta_line == 0 {
                token.start_char - prev_char
            } else {
                token.start_char
            };
            
            encoded.push(SemanticToken {
                delta_line,
                delta_start: delta_char,
                length: token.length,
                token_type: token.token_type,
                token_modifiers_bitset: token.token_modifiers,
            });
            
            prev_line = token.line;
            prev_char = token.start_char;
        }
        
        encoded
    }
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LspError {
    #[error("Lexer error: {0}")]
    LexerError(#[from] lexariel::LexerError),
    
    #[error("Parser error: {0}")]
    ParserError(#[from] parseid::ParserError),
    
    #[error("Analysis error: {message}")]
    AnalysisError { message: String },
    
    #[error("Document not found: {uri}")]
    DocumentNotFound { uri: String },
    
    #[error("Invalid position: line {line}, character {character}")]
    InvalidPosition { line: u32, character: u32 },
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type LspResult<T> = Result<T, LspError>;

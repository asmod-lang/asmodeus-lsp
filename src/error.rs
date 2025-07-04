use thiserror::Error;

#[derive(Error, Debug)]
pub enum LspError {
    #[error("Lexer error: {0}")]
    LexerError(#[from] lexariel::LexerError),

    #[error("Parser error: {0}")]
    ParserError(#[from] parseid::ParserError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type LspResult<T> = Result<T, LspError>;

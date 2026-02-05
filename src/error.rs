use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum SqlexError {
    #[error("Failed to read file: {path}")]
    FileRead {
        path: String,
        source: std::io::Error,
    },

    #[error("SQL syntax error at line {line}, column {column}: {message}")]
    SyntaxError {
        line: usize,
        column: usize,
        message: String,
    },

    #[error("Unsupported dialect: {0}")]
    UnsupportedDialect(String),

    #[error("No SQL files found in: {0}")]
    NoFilesFound(String),
}

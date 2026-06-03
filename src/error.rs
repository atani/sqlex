use thiserror::Error;

#[derive(Error, Debug)]
pub enum SqlexError {
    #[error("Unsupported dialect: {0}")]
    UnsupportedDialect(String),
}

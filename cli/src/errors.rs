//! CLI Error Mappings

use thiserror::Error;

#[derive(Debug, Error)]
#[error("...")]
pub enum SolKeriCliError {
    #[error("Failed getting transaction")]
    GetTransactionError,
    #[error("Failed decoding transaction")]
    DecodeTransactionError,
    // Add custom errors here
    #[error("Keri Error")]
    KeriError(#[from] keri::error::Error),
    #[error("Io Error")]
    IoError(#[from] std::io::Error),
}

pub type SolKeriResult<T> = std::result::Result<T, SolKeriCliError>;

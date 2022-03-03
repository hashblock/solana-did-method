//! CLI Error Mappings

use thiserror::Error;

#[derive(Debug, Error)]
#[error("...")]
pub enum SolKeriError {
    #[error("Threshold exceeds allowed size {0}")]
    ThresholdError(usize),
    #[error("Failed getting transaction")]
    GetTransactionError,
    #[error("Failed decoding transaction")]
    DecodeTransactionError,
    // Add custom errors here
    #[error("Keri Error")]
    KeriError(#[from] keri::error::Error),
    #[error("Io Error")]
    IoError(#[from] std::io::Error),
    #[error("Serde Error")]
    SerdeError(#[from] serde_json::Error),
}

pub type SolKeriResult<T> = std::result::Result<T, SolKeriError>;

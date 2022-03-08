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
    #[error("Invalid did string {0}")]
    InvalidDidString(String),
    #[error("DID already exists {0}")]
    DIDExists(String),
    // Add custom errors here
    #[error("Solana RpcError")]
    SolRpc(#[from] solana_client::client_error::ClientError),
    #[error("Keri Error")]
    KeriError(#[from] keri::error::Error),
    #[error("Io Error")]
    IoError(#[from] std::io::Error),
    #[error("Serde Error")]
    SerdeError(#[from] serde_json::Error),
}

pub type SolKeriResult<T> = std::result::Result<T, SolKeriError>;

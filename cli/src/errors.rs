//! CLI Error Mappings

use thiserror::Error;

#[derive(Debug, Error)]
#[error("...")]
pub enum SolDidError {
    #[error("Could not find HOME directory")]
    HomeNotFoundError,
    #[error("{0} not found in {1}")]
    WalletConfigNotFound(String, String),
    #[error("{0} not found in {1}")]
    KeyConfigNotFound(String, String),
    #[error("Key {0} not found")]
    KeyNotFound(String),
    #[error("Keys {0} already exists")]
    KeysExistError(String),
    #[error("Attempting to duplicate key")]
    DuplicateKeyError,
    #[error("Unknown Key Type")]
    UnknownKeyTypeError,
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
    // Add library/crate errors here
    #[error("Solana RpcError")]
    SolRpc(#[from] solana_client::client_error::ClientError),
    #[error("Io Error")]
    IoError(#[from] std::io::Error),
    #[error("HBKR Error")]
    HbkrError(#[from] hbkr_rs::errors::KrError),
    // #[error("Serde Error")]
    // SerdeError(#[from] serde_json::Error),
}

pub type SolDidResult<T> = std::result::Result<T, SolDidError>;

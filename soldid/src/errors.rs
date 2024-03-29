//! CLI Error Mappings

use ed25519_dalek::SignatureError;
use thiserror::Error;

#[derive(Debug, Error)]
// #[error("...")]
pub enum SolDidError {
    #[error("Solana default configuration not found")]
    SolanaConfigMissing,
    #[error("Could not find HOME directory")]
    HomeNotFoundError,
    #[error("{0} not found in {1}")]
    WalletConfigNotFound(String, String),
    #[error("{0} not found in {1}")]
    KeyConfigNotFound(String, String),
    #[error("Key {0} not found")]
    KeyNotFound(String),
    #[error("Name: {0} not a keyset found in wallet")]
    NameNotFound(String),
    #[error("Prefix: {0} not a keyset found in wallet")]
    PrefixNotFound(String),
    #[error("Keys with prefix {0} already exists")]
    KeysPrefixExistError(String),
    #[error("Keys with name {0} already exists")]
    KeysNameExistError(String),
    #[error("Keys incoherent")]
    KeySetIncoherence,
    #[error("Attempting to duplicate key")]
    DuplicateKeyError,
    #[error("Unknown Key Type")]
    UnknownKeyTypeError,
    #[error("Threshold exceeds allowed size {0}")]
    ThresholdError(usize),
    #[error("Can not rotate keys: No inception or previous rotation found")]
    RotationIncoherence,
    #[error("Can not rotate keys: KeySet has been revoked or decommissioned")]
    RotationIncompatible,
    #[error("Attempt to rotate with empty next keyset. Use decommission instead")]
    RotationToEmptyError,
    #[error("Failed getting transaction")]
    GetTransactionError,
    #[error("Failed decoding transaction")]
    DecodeTransactionError,
    #[error("Invalid did string {0}")]
    InvalidDidString(String),
    #[error("DID account already exists {0}")]
    DIDAccountExists(String),
    #[error("Called Inception with 0 current keys.")]
    DIDInvalidInceptionZeroKeys,
    #[error("DID account {0} does not exists")]
    DIDAccountNotExists(String),
    #[error("Called Rotation with 0 current keys. Should use Decommision instead")]
    DIDInvalidRotationUseDecommision,
    // Add custom errors here
    // Add library/crate errors here
    #[error("Solana RpcError")]
    SolRpc(#[from] solana_client::client_error::ClientError),
    #[error("Io Error")]
    IoError(#[from] std::io::Error),
    #[error("HBKR Error")]
    HbkrError(#[from] hbkr_rs::errors::KrError),
    #[error("Std Error")]
    StdError(#[from] Box<dyn std::error::Error>),
    #[error("ED25519 Signature Error")]
    EDError(#[from] SignatureError),
    #[error("Base 58 decoding error")]
    Bse58Error(#[from] bs58::decode::Error),
}

pub type SolDidResult<T> = std::result::Result<T, SolDidError>;

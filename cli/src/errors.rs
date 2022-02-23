//! CLI Error Mappings

use thiserror::Error;

#[derive(Debug, Error)]
#[error("...")]
pub enum ApplicationError {
    // Throw this error when attempting to
    // initialize account that is already
    // initialized
    // #[error("Invalid DID Reference")]
    // InvalidDidReference,
    // Add custom errors here
    #[error("Keri Error")]
    KeriError(#[from] keri::error::Error),
}

pub type AppResult<T> = std::result::Result<T, ApplicationError>;

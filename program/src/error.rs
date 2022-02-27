//! @brief Program Error Enum
use num_derive::FromPrimitive;
use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Debug, Error, FromPrimitive)]
#[error("...")]
pub enum SDMProgramError {
    #[error("DID Account is not initialized")]
    DidNotInitialized,
    #[error("DID Account is already initialized")]
    DidAlreadyInitialized,
    #[error("Invalid DID Reference")]
    InvalidDidReference,
    #[error("Owner is not signer for DID")]
    OwnerNotSignerError,
    // Add custom errors here
}

/// Enables 'into()` on custom error to convert
/// to ProgramError
impl From<SDMProgramError> for ProgramError {
    fn from(e: SDMProgramError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

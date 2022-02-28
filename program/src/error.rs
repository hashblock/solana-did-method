//! @brief Program Error Enum

use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Debug, Error, num_enum::IntoPrimitive)]
#[repr(u8)]
#[error("...")]
pub enum SDMProgramError {
    #[error("DID Account is not initialized")]
    DidNotInitialized,
    #[error("DID Account is already initialized")]
    DidAlreadyInitialized,
    #[error("DID Data Account version incorrect")]
    DidDataVersionInvalid,
    #[error("DID Key not valid key")]
    DidInvalidKey,
    #[error("Invalid DID Reference")]
    InvalidDidReference,
    #[error("Owner is not signer for DID")]
    OwnerNotSignerError,
}

/// Enables 'into()` on custom error to convert
/// to ProgramError
impl From<SDMProgramError> for ProgramError {
    fn from(e: SDMProgramError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

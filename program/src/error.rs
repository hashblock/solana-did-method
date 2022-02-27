//! @brief Program Error Enum
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

#[derive(Debug, Error, FromPrimitive)]
#[error("...")]
pub enum SDMProgramError {
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

/// Supports error.print
impl<T> DecodeError<T> for SDMProgramError {
    fn type_of() -> &'static str {
        "CustomProgramError"
    }
}

/// Supports error.print
impl PrintProgramError for SDMProgramError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            SDMProgramError::InvalidDidReference => {
                println!("Not a valid DID reference")
            }
            SDMProgramError::OwnerNotSignerError => {
                println!("Owner not equal to signer")
            }
        }
    }
}

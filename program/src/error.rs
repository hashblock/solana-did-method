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
pub enum CustomProgramError {
    // Throw this error when attempting to
    // initialize account that is already
    // initialized
    #[error("Invalid DID Reference")]
    InvalidDidReference,
    // Add custom errors here
}

/// Enables 'into()` on custom error to convert
/// to ProgramError
impl From<CustomProgramError> for ProgramError {
    fn from(e: CustomProgramError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

/// Supports error.print
impl<T> DecodeError<T> for CustomProgramError {
    fn type_of() -> &'static str {
        "CustomProgramError"
    }
}

/// Supports error.print
impl PrintProgramError for CustomProgramError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            CustomProgramError::InvalidDidReference => {
                println!("Not a valid DID reference")
            }
        }
    }
}

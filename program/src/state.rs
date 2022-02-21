//! @brief Program account state management

use crate::error::CustomProgramError;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    borsh::try_from_slice_unchecked,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
};
use std::io::BufWriter;

/// Maintains account data
#[derive(BorshDeserialize, BorshSerialize, Debug, Default, PartialEq)]
pub struct ProgramAccountState {
    is_initialized: bool,
    data_version: u8,
    content: u8,
}

impl ProgramAccountState {
    /// Signal initialized
    pub fn set_initialized(&mut self) {
        self.is_initialized = true;
    }
    /// Get the initialized flag
    pub fn initialized(&self) -> bool {
        self.is_initialized
    }
    /// Gets the current data version
    pub fn version(&self) -> u8 {
        self.data_version
    }
    /// Get account content
    pub fn content(&self) -> u8 {
        self.content
    }
    /// Set account content and return
    /// previous content value
    pub fn set_content(&mut self, new_content: u8) -> u8 {
        let old_content = self.content;
        self.content = new_content;
        old_content
    }
}

/// Declaration of the current data version.
const DATA_VERSION: u8 = 1; // Adding string to content

/// Need size for account state
/// 1 byte for 'is_initialized'
/// 1 byte for 'data_version`
/// 1 byte for `content`
const ACCOUNT_STATE_SPACE: usize = 3;

/// Implement Sealed trait for ProgramAccountState
/// to satisfy Pack trait constraints
impl Sealed for ProgramAccountState {}

/// Implement IsInitialized trait for ProgramAccountState
/// to satisfy Pack trait constraints
impl IsInitialized for ProgramAccountState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for ProgramAccountState {
    const LEN: usize = ACCOUNT_STATE_SPACE;

    /// Store 'state' of account to its data area
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut bw = BufWriter::new(dst);
        self.serialize(&mut bw).unwrap();
    }

    /// Retrieve 'state' of account from account data area
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let initialized = src[0] != 0;
        // Check initialized
        if initialized {
            // Version check
            if src[1] == DATA_VERSION {
                msg!("Processing consistent version data");
                Ok(try_from_slice_unchecked::<ProgramAccountState>(src)?)
            } else {
                msg!("Incoherrent data version detected");
                Err(CustomProgramError::DataVersionMismatchError.into())
            }
        } else {
            msg!("Processing pre-initialized data");
            Ok(ProgramAccountState {
                is_initialized: false,
                data_version: DATA_VERSION,
                content: 0,
            })
        }
    }
}

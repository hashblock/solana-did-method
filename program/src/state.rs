//! @brief Program account state management

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
};

/// Maintains account data
#[derive(BorshDeserialize, BorshSerialize, Debug, Default, PartialEq)]
pub struct ProgramAccountState {}

/// Need size for account state
const ACCOUNT_STATE_SPACE: usize = 0;

/// Implement Sealed trait for ProgramAccountState
/// to satisfy Pack trait constraints
impl Sealed for ProgramAccountState {}

/// Implement IsInitialized trait for ProgramAccountState
/// to satisfy Pack trait constraints
impl IsInitialized for ProgramAccountState {
    fn is_initialized(&self) -> bool {
        unreachable!()
    }
}

impl Pack for ProgramAccountState {
    const LEN: usize = ACCOUNT_STATE_SPACE;

    /// Store 'state' of account to its data area
    fn pack_into_slice(&self, _dst: &mut [u8]) {
        unreachable!()
    }

    /// Retrieve 'state' of account from account data area
    fn unpack_from_slice(_src: &[u8]) -> Result<Self, ProgramError> {
        unreachable!()
    }
}

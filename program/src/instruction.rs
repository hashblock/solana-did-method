//! @brief Program instruction enum
//!

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    borsh::try_from_slice_unchecked, program_error::ProgramError, pubkey::Pubkey,
};

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct InceptionDID {
    pub prefix: Pubkey,
    pub keys: Vec<Pubkey>,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq)]
/// All custom program instructions
pub enum SDMInstruction {
    /// Initializes a new account with an Inception Event
    /// Accounts expected by this insruction
    /// 0. `[writeable]` The account to initialize
    /// 1. `[] The new DID owner/holder
    ///
    /// The inception data includes
    /// 0. InceptionDID
    ///
    SDMInception(InceptionDID),
    /// Used for testing
    SDMInvalidVersionTest,
}

impl SDMInstruction {
    /// Unpack inbound buffer to associated Instruction
    /// The expected format for input is a Borsh serialized vector
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let payload = try_from_slice_unchecked::<SDMInstruction>(input).unwrap();
        match payload {
            SDMInstruction::SDMInception(_) => Ok(payload),
            // For testing only
            SDMInstruction::SDMInvalidVersionTest => Ok(payload),
        }
    }
}

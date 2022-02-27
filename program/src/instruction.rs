//! @brief Program instruction enum
//!
use std::collections::BTreeMap;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{borsh::try_from_slice_unchecked, program_error::ProgramError};

#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq)]
/// All custom program instructions
pub enum SDMInstruction {
    InceptionEvent(BTreeMap<String, String>),
}

impl SDMInstruction {
    /// Unpack inbound buffer to associated Instruction
    /// The expected format for input is a Borsh serialized vector
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let payload = try_from_slice_unchecked::<SDMInstruction>(input).unwrap();
        match payload {
            SDMInstruction::InceptionEvent(_) => Ok(payload),
        }
    }
}

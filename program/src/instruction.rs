//! @brief Program instruction enum
//!
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{borsh::try_from_slice_unchecked, program_error::ProgramError};

#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq)]
/// All custom program instructions
pub enum ProgramInstruction {
    InitializeAccount,
    SetContent(u8),
}

impl ProgramInstruction {
    /// Unpack inbound buffer to associated Instruction
    /// The expected format for input is a Borsh serialized vector
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let payload = try_from_slice_unchecked::<ProgramInstruction>(input).unwrap();
        match payload {
            ProgramInstruction::InitializeAccount => Ok(payload),
            ProgramInstruction::SetContent(_) => Ok(payload),
        }
    }
}

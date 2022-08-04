//! @brief Program instruction enum
//!

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    borsh::try_from_slice_unchecked, program_error::ProgramError, pubkey::Pubkey,
};

#[derive(BorshDeserialize, BorshSerialize, Clone, Copy, Debug, PartialEq)]
pub enum SMDKeyType {
    Ed25519,
    PASTA,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct DIDInception {
    pub keytype: SMDKeyType,
    pub prefix: [u8; 32],
    pub bump: u8,
    pub keys: Vec<Pubkey>,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct DIDRotation {
    pub keytype: SMDKeyType,
    pub prefix: [u8; 32],
    pub keys: Vec<Pubkey>,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
pub struct DIDDecommission {
    pub keytype: SMDKeyType,
    pub prefix: [u8; 32],
}

#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq)]
pub struct InitializeDidAccount {
    pub rent: u64,
    pub storage: u64,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq)]
/// All custom program instructions
pub enum SDMInstruction {
    /// Initialize a DID account
    /// Accounts expected by this insruction
    /// 0. `[writeable, signable]` Authorizing account
    /// 1. `[writeable]` The DID/PDA account to instantiate
    ///
    /// The initialize data includes
    /// 0. InitializeDidAccount
    ///
    // SDMInitialize(InitializeDidAccount),
    /// Sets a new accounts Inception Event
    /// Accounts expected by this insruction
    /// 0. `[writeable]` signable]` Authorizing account
    /// 1. `[writeable]` The new DID PDA
    ///
    /// The inception data includes
    /// 0. InceptionDidAccount details information about the PDA creation
    /// 1. DIDInception is the payload containing the DID active keys
    ///
    SDMInception(InitializeDidAccount, DIDInception),
    /// Rotate DID public keys
    /// Accounts expected by this instruction
    /// 0. `[writeable, signable]` Authorizing account
    /// 1. `[writeable]` The DID PDA
    ///
    /// The rotation data includes
    /// 0. DIDRotation with verifying information and new keys
    SDMRotation(DIDRotation),
    /// Decommission DID public keys
    /// Accounts expected by this instruction
    /// 0. `[writeable, signable]` Authorizing account
    /// 1. `[writeable]` The DID PDA
    ///
    /// The decommission data includes
    /// 0. DIDDecommission with verifying information and new keys
    SDMDecommission(DIDDecommission),
}

impl SDMInstruction {
    /// Unpack inbound buffer to associated Instruction
    /// The expected format for input is a Borsh serialized vector
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        // let payload = SDMInstruction::try_from_slice(input)?;
        let payload = try_from_slice_unchecked::<SDMInstruction>(input).unwrap();

        match payload {
            SDMInstruction::SDMInception(_, _) => Ok(payload),
            SDMInstruction::SDMRotation(_) => Ok(payload),
            SDMInstruction::SDMDecommission(_) => Ok(payload),
        }
    }
}

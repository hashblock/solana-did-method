//! @brief Program account state management

use std::io::BufWriter;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

pub use crate::error::SDMProgramError;
use crate::instruction::InceptionDID;

/// Indicates the current version supported
/// If different from persist state, a copy on
/// read occurs
const CURRENT_DATA_VERSION: u16 = 1;

#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq)]
pub enum SDMDidState {
    Inception,
    Rotated,
}
#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq)]
pub struct SDMDidDocCurrent {
    state: SDMDidState,
    prefix: [u8; 32],
    bump: u8,
    pub keys: Vec<Pubkey>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
#[allow(dead_code)]
pub struct SDMDid {
    initialized: bool,
    pub version: u16,
    pub did_doc: SDMDidDocCurrent,
}

impl SDMDid {
    /// Sets the initialization flag
    pub fn set_initialized(&mut self) {
        self.initialized = true
    }
    /// Sets the initialization flag
    pub fn flip_version(&mut self) {
        self.version = 20
    }
    /// Assumes the account has not been initialized yet
    /// If so, returns default state or otherwise throws error
    pub fn unpack_unitialized(data: &[u8], with: InceptionDID) -> Result<Self, SDMProgramError> {
        let is_initialized = data[0] != 0;
        if is_initialized {
            Err(SDMProgramError::DidAlreadyInitialized)
        } else {
            Ok(Self {
                initialized: !is_initialized,
                version: CURRENT_DATA_VERSION,
                did_doc: SDMDidDocCurrent {
                    state: SDMDidState::Inception,
                    prefix: with.prefix,
                    bump: with.bump,
                    keys: with.keys,
                },
            })
        }
    }

    /// Assumes the account statte has previously been initialized
    /// If so, unpacks current statte or otherwise throws error
    pub fn unpack(data: &[u8]) -> Result<Self, SDMProgramError> {
        let is_initialized = data[0] != 0;
        if !is_initialized {
            Err(SDMProgramError::DidNotInitialized)
        } else {
            let mut version: [u8; 2] = [0, 0];
            version[0] = data[1];
            version[1] = data[2];
            let version = u16::from_le_bytes(version);
            if version == CURRENT_DATA_VERSION {
                let current = SDMDid::try_from_slice(data).unwrap();
                Ok(current)
            } else {
                Err(SDMProgramError::DidDataVersionInvalid)
            }
        }
    }

    /// Serializes the current data to the account state
    pub fn pack(&mut self, data: &mut [u8]) -> Result<(), SDMProgramError> {
        let mut bw = BufWriter::new(data);
        self.serialize(&mut bw).unwrap();
        Ok(())
    }
}

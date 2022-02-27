//! @brief Program account state management

use std::io::BufWriter;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{msg, pubkey::Pubkey};

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
    prefix: Pubkey,
    keys: Vec<Pubkey>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
#[allow(dead_code)]
pub struct SDMDid {
    initialized: bool,
    version: u16,
    pub did_doc: SDMDidDocCurrent,
}

impl SDMDid {
    /// Sets the initialization flag
    pub fn set_initialized(&mut self) {
        self.initialized = true
    }
    /// Assumes the account has not been initialized yet
    /// If so, returns default state or otherwise throws error
    pub fn unpack_unitialized(data: &[u8], with: InceptionDID) -> Result<Self, SDMProgramError> {
        msg!("Client size = {}", data.len());
        let is_initialized = data[0] != 0;
        if is_initialized {
            Err(SDMProgramError::DidAlreadyInitialized)
        } else {
            Ok(Self {
                initialized: is_initialized,
                version: CURRENT_DATA_VERSION,
                did_doc: SDMDidDocCurrent {
                    state: SDMDidState::Inception,
                    prefix: with.prefix,
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
            Ok(Self {
                initialized: is_initialized,
                did_doc: SDMDidDocCurrent {
                    state: SDMDidState::Inception,
                    prefix: Pubkey::new_unique(),
                    keys: Vec::<Pubkey>::new(),
                },
                version: CURRENT_DATA_VERSION,
            })
        }
    }

    /// Serializes the current data to the account state
    pub fn pack(&mut self, data: &mut [u8]) -> Result<(), SDMProgramError> {
        msg!("Packing data of size {}", data.len());
        // data[0] = self.initialized as u8;
        let mut bw = BufWriter::new(data);
        msg!("Data serialized size {}", self.try_to_vec().unwrap().len());
        self.serialize(&mut bw).unwrap();
        Ok(())
    }
}

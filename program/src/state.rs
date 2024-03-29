//! @brief Program account state management

use std::io::BufWriter;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{borsh::try_from_slice_unchecked, pubkey::Pubkey};

pub use crate::error::SDMProgramError;
use crate::instruction::{DIDDecommission, DIDInception, DIDRotation, SMDKeyType};

/// Indicates the current version supported
/// If different from persist state, a copy on
/// read occurs
const CURRENT_DATA_VERSION: u16 = 1;

#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq)]
pub enum SDMDidState {
    Inception,
    Rotated,
    Decommissioned,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq)]
pub struct SDMDidDocCurrent {
    state: SDMDidState,
    keytype: SMDKeyType,
    authority: Pubkey,
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
    /// Verifies that keytypes and prefix match
    pub fn verify_inbound(
        &self,
        keytype: SMDKeyType,
        prefix: [u8; 32],
    ) -> Result<(), SDMProgramError> {
        if keytype == self.did_doc.keytype && prefix == self.did_doc.prefix {
            Ok(())
        } else {
            Err(SDMProgramError::InvalidDidReference)
        }
    }
    /// Verify that the authority key is equal on the DID
    pub fn verify_authority(&self, in_auth_key: Pubkey) -> Result<(), SDMProgramError> {
        if self.did_doc.authority == in_auth_key {
            Ok(())
        } else {
            Err(SDMProgramError::InvalidAuthority)
        }
    }
    /// Rotate the active keys from the instruction data
    pub fn rotate_with(&mut self, with: DIDRotation) -> Result<(), SDMProgramError> {
        self.did_doc.keys = with.keys;
        self.did_doc.state = SDMDidState::Rotated;
        Ok(())
    }
    /// Rotate the active keys from the instruction data
    pub fn decommission_with(&mut self, _with: DIDDecommission) -> Result<(), SDMProgramError> {
        self.did_doc.keys = Vec::<Pubkey>::new();
        self.did_doc.state = SDMDidState::Decommissioned;
        Ok(())
    }
    /// Sets the initialization flag
    pub fn set_initialized(&mut self) {
        self.initialized = true
    }

    /// Assumes the account has not been initialized yet
    /// If so, returns default state or otherwise throws error
    pub fn unpack_unitialized(
        data: &[u8],
        with: DIDInception,
        authority: &Pubkey,
    ) -> Result<Self, SDMProgramError> {
        let is_initialized = data[0] != 0;
        if is_initialized {
            Err(SDMProgramError::DidAlreadyInitialized)
        } else {
            Ok(Self {
                initialized: !is_initialized,
                version: CURRENT_DATA_VERSION,
                did_doc: SDMDidDocCurrent {
                    state: SDMDidState::Inception,
                    keytype: with.keytype,
                    authority: authority.clone(),
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
                let current = try_from_slice_unchecked::<SDMDid>(data).unwrap();
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

//! Inception event management
//! Inception events are the events that surround the genesis of a new set of keys
//! that are to be managed. This only happens once for the same set of keys, attempting
//! to create an inception even with the same keys will result in the same outcome
//! and will NOT create a new event.
//!
//! Keys involved in overall lifecycle from event generation to putting on the ledger:
//!
//! 1. `[controller keys]` One (1) are more keys are designated for managing
//! 2. `[event signer]` The key that signs the event for authenticity. This may be one of the controller keys
//! 3. `[transaction signer]` The key that signs the transaction.
//! 4. `[transaction payer]` This is a wallet account that has SOL to pay for:
//!     1. `[transaction]` Running a transaction in SOL costs
//!     2. `[DID]` The cost to maintain the DID account on Solana
//!

use crate::errors::{SolKeriError, SolKeriResult};
use keri::{
    derivation::{basic::Basic, self_addressing::SelfAddressing},
    event::{
        event_data::InceptionEvent,
        sections::{key_config::nxt_commitment, threshold::SignatureThreshold, KeyConfig},
        Event, EventMessage, SerializationFormats,
    },
    keys::PublicKey,
    prefix::{BasicPrefix, Prefix, SelfAddressingPrefix},
};
use solana_sdk::{signature::Keypair, signer::Signer};

pub trait SolDidEvent {
    /// Get the underlying Keri event
    fn event(&self) -> &Event;
    /// Get the underlying Keri event
    fn event_message(&self) -> &EventMessage;
    /// Retrieve the prefix bytes
    fn prefix_digest(&self) -> &Vec<u8>;
    /// Retrieve the prefix as a string
    fn prefix_as_string(&self) -> String;
    /// Retrieve the 'did:solana:prefix' string
    fn did_string(&self) -> String;
    /// Convenient serialization
    fn serialize(&self) -> SolKeriResult<Vec<u8>>;
}

#[allow(dead_code)]
#[derive(Debug)]
/// Encapsulates the result of a created Inception event
pub struct SolDidInception {
    /// The Inception Event block
    event_message: EventMessage,

    /// Number of keys in key set required to sign for various operations
    threshold: u64,

    /// List of keypairs initially identified for managing
    initial_key_set: Vec<Keypair>,

    /// List of BasicPrefix derived from initial_key_set
    initial_prefix_set: Vec<BasicPrefix>,

    /// Generated list of keypairs for next rotation
    next_key_set: Vec<Keypair>,

    /// List of BasicPrefix derived from next_key_set
    next_prefix_set: Vec<BasicPrefix>,
}

impl SolDidInception {
    fn prefix(&self) -> &SelfAddressingPrefix {
        match &self.event_message.event.prefix {
            // keri::prefix::IdentifierPrefix::SelfAddressing(prx) => Pubkey::new(&prx.digest),
            keri::prefix::IdentifierPrefix::SelfAddressing(prx) => prx,
            _ => unreachable!(),
        }
    }

    pub fn active_pubkeys(&self) -> Vec<solana_sdk::pubkey::Pubkey> {
        self.initial_key_set
            .iter()
            .map(|kp| kp.pubkey().clone())
            .collect()
    }
}
impl SolDidEvent for SolDidInception {
    fn event(&self) -> &Event {
        &self.event_message.event
    }

    fn event_message(&self) -> &EventMessage {
        &self.event_message
    }
    fn prefix_as_string(&self) -> String {
        self.prefix().to_str()
    }

    fn did_string(&self) -> String {
        format!("did:solana:{}", self.prefix_as_string())
    }

    fn serialize(&self) -> SolKeriResult<Vec<u8>> {
        Ok(self.event_message().serialize()?)
    }

    fn prefix_digest(&self) -> &Vec<u8> {
        &self.prefix().digest
    }
}

/// Generate An inception event. </p>
/// This involved preparing a set of keys to be managed and generating
/// the next set of keypairs that will be rotated to.
///
/// Inputs:
///
/// managed_keys - One (1) are more keys are designated for managing</p>
/// threshold - Number of key signatures required to unlock something
///

pub fn generate_inception_event(
    managed_keys: Vec<Keypair>,
    threshold: u64,
) -> SolKeriResult<SolDidInception> {
    if threshold > managed_keys.len() as u64 {
        return Err(SolKeriError::ThresholdError(managed_keys.len()));
    }

    // Build the first set of BasicPrefixes
    let keri_basic_keys = managed_keys
        .iter()
        .map(|k| {
            BasicPrefix::new(
                Basic::Ed25519,
                PublicKey::new(k.pubkey().to_bytes().to_vec()),
            )
        })
        .collect::<Vec<BasicPrefix>>();

    // Build the next set of keypairs
    let next_kp_set = managed_keys
        .iter()
        .map(|_| Keypair::new())
        .collect::<Vec<Keypair>>();

    // Build next set of BasicPrefixes
    let next_keri_basic_keys = next_kp_set
        .iter()
        .map(|k| {
            BasicPrefix::new(
                Basic::Ed25519,
                PublicKey::new(k.pubkey().to_bytes().to_vec()),
            )
        })
        .collect::<Vec<BasicPrefix>>();

    let next_key_hash = nxt_commitment(
        &SignatureThreshold::Simple(threshold),
        &next_keri_basic_keys,
        &SelfAddressing::Blake3_256,
    );
    let key_config = KeyConfig::new(
        keri_basic_keys.to_vec(),
        Some(next_key_hash),
        Some(SignatureThreshold::Simple(threshold)),
    );
    let event_message = InceptionEvent::new(key_config, None, None)
        .incept_self_addressing(SelfAddressing::Blake3_256, SerializationFormats::JSON)?;

    Ok(SolDidInception {
        initial_key_set: managed_keys,
        threshold,
        initial_prefix_set: keri_basic_keys,
        next_key_set: next_kp_set,
        next_prefix_set: next_keri_basic_keys,
        event_message,
    })
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use solana_sdk::pubkey::Pubkey;

    use crate::errors::SolKeriResult;

    use super::*;

    #[test]
    fn test_compose_pass() -> SolKeriResult<()> {
        let mut keys = Vec::<Keypair>::new();
        keys.push(Keypair::new());
        keys.push(Keypair::new());
        let threshold = keys.len() as u64 - 1u64;
        let sol_did_res = generate_inception_event(keys, threshold);
        assert!(sol_did_res.is_ok());
        let sol_did_icp = sol_did_res.unwrap();
        println!("{:?}", sol_did_icp.prefix_as_string());
        println!("DID => {}", sol_did_icp.did_string());
        // print!("\n{}\n", serde_json::to_string(&sol_did_icp.event())?);
        // println!("Serialized length {:?}", sol_did_icp.serialize()?.len());
        Ok(())
    }
    #[test]
    fn test_compose_fail() -> SolKeriResult<()> {
        let mut keys = Vec::<Keypair>::new();
        keys.push(Keypair::new());
        keys.push(Keypair::new());
        let threshold = keys.len() as u64 + 1u64;
        let sol_did_res = generate_inception_event(keys, threshold);
        assert!(sol_did_res.is_err());
        Ok(())
    }
}

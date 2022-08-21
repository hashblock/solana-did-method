//! Chain event mirroring on keys for DID

use crate::errors::{SolDidError, SolDidResult};

use super::{generic_keys::Key, wallet_enums::KeyType};
use borsh::{BorshDeserialize, BorshSerialize};
use chrono::TimeZone;
use hbkr_rs::{
    event::Event, event_message::EventMessage, key_manage::Privatekey, said_event::SaidEvent,
    EventTypeTag, Prefix, Typeable,
};

use std::{collections::HashMap, fmt};

#[derive(BorshDeserialize, BorshSerialize, Clone, Copy, Debug, Default)]
pub enum ChainEventType {
    #[default]
    Inception,
    Rotation,
    DelegatedInception,
    DelegatedRotation,
    Revoked,
    Decommissioned,
}

impl ChainEventType {
    pub fn can_rotate(prev: ChainEventType) -> bool {
        // Expand when we have more coverage
        if let ChainEventType::Inception | ChainEventType::Rotation = prev {
            true
        } else {
            false
        }
    }
}

impl From<EventTypeTag> for ChainEventType {
    fn from(ett: EventTypeTag) -> Self {
        match ett {
            EventTypeTag::Icp => ChainEventType::Inception,
            EventTypeTag::Rot => ChainEventType::Rotation,
            EventTypeTag::Ixn | EventTypeTag::Dip | EventTypeTag::Drt | EventTypeTag::Rct => {
                todo!()
            }
        }
    }
}

/// Enum identifying which group a key exists in
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, Hash, Eq, PartialEq, PartialOrd)]
pub enum KeyBlock {
    NONE,
    CURRENT,
    NEXT,
    PAST,
}

/// ChainEven tracks/associates key changes for DID to a confirmed signature chain event
/// Signatures are base58 representation
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, Default)]
pub struct ChainEvent {
    pub event_type: ChainEventType,
    pub time_stamp: i64,
    pub did_signature: String,
    pub km_sn: u64,
    pub km_digest: String,
    pub km_keytype: KeyType,
    pub keysets: HashMap<KeyBlock, Vec<Key>>,
}

impl fmt::Display for ChainEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = chrono::Utc;
        write!(f, "Event type:     {:?}\n", self.event_type)?;
        write!(f, "Tx signature:   {}\n", self.did_signature)?;
        write!(
            f,
            "Datetime (UTC): {}\n",
            v.timestamp_millis(self.time_stamp)
        )?;
        for (key, keys) in self.keysets.iter() {
            write!(f, "- {:?}\n", key)?;
            let mut counter = 0i8;
            for k in keys {
                write!(f, "{counter} {:?}\n", k)?;
                counter = counter + 1;
            }
        }
        Ok(())
    }
}

impl ChainEvent {
    pub fn get_keys(&self) -> SolDidResult<&HashMap<KeyBlock, Vec<Key>>> {
        Ok(&self.keysets)
    }
    pub fn get_keys_for(&self, block_type: KeyBlock) -> SolDidResult<&Vec<Key>> {
        if self.keysets.contains_key(&block_type) {
            Ok(self.keysets.get(&block_type).unwrap())
        } else {
            Err(SolDidError::KeySetIncoherence)
        }
    }
    /// Gets a particular keyblock and return as Vec<Privatekey>
    pub fn get_keys_as_private_for(&self, block_type: KeyBlock) -> SolDidResult<Vec<Privatekey>> {
        if self.keysets.contains_key(&block_type) {
            Ok(self
                .keysets
                .get(&block_type)
                .unwrap()
                .iter()
                .map(|k| Privatekey::from(k.key()))
                .collect())
        } else {
            Err(SolDidError::KeySetIncoherence)
        }
    }

    /// Gets a particular keyblock and return as Vec<String>
    pub fn get_keys_as_strings_for(&self, block_type: KeyBlock) -> SolDidResult<Vec<String>> {
        if self.keysets.contains_key(&block_type) {
            Ok(self
                .keysets
                .get(&block_type)
                .unwrap()
                .iter()
                .map(|k| k.key())
                .collect())
        } else {
            Err(SolDidError::KeySetIncoherence)
        }
    }
}

impl From<&EventMessage<SaidEvent<Event>>> for ChainEvent {
    fn from(event: &EventMessage<SaidEvent<Event>>) -> Self {
        let mut ce = ChainEvent::default();
        ce.km_sn = event.event.get_sn();
        ce.km_digest = event.get_digest().to_str();
        ce.event_type = ChainEventType::from(event.event.get_type());
        ce
    }
}

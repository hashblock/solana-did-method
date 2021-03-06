//! Chain event mirroring on keys for DID

use crate::errors::{SolDidError, SolDidResult};

use super::{wallet_enums::KeyType, Key};
use borsh::{BorshDeserialize, BorshSerialize};
use hbkr_rs::{
    event::Event, event_message::EventMessage, said_event::SaidEvent, EventTypeTag, Prefix,
    Typeable,
};
use std::collections::HashMap;

#[derive(BorshDeserialize, BorshSerialize, Clone, Copy, Debug, Default)]
pub enum ChainEventType {
    #[default]
    Inception,
    Rotation,
    DelegatedInception,
    DelegatedRotation,
    Revoked,
    Decommisioned,
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
    pub did_signature: String,
    pub km_sn: u64,
    pub km_digest: String,
    pub km_keytype: KeyType,
    pub keysets: HashMap<KeyBlock, Vec<Key>>,
}

impl ChainEvent {
    pub fn get_keys_for(&self, block_type: KeyBlock) -> SolDidResult<&Vec<Key>> {
        if self.keysets.contains_key(&block_type) {
            Ok(self.keysets.get(&block_type).unwrap())
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

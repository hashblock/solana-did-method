//! Chain event mirroring on keys for DID

use super::{wallet_enums::KeyType, Key};
use borsh::{BorshDeserialize, BorshSerialize};
use hbkr_rs::{event::Event, event_message::EventMessage, said_event::SaidEvent, Prefix};
use std::collections::HashMap;

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug)]
pub enum ChainEventType {
    Inception,
    Rotation,
    DelegatedInception,
    DelegatedRotation,
    Revoked,
    Decommisioined,
}

impl Default for ChainEventType {
    fn default() -> Self {
        ChainEventType::Inception
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

impl ChainEvent {}

impl From<&EventMessage<SaidEvent<Event>>> for ChainEvent {
    fn from(event: &EventMessage<SaidEvent<Event>>) -> Self {
        let mut ce = ChainEvent::default();
        ce.km_sn = event.event.get_sn();
        ce.km_digest = event.get_digest().to_str();
        ce
    }
}

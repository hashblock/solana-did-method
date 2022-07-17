//! Chain trait definition

use hbkr_rs::{event::Event, event_message::EventMessage, said_event::SaidEvent};

use crate::errors::SolDidResult;

/// DIdSigner is a type able to sign transactions
pub type DidSigner = Vec<u8>;
pub trait Chain {
    fn inception_inst(&self, event_msg: &EventMessage<SaidEvent<Event>>) -> SolDidResult<String>;
    fn rotation_inst_fn(&self, event_msg: &EventMessage<SaidEvent<Event>>) -> SolDidResult<String>;
    fn inst_signer(&self) -> DidSigner;
}

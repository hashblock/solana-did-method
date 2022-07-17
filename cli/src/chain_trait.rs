//! Chain trait definition

use hbkr_rs::{event::Event, event_message::EventMessage, said_event::SaidEvent};

use crate::errors::SolDidResult;

/// DIdSigner is a type able to sign transactions
pub type DidSigner = Vec<u8>;
pub type ChainSignature = String;
pub trait Chain {
    /// Inception instruction put on the chain
    fn inception_inst(
        &self,
        event_msg: &EventMessage<SaidEvent<Event>>,
    ) -> SolDidResult<ChainSignature>;
    /// Rotation instruction put on the chain
    fn rotation_inst_fn(
        &self,
        event_msg: &EventMessage<SaidEvent<Event>>,
    ) -> SolDidResult<ChainSignature>;
    /// Get the signer bytes
    fn inst_signer(&self) -> DidSigner;
    /// Get the chain URL in use
    fn url(&self) -> &String;
}

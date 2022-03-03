//! Miscellaneous utility functions

use borsh::BorshDeserialize;
use solana_client::rpc_client::RpcClient;
use solana_did_method::{
    instruction::{InceptionDID, SDMInstruction},
    state::SDMDidState,
};
use solana_sdk::{pubkey::PUBKEY_BYTES, signature::Signature};
use solana_transaction_status::UiTransactionEncoding;

use crate::{errors::SolKeriError, errors::SolKeriResult};

/// Fetches and decodes a transactions instruction data
pub fn instruction_from_transaction(
    connection: &RpcClient,
    signature: &Signature,
) -> SolKeriResult<SDMInstruction> {
    let tx_post = connection.get_transaction(&signature, UiTransactionEncoding::Base64);
    if tx_post.is_ok() {
        let dc = tx_post.unwrap().transaction.transaction.decode();
        // println!("{:?}", dc);
        match dc {
            Some(tx) => {
                println!("Proof instruction {:?}", tx.message.instructions[0]);
                println!("Program instruction {:?}", tx.message.instructions[1]);
                Ok(SDMInstruction::try_from_slice(
                    &tx.message.instructions[1].data,
                )?)
            }
            None => Err(SolKeriError::DecodeTransactionError),
        }
    } else {
        Err(SolKeriError::GetTransactionError)
    }
}

/// Calculate the size of the DID account state data size
/// based on number of keys being managed
pub fn get_inception_datasize(my_did: &InceptionDID) -> usize {
    0usize
        .saturating_add(std::mem::size_of::<bool>()) // Initialized
        .saturating_add(std::mem::size_of::<u16>()) // Version
        .saturating_add(std::mem::size_of::<SDMDidState>()) // State
        .saturating_add(PUBKEY_BYTES) // Prefix pubkey
        .saturating_add(std::mem::size_of::<u8>()) // bump
        .saturating_add(std::mem::size_of::<u32>())
        .saturating_add(PUBKEY_BYTES * my_did.keys.len())
}

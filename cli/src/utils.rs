//! Miscellaneous utility functions

use borsh::BorshDeserialize;
use solana_client::rpc_client::RpcClient;
use solana_did_method::{
    instruction::{InceptionDID, SDMInstruction},
    state::SDMDidState,
};
use solana_sdk::{
    account::Account,
    pubkey::{Pubkey, PUBKEY_BYTES},
    signature::Signature,
};
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
                // println!("Proof instruction {:?}", tx.message.instructions[0]);
                // println!("Program instruction {:?}", tx.message.instructions[1]);
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

/// Gets the did account from a did pda
pub fn get_did_pda_account(
    connection: &RpcClient,
    did_pda: &Pubkey,
) -> SolKeriResult<Option<Account>> {
    Ok(Some(connection.get_account(&did_pda)?))
}

/// Form a PDA public key
pub fn gen_pda_pk(prefix_digest: &Vec<u8>, project_pk: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[prefix_digest], &project_pk)
}

/// Gets the did account from a did pda
pub fn get_did_pda_account_from_digest(
    connection: &RpcClient,
    prefix_digest: &Vec<u8>,
    project_pk: &Pubkey,
) -> SolKeriResult<Option<Account>> {
    let (pda_pk, _bump) = gen_pda_pk(prefix_digest, project_pk);
    Ok(Some(connection.get_account(&pda_pk)?))
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
        .saturating_add(std::mem::size_of::<u32>()) // Borsh vector count
        .saturating_add(PUBKEY_BYTES * my_did.keys.len()) // Vector of keys size
}

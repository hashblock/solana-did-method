//! Utility functions

use borsh::BorshDeserialize;
use solana_client::rpc_client::RpcClient;
use solana_keri::instruction::SolKeriInstruction;
use solana_sdk::signature::Signature;
use solana_transaction_status::UiTransactionEncoding;

use crate::{errors::SolKeriCliError, SolKeriResult};

/// Fetches and decodes a transactions instruction data
pub fn instruction_from_transaction(
    connection: &RpcClient,
    signature: &Signature,
) -> SolKeriResult<SolKeriInstruction> {
    let tx_post = connection.get_transaction(&signature, UiTransactionEncoding::Base64);
    if tx_post.is_ok() {
        match tx_post.unwrap().transaction.transaction.decode() {
            Some(tx) => Ok(SolKeriInstruction::try_from_slice(
                &tx.message.instructions[0].data,
            )?),
            None => Err(SolKeriCliError::DecodeTransactionError),
        }
    } else {
        Err(SolKeriCliError::GetTransactionError)
    }
}

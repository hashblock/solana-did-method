//! Utility functions

use borsh::BorshDeserialize;
use solana_client::rpc_client::RpcClient;
use solana_did_method::instruction::SolKeriInstruction;
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
        let dc = tx_post.unwrap().transaction.transaction.decode();
        // println!("{:?}", dc);
        match dc {
            Some(tx) => {
                println!("Proof instruction {:?}", tx.message.instructions[0]);
                println!("Program instruction {:?}", tx.message.instructions[1]);
                Ok(SolKeriInstruction::try_from_slice(
                    &tx.message.instructions[1].data,
                )?)
            }
            None => Err(SolKeriCliError::DecodeTransactionError),
        }
    } else {
        Err(SolKeriCliError::GetTransactionError)
    }
}

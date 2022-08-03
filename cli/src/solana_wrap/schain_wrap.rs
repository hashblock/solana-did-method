//! Solana Chain wraps the interface and behavior for block chain

use std::{fmt::Debug, str::FromStr};

use crate::{
    chain_trait::{Chain, ChainSignature, DidSigner},
    errors::{SolDidError, SolDidResult},
};

use hbkr_rs::{
    event::Event,
    event_message::EventMessage,
    key_manage::{KeySet, PubKey, Publickey},
    said_event::SaidEvent,
    Prefix,
};

use solana_client::rpc_client::RpcClient;
use solana_did_method::{
    id,
    instruction::{
        DIDDecommission, DIDInception, DIDRotation, InitializeDidAccount, SDMInstruction,
        SMDKeyType,
    },
    state::SDMDidState,
};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    ed25519_instruction,
    instruction::{AccountMeta, CompiledInstruction, Instruction},
    message::Message,
    pubkey::{Pubkey, PUBKEY_BYTES},
    signature::{read_keypair_file, Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};
use solana_transaction_status::UiTransactionEncoding;

pub struct SolanaChain {
    rpc_url: String,
    rpc_client: RpcClient,
    signer: Keypair,
    program_id: Pubkey,
}

impl SolanaChain {
    /// Create a new chain instance with designated client and signer
    pub fn new(rpc_client: RpcClient, signer: Keypair, program_id: Option<Pubkey>) -> Self {
        let rpc_url = rpc_client.url();
        Self {
            rpc_url,
            rpc_client,
            signer,
            program_id: match program_id {
                Some(pk) => pk,
                None => id(),
            },
        }
    }
    /// Set the program ID from Publickey
    pub fn set_program_id_from_publickey(&mut self, from: &Publickey) -> SolDidResult<Publickey> {
        let last_pubkey = self.program_id();
        self.program_id = Pubkey::new(&from.to_bytes());
        Ok(last_pubkey)
    }
    /// Set the program ID from Publickey
    pub fn set_program_id_with_pubkey(&mut self, from: &Pubkey) -> SolDidResult<Publickey> {
        let last_pubkey = self.program_id();
        self.program_id = from.clone();
        Ok(last_pubkey)
    }
    /// Get the version of the chain node
    pub fn version(&self) -> semver::Version {
        let version = self.rpc_client.get_version().unwrap();
        semver::Version::parse(&version.solana_core).unwrap()
    }

    /// Generate a safe PDA account address
    /// Form a PDA public key
    fn safe_pda_from_digest(
        &self,
        prefix: &String,
        prefix_digest: &Vec<u8>,
    ) -> SolDidResult<(Pubkey, u8)> {
        let (pda_pk, bump) = Pubkey::find_program_address(&[prefix_digest], &self.program_id);
        let check_acc = self.rpc_client.get_account(&pda_pk);
        if check_acc.is_ok() {
            Err(SolDidError::DIDAccountExists(prefix.to_string()))
        } else {
            Ok((pda_pk, bump))
        }
    }

    /// Get the prefix as 32 byte array
    fn prefix_bytes(event_msg: &EventMessage<SaidEvent<Event>>) -> [u8; 32] {
        // Get prefix in bytes
        let mut prefix_bytes = [0u8; 32];
        match event_msg.event.get_prefix() {
            hbkr_rs::identifier_prefix::IdentifierPrefix::SelfAddressing(sa) => {
                prefix_bytes.copy_from_slice(&sa.digest)
            }
            _ => unreachable!(),
        }
        prefix_bytes
    }

    /// Submits a transaction with programs instruction
    fn submit_transaction(&self, instructions: Vec<Instruction>) -> SolDidResult<Signature> {
        let mut transaction =
            Transaction::new_unsigned(Message::new(&instructions, Some(&self.signer.pubkey())));
        let recent_blockhash = self.rpc_client.get_latest_blockhash().unwrap();
        transaction
            .try_sign(&vec![&self.signer], recent_blockhash)
            .unwrap();
        Ok(self.rpc_client.send_and_confirm_transaction(&transaction)?)
    }
    /// Fetches and decodes a transactions instruction data
    pub fn inception_instructions_from_transaction(
        &self,
        signature: &String,
    ) -> SolDidResult<Vec<CompiledInstruction>> {
        let signature = Signature::from_str(signature).unwrap();
        let tx_post = self
            .rpc_client
            .get_transaction(&signature, UiTransactionEncoding::Base64);
        if tx_post.is_ok() {
            let dc = tx_post.unwrap().transaction.transaction.decode();
            match dc {
                Some(tx) => Ok([
                    tx.message.instructions()[0].clone(),
                    tx.message.instructions()[1].clone(),
                ]
                .to_vec()),
                None => Err(SolDidError::DecodeTransactionError),
            }
        } else {
            Err(SolDidError::GetTransactionError)
        }
    }
}

/// Default implementation for SolanaChain
impl Default for SolanaChain {
    fn default() -> Self {
        let cli_config = match &*solana_cli_config::CONFIG_FILE {
            Some(cfgpath) => solana_cli_config::Config::load(&cfgpath).unwrap(),
            None => solana_cli_config::Config::default(),
        };
        Self {
            rpc_client: RpcClient::new_with_commitment(
                cli_config.json_rpc_url.clone(),
                CommitmentConfig::confirmed(),
            ),
            rpc_url: cli_config.json_rpc_url.clone(),
            signer: read_keypair_file(cli_config.keypair_path).unwrap(),
            program_id: id(),
        }
    }
}

/// Debug for SolanaChain
impl Debug for SolanaChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SolanaChain")
            .field("rpc_url", &self.rpc_url)
            // .field("rpc_client", &self.rpc_client)
            .field("signer", &self.signer)
            .field("program_id", &self.program_id)
            .finish()
    }
}

/// Calculate the size of the DID account state data size
/// based on number of keys being managed
pub fn get_inception_datasize(key_count: usize) -> usize {
    0usize
        .saturating_add(std::mem::size_of::<bool>()) // Initialized
        .saturating_add(std::mem::size_of::<u16>()) // Version
        .saturating_add(std::mem::size_of::<SDMDidState>()) // State
        .saturating_add(PUBKEY_BYTES) // Prefix pubkey
        .saturating_add(std::mem::size_of::<u8>()) // bump
        .saturating_add(std::mem::size_of::<u32>()) // Borsh vector count
        .saturating_add(PUBKEY_BYTES * key_count) // Vector of keys size
}

const DID_INCEPT_RENT_MULTIPLIER: u64 = 10;

/// Chain trait implementation
impl Chain for SolanaChain {
    /// Inception
    fn inception_inst(
        &self,
        key_set: &dyn KeySet,
        event_msg: &EventMessage<SaidEvent<Event>>,
    ) -> SolDidResult<ChainSignature> {
        // Verify prefix is not already a PDA collision
        // Create a PDA for our DID
        let digest_bytes = event_msg.get_digest().digest;
        let prefix = event_msg.event.get_prefix().to_str();
        let (pda_key, bump) = self.safe_pda_from_digest(&prefix, &digest_bytes)?;
        // Now we want to create two (2) instructions:
        // 1. The ed25519 signature verification on the serialized message
        let verify_instruction = ed25519_instruction::new_ed25519_instruction(
            &ed25519_dalek::Keypair::from_bytes(&self.signer.to_bytes())?,
            &event_msg.serialize()?,
        );
        // 2. The inception instruction of the DID for program
        // Convert pasta keys to Solana Pubkey for serialization
        let keys = key_set
            .current_public_keys()
            .iter()
            .map(|k| Pubkey::from_str(&k.as_base58_string()).unwrap())
            .collect::<Vec<Pubkey>>();
        if keys.len() == 0 {
            return Err(SolDidError::DIDInvalidInceptionZeroKeys);
        }

        // Get prefix in bytes
        let prefix_bytes = SolanaChain::prefix_bytes(event_msg);
        // Setup DID inception data
        let data_size = get_inception_datasize(keys.len()) * DID_INCEPT_RENT_MULTIPLIER as usize;
        let did_account = DIDInception {
            keytype: SMDKeyType::PASTA,
            prefix: prefix_bytes,
            bump,
            keys,
        };

        // Get rent calc
        let rent_exemption_amount = self
            .rpc_client
            .get_minimum_balance_for_rent_exemption(data_size)?;
        // TODO - We are paying more in rent than the size of the data which
        // may grow due to rotation variations
        let init = InitializeDidAccount {
            rent: DID_INCEPT_RENT_MULTIPLIER * rent_exemption_amount,
            storage: data_size as u64,
        };
        // Accounts to pass to instruction
        let accounts = &[
            AccountMeta::new(self.signer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new(solana_sdk::system_program::id(), false),
        ];
        // Build instruction array and submit transaction
        let txn = self.submit_transaction(
            [
                verify_instruction,
                Instruction::new_with_borsh(
                    self.program_id,
                    &SDMInstruction::SDMInception(init, did_account),
                    accounts.to_vec(),
                ),
            ]
            .to_vec(),
        );
        assert!(txn.is_ok());
        let signature = txn.unwrap();
        Ok(signature.to_string())
    }

    /// Rotation
    fn rotation_inst(
        &self,
        inception_digest: &Vec<u8>,
        key_set: &dyn KeySet,
        event_msg: &EventMessage<SaidEvent<Event>>,
    ) -> SolDidResult<ChainSignature> {
        // Validate we have a did
        let (pda_key, _bump) = Pubkey::find_program_address(&[inception_digest], &self.program_id);
        let check_acc = self.rpc_client.get_account(&pda_key);
        if check_acc.is_err() {
            return Err(SolDidError::DIDAccountNotExists(pda_key.to_string()));
        }
        let _rotation_digest = event_msg.get_digest().digest;
        // Now we want to create two (2) instructions:
        // 1. The ed25519 signature verification on the serialized message
        let verify_instruction = ed25519_instruction::new_ed25519_instruction(
            &ed25519_dalek::Keypair::from_bytes(&self.signer.to_bytes())?,
            &event_msg.serialize()?,
        );
        // 2. The rotation instruction of the DID for program
        // Convert pasta keys to Solana Pubkey for serialization
        let keys = key_set
            .current_public_keys()
            .iter()
            .map(|k| Pubkey::from_str(&k.as_base58_string()).unwrap())
            .collect::<Vec<Pubkey>>();
        if keys.len() == 0 {
            return Err(SolDidError::DIDInvalidRotationUseDecommision);
        }
        // Create the instruction data
        let did_rotation = DIDRotation {
            keytype: SMDKeyType::PASTA,
            prefix: SolanaChain::prefix_bytes(event_msg),
            keys,
        };
        // Accounts to pass to instruction
        let accounts = &[
            AccountMeta::new(self.signer.pubkey(), true),
            AccountMeta::new(pda_key, false),
        ];
        let txn = self.submit_transaction(
            [
                verify_instruction,
                Instruction::new_with_borsh(
                    self.program_id,
                    &SDMInstruction::SDMRotation(did_rotation),
                    accounts.to_vec(),
                ),
            ]
            .to_vec(),
        );
        println!("ROT Txn {:?}", txn);
        assert!(txn.is_ok());
        let signature = txn.unwrap();
        Ok(signature.to_string())
    }

    /// Decommission
    fn decommission_inst(
        &self,
        inception_digest: &Vec<u8>,
        key_set: &dyn KeySet,
        event_msg: &EventMessage<SaidEvent<Event>>,
    ) -> SolDidResult<ChainSignature> {
        // Validate we have a did
        let (pda_key, _bump) = Pubkey::find_program_address(&[inception_digest], &self.program_id);
        let check_acc = self.rpc_client.get_account(&pda_key);
        if check_acc.is_err() {
            return Err(SolDidError::DIDAccountNotExists(pda_key.to_string()));
        }
        // Now we want to create two (2) instructions:
        // 1. The ed25519 signature verification on the serialized message
        let verify_instruction = ed25519_instruction::new_ed25519_instruction(
            &ed25519_dalek::Keypair::from_bytes(&self.signer.to_bytes())?,
            &event_msg.serialize()?,
        );
        // 2. The decommission instruction of the DID for program
        // Convert pasta keys to Solana Pubkey for serialization
        let keys = key_set
            .current_public_keys()
            .iter()
            .map(|k| Pubkey::from_str(&k.as_base58_string()).unwrap())
            .collect::<Vec<Pubkey>>();
        if keys.len() == 0 {
            return Err(SolDidError::DIDInvalidRotationUseDecommision);
        }
        // Create the instruction data
        let did_decomm = DIDDecommission {
            keytype: SMDKeyType::PASTA,
            prefix: SolanaChain::prefix_bytes(event_msg),
            keys,
        };
        // Accounts to pass to instruction
        let accounts = &[
            AccountMeta::new(self.signer.pubkey(), true),
            AccountMeta::new(pda_key, false),
        ];
        let txn = self.submit_transaction(
            [
                verify_instruction,
                Instruction::new_with_borsh(
                    self.program_id,
                    &SDMInstruction::SDMDecommission(did_decomm),
                    accounts.to_vec(),
                ),
            ]
            .to_vec(),
        );
        println!("ROT Txn {:?}", txn);
        assert!(txn.is_ok());
        let signature = txn.unwrap();
        Ok(signature.to_string())
    }

    fn inst_signer(&self) -> DidSigner {
        self.signer.to_bytes().to_vec()
    }

    fn url(&self) -> &String {
        &self.rpc_url
    }

    fn program_id(&self) -> hbkr_rs::key_manage::Publickey {
        hbkr_rs::key_manage::Publickey::new(self.program_id.to_bytes().to_vec())
    }
}

#[cfg(test)]
mod chain_tests {
    use super::*;
    use crate::errors::SolDidResult;

    #[test]
    fn test_chain_default_pass() -> SolDidResult<()> {
        let mchain = SolanaChain::default();
        assert_eq!(mchain.program_id, id());
        Ok(())
    }
}

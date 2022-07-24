//! Solana Chain wraps the interface and behavior for block chain

use crate::{
    chain_trait::{Chain, ChainSignature, DidSigner},
    errors::SolDidResult,
};

use hbkr_rs::{
    event::Event, event_message::EventMessage, key_manage::Publickey, said_event::SaidEvent,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_did_method::id;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair},
};

pub struct SolanaChain {
    rpc_url: String,
    rpc_client: RpcClient,
    signer: Keypair,
    program_id: Pubkey,
}

impl SolanaChain {
    /// Create a new chain instance with designated client and signer
    pub fn new(rpc_client: RpcClient, signer: Keypair, program_id: Pubkey) -> Self {
        let rpc_url = rpc_client.url();
        Self {
            rpc_url,
            rpc_client,
            signer,
            program_id,
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
    pub async fn version(&self) -> semver::Version {
        let version = self.rpc_client.get_version().await.unwrap();
        semver::Version::parse(&version.solana_core).unwrap()
    }
}

/// Default implementation for SolanaChain
impl Default for SolanaChain {
    fn default() -> Self {
        let cli_config = match &*solana_cli_config::CONFIG_FILE {
            Some(cfgpath) => solana_cli_config::Config::load(&cfgpath).unwrap(),
            None => solana_cli_config::Config::default(),
        };
        let rpc_url = cli_config.json_rpc_url.clone();
        let default_signer = read_keypair_file(cli_config.keypair_path).unwrap();
        let rpc_client =
            RpcClient::new_with_commitment(cli_config.json_rpc_url, CommitmentConfig::confirmed());

        Self {
            rpc_client: rpc_client,
            rpc_url,
            signer: default_signer,
            program_id: id(),
        }
    }
}

/// Chain trait implementation
impl Chain for SolanaChain {
    fn inception_inst(
        &self,
        _event_msg: &EventMessage<SaidEvent<Event>>,
    ) -> SolDidResult<ChainSignature> {
        todo!()
    }

    fn rotation_inst(
        &self,
        _event_msg: &EventMessage<SaidEvent<Event>>,
    ) -> SolDidResult<ChainSignature> {
        todo!()
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

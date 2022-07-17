//! Solana Chain wraps the interface and behavior for block chain

use crate::{
    chain_trait::{Chain, ChainSignature, DidSigner},
    errors::SolDidResult,
};

use hbkr_rs::{event::Event, event_message::EventMessage, said_event::SaidEvent};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{read_keypair_file, Keypair},
};

pub struct SolanaChain {
    rpc_url: String,
    rpc_client: RpcClient,
    signer: Keypair,
}

impl SolanaChain {
    /// Create a new chain instance with designated client and signer
    pub fn new(rpc_client: RpcClient, signer: Keypair) -> Self {
        let rpc_url = rpc_client.url();
        Self {
            rpc_url,
            rpc_client,
            signer,
        }
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

    fn rotation_inst_fn(
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
}

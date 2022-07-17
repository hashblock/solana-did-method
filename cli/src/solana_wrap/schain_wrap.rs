//! Solana Chain wraps the interface and behavior for block chain

use crate::{
    chain_trait::{Chain, DidSigner},
    errors::SolDidResult,
};

use hbkr_rs::{event::Event, event_message::EventMessage, said_event::SaidEvent};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{read_keypair_file, Keypair},
};

pub struct SolanaChain {
    rpc_client: RpcClient,
    signer: Keypair,
}

impl SolanaChain {}

impl Default for SolanaChain {
    fn default() -> Self {
        let cli_config = match &*solana_cli_config::CONFIG_FILE {
            Some(cfgpath) => solana_cli_config::Config::load(&cfgpath).unwrap(),
            None => solana_cli_config::Config::default(),
        };
        let default_signer = read_keypair_file(cli_config.keypair_path).unwrap();
        let rpc_client =
            RpcClient::new_with_commitment(cli_config.json_rpc_url, CommitmentConfig::confirmed());

        Self {
            rpc_client: rpc_client,
            signer: default_signer,
        }
    }
}

impl Chain for SolanaChain {
    fn inception_inst(&self, event_msg: &EventMessage<SaidEvent<Event>>) -> SolDidResult<String> {
        todo!()
    }

    fn rotation_inst_fn(&self, event_msg: &EventMessage<SaidEvent<Event>>) -> SolDidResult<String> {
        todo!()
    }

    fn inst_signer(&self) -> DidSigner {
        self.signer.to_bytes().to_vec()
    }
}

//! cli for managing sol::keri dids and keys
mod clparse;
pub mod errors;

use cli::errors::{SolDidError, SolDidResult};
// pub use errors::SolDidResult;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signer::{keypair::read_keypair_file, Signer},
};

use crate::clparse::command_line;

/// Generates the default RpcClient and Signer from local configuration
fn default_config() -> SolDidResult<(RpcClient, Box<dyn Signer>)> {
    let cli_config = match &*solana_cli_config::CONFIG_FILE {
        Some(cfgpath) => solana_cli_config::Config::load(&cfgpath)?,
        None => return Err(SolDidError::SolanaConfigMissing),
    };
    let default_signer = read_keypair_file(cli_config.keypair_path)?;
    let rpc_client =
        RpcClient::new_with_commitment(cli_config.json_rpc_url, CommitmentConfig::confirmed());
    Ok((rpc_client, Box::new(default_signer)))
}
#[tokio::main]
async fn main() -> SolDidResult<()> {
    let cmdline = command_line().get_matches();
    let (_command, _matches) = cmdline.subcommand().unwrap();
    let (_client, _signer) = default_config()?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use borsh::BorshSerialize;
    use cli::errors::SolDidResult;
    use solana_sdk::pubkey::Pubkey;

    use crate::default_config;

    #[derive(BorshSerialize, Debug)]
    struct FauxAccount {
        prefix: Pubkey,
        keys: Vec<Pubkey>,
    }

    #[test]
    fn test_main() -> SolDidResult<()> {
        let (client, signer) = default_config()?;
        println!("{:?}", client.url());
        println!("{:?}", signer);
        Ok(())
    }
    #[test]
    fn test_serialization() {
        let dummy_pk = Pubkey::from_str("SDMHqNqN82QSjEaEuqybmpXsjtX98YuTsX6YCdT99to").unwrap();
        let dummy_pk1 = Pubkey::from_str("FDMHqNqN82QSjEaEuqybmpXsjtX98YuTsX6YCdT99to").unwrap();
        let dummy_pk2 = Pubkey::from_str("HDMHqNqN82QSjEaEuqybmpXsjtX98YuTsX6YCdT99to").unwrap();

        let mut keys = Vec::<Pubkey>::new();
        for i in 0..2 {
            if i == 0 {
                keys.push(dummy_pk1)
            } else {
                keys.push(dummy_pk2)
            }
        }
        let faux_account = FauxAccount {
            prefix: dummy_pk,
            keys,
        };
        let y = 33 + (2 * 32) + 1;
        let z = std::mem::size_of_val(&faux_account);
        let saccount = faux_account.try_to_vec().unwrap();
        let w = saccount.len();
        println!("size calc {y} or size mem {z} = {w}");
    }
}

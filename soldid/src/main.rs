//! cli for managing sol::keri dids and keys
mod clparse;
pub mod errors;

use clap::ArgMatches;
use soldid::{
    errors::SolDidResult,
    pkey_wrap::PastaKeySet,
    solana_wrap::schain_wrap::SolanaChain,
    wallet::{init_wallet, load_wallet_from, Wallet},
};

use crate::clparse::{command_line, DID_CREATE, DID_DECOMMISION, DID_LIST, DID_ROTATE};

/// List the keys and their prefixes
fn list_dids(wallet: &Wallet) -> SolDidResult<()> {
    let wkeys = wallet.keys()?;
    if wkeys.len() > 0 {
        for keys in wallet.keys()? {
            println!("Keys: {} has prefix {}", keys.name(), keys.prefix())
        }
    } else {
        println!("No DID keysets exist at this time");
    }
    Ok(())
}

/// Create a new DID extracts the name, keycount and threshold arguments
fn create_did(
    wallet: &mut Wallet,
    matches: &ArgMatches,
    schain: &mut SolanaChain,
) -> SolDidResult<(String, String, Vec<u8>)> {
    // wallet.new_did()
    let key_count = *matches.get_one::<i8>("keys").unwrap();
    let threshold = *matches.get_one::<i8>("threshold").unwrap();
    let kset_name = &*matches.get_one::<String>("name").unwrap();
    let kset = PastaKeySet::new_for(key_count);
    wallet.new_did(kset_name, &kset, threshold, Some(schain))

    // Ok(())
}

/// Rotate a new DID
fn simple_rotate_did(
    _wallet: &mut Wallet,
    _matches: &ArgMatches,
    _schain: &mut SolanaChain,
) -> SolDidResult<()> {
    Ok(())
}

/// Decommision a new DID
fn decommision_did(
    _wallet: &mut Wallet,
    _matches: &ArgMatches,
    _schain: &mut SolanaChain,
) -> SolDidResult<()> {
    Ok(())
}

#[tokio::main]
async fn main() -> SolDidResult<()> {
    // Parse command line
    let cmdline = command_line().get_matches();

    // Load chain wrapper
    let mut chain = SolanaChain::default();
    // Load default wallet or use user provided command line wallet path
    let mut wallet = match cmdline.value_source("wallet").unwrap() {
        clap::ValueSource::DefaultValue => init_wallet()?,
        clap::ValueSource::CommandLine => load_wallet_from(cmdline.get_one("wallet").unwrap())?,
        _ => todo!(),
    };
    let (command, matches) = cmdline.subcommand().unwrap();
    match command {
        DID_LIST => list_dids(&wallet)?,
        DID_CREATE => {
            let _res = create_did(&mut wallet, matches, &mut chain)?;
            {}
        }
        DID_ROTATE => simple_rotate_did(&mut wallet, matches, &mut chain)?,
        DID_DECOMMISION => decommision_did(&mut wallet, matches, &mut chain)?,
        _ => {}
    }

    Ok(())
}

//! cli for managing sol::keri dids and keys
mod clparse;
pub mod errors;

use std::str::FromStr;

use clap::ArgMatches;
use clparse::DID_CLOSE;
use solana_did_method::state::SDMDid;
use solana_sdk::{borsh::try_from_slice_unchecked, pubkey::Pubkey};
use soldid::{
    errors::SolDidResult,
    pkey_wrap::PastaKeySet,
    solana_wrap::schain_wrap::SolanaChain,
    wallet::{init_wallet, load_wallet_from, Wallet},
};

use crate::clparse::{command_line, DID_CREATE, DID_DECOMMISION, DID_LIST, DID_ROTATE};

/// List the keys and their prefixes
fn list_dids(wallet: &Wallet, schain: &mut SolanaChain) -> SolDidResult<()> {
    let wkeys = wallet.keys()?;
    for (pkey, acc) in schain.get_dids() {
        let adata = try_from_slice_unchecked::<SDMDid>(&acc.data).unwrap();
        //     let dbuf = base64::decode(acc.data);
        //     // let dbuf = bs58::decode(acc.data).into_vec().unwrap();
        //     if dbuf.is_ok() {
        //         let adata = try_from_slice_unchecked::<SDMDid>(&dbuf.unwrap()).unwrap();
        println!("DID pubkey {:?}", pkey);
        println!("DID account {:?}", adata);
        //     } else {
        //         println!("Decode error");
        //         dbuf.unwrap();
        //     }
    }
    if wkeys.len() > 0 {
        for keys in wallet.keys()? {
            println!(
                "Keys: {} has prefix {} at account {:?}",
                keys.name(),
                keys.prefix(),
                keys.account()
            )
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

/// Close the DID account on the chain
fn close_did(
    _wallet: &mut Wallet,
    matches: &ArgMatches,
    schain: &mut SolanaChain,
) -> SolDidResult<()> {
    let pda_key = &*matches.get_one::<String>("pda").unwrap();
    let sol_pk = Pubkey::from_str(pda_key).unwrap();
    schain.close_did(&sol_pk)?;
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
        DID_LIST => list_dids(&wallet, &mut chain)?,
        DID_CREATE => {
            let _res = create_did(&mut wallet, matches, &mut chain)?;
            {}
        }
        DID_ROTATE => simple_rotate_did(&mut wallet, matches, &mut chain)?,
        DID_DECOMMISION => decommision_did(&mut wallet, matches, &mut chain)?,
        DID_CLOSE => close_did(&mut wallet, matches, &mut chain)?,
        _ => {}
    }

    Ok(())
}

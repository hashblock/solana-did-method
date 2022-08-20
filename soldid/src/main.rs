//! cli for managing sol::keri dids and keys
mod clparse;
pub mod errors;

use std::str::FromStr;

use chrono::TimeZone;
use clap::ArgMatches;
use clparse::{DID_CLOSE, KEYS_LIST};
use hbkr_rs::key_manage::PubKey;
use solana_did_method::state::SDMDid;
use solana_sdk::{borsh::try_from_slice_unchecked, pubkey::Pubkey};
use soldid::{
    errors::SolDidResult,
    pkey_wrap::PastaKeySet,
    solana_wrap::schain_wrap::SolanaChain,
    wallet::{generic_keys::Keys, init_wallet, load_wallet_from, Wallet},
};

use crate::clparse::{command_line, DID_CREATE, DID_DECOMMISION, DID_LIST, DID_ROTATE};

/// List the keys and their prefixes
fn list_dids(wallet: &Wallet, schain: &mut SolanaChain) -> SolDidResult<()> {
    let wkeys = wallet.keys()?;
    if wkeys.len() > 0 {
        for keys in wallet.keys()? {
            let did_pk = Pubkey::from_str(&keys.account().as_base58_string()).unwrap();
            println!(
                "Getting DID document for '{}' at account {:?}",
                keys.name(),
                did_pk,
            );
            let did_acc = schain.get_did(&did_pk);
            let adata = try_from_slice_unchecked::<SDMDid>(&did_acc.data).unwrap();
            println!("DID account {:?}", adata);
        }
    } else {
        println!("No DID keysets exist");
    }
    Ok(())
}

/// Print key set information
fn display_keys(keyset: &Keys, detail: Option<&bool>) {
    let ces = keyset.chain_events();
    let v = chrono::Utc;

    println!("Keys");
    println!("----");
    println!("Name:    {}", keyset.name());
    println!("Prefix:  {}", keyset.prefix());
    println!(
        "Account: {:?}",
        Pubkey::from_str(keyset.account().as_base58_string().as_str()).unwrap()
    );
    if *detail.unwrap() {
        println!("\nEvents");
        println!("-----");
        for ce in ces {
            println!("Event type:     {:?}", ce.event_type);
            println!("Tx signature:   {}", ce.did_signature);
            println!("Datetime (UTC): {}", v.timestamp_millis(ce.time_stamp));
        }
    } else {
        let lce = ces.last().unwrap();
        println!("\nLast Event");
        println!("---- -----");
        println!("Event type:     {:?}", lce.event_type);
        println!("Tx signature:   {}", lce.did_signature);
        println!("Datetime (UTC): {}", v.timestamp_millis(lce.time_stamp));
        println!("Datetime (UTC): {}", lce.time_stamp);
    }
    // println!("{:?}", keyset);
}

fn display_all_keys(wallet: &Wallet, detail: Option<&bool>) {
    for keys in wallet.keys().unwrap() {
        display_keys(keys, detail)
    }
}
/// List keys in wallet and optionally detail the change log
fn list_keys(wallet: &Wallet, matches: &ArgMatches) -> SolDidResult<()> {
    let full_changes = matches.get_one::<bool>("changes");
    let kset_name = matches.get_one::<String>("name");
    if kset_name.is_some() {
        match kset_name.unwrap().as_str() {
            "all" => display_all_keys(wallet, full_changes),
            _ => display_keys(wallet.keys_for_name(kset_name.unwrap())?, full_changes),
        }
        // display_keys(wallet.keys_for_name(kset_name.unwrap())?, full_changes);
    } else {
        println!("\nKeys named in wallet");
        println!("----------------------");
        for key_set in wallet.keys()? {
            println!("{}", key_set.name());
        }
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
        KEYS_LIST => list_keys(&wallet, matches)?,
        _ => {}
    }

    Ok(())
}

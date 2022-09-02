// #[cfg(test)]
// mod tests {
use borsh::BorshDeserialize;
use hbkr_rs::key_manage::KeySet;
use solana_did_method::{id, instruction::SDMInstruction};
use solana_rpc::rpc::JsonRpcConfig;
use solana_sdk::{
    // ed25519_instruction,
    pubkey::Pubkey,
    signature::Keypair,
};
use solana_test_validator::{TestValidator, TestValidatorGenesis};
use soldid::{
    chain_trait::Chain,
    errors::{SolDidError, SolDidResult},
    pkey_wrap::PastaKeySet,
    solana_wrap::schain_wrap::SolanaChain,
    wallet::{load_wallet_from, Wallet},
};
use std::{
    env, fs,
    path::{Path, PathBuf},
    str::FromStr,
    thread::sleep,
    time::Duration,
};

/// Location/Name of ProgramTestGenesis ledger
const LEDGER_PATH: &str = "./.ledger";
/// Path to BPF program (*.so)
const PROG_PATH: &str = "../target/deploy/";
/// Program name from program/Cargo.toml
const PROG_NAME: &str = "solana_did_method";

/// Test wallet core path
const TEST_WALLET_LOCATION: &str = "/.solwall_test";

/// Setup the test validator with predefined properties
fn setup_validator() -> SolDidResult<(TestValidator, Keypair, Pubkey)> {
    // Extend environment variable to include our program location
    std::env::set_var("BPF_OUT_DIR", PROG_PATH);
    // Instantiate the test validator
    let mut test_validator = TestValidatorGenesis::default();
    // Once instantiated, TestValidatorGenesis configuration functions follow
    // a builder pattern enabling chaining of settings function calls
    let (test_validator, kp) = test_validator
        // Set the ledger path and name
        // maps to `solana-test-validator --ledger <DIR>`
        .ledger_path(LEDGER_PATH)
        // Load our program. Ignored if reusing ledger
        // maps to `solana-test-validator --bpf-program <ADDRESS_OR_PATH BPF_PROGRAM.SO>`
        .add_program(PROG_NAME, id())
        // Start the test validator
        .rpc_config(JsonRpcConfig {
            enable_rpc_transaction_history: true,
            // enable_extended_tx_metadata_storage: true,
            ..JsonRpcConfig::default_for_test()
        })
        .start();
    Ok((test_validator, kp, id()))
}

/// Convenience function to remove existing ledger before TestValidatorGenesis setup
/// maps to `solana-test-validator ... --reset`
fn clean_ledger_setup_validator() -> SolDidResult<(TestValidator, Keypair, Pubkey)> {
    if PathBuf::from_str(LEDGER_PATH).unwrap().exists() {
        std::fs::remove_dir_all(LEDGER_PATH).unwrap();
    }
    setup_validator()
}

// Generate a test wallet
fn build_test_wallet() -> SolDidResult<Wallet> {
    let location = match env::var("HOME") {
        Ok(val) => val + TEST_WALLET_LOCATION,
        Err(_) => return Err(SolDidError::HomeNotFoundError),
    };
    let wpath = Path::new(&location).to_path_buf();
    load_wallet_from(&wpath)
}

// remove a test wallet
fn remove_test_wallet(wallet: Wallet) -> SolDidResult<()> {
    fs::remove_dir_all(wallet.full_path().parent().unwrap())?;
    Ok(())
}

// Simplify for test usage
fn build_and_run_inception(
    vchain: &dyn Chain,
    wallet: &mut Wallet,
    key_count: i8,
    key_threshold: u64,
) -> SolDidResult<(String, String, Vec<u8>)> {
    // Get key_count keys
    let kset1 = PastaKeySet::new_for(key_count);
    assert!(!kset1.is_barren());
    let keys_name = "Franks First".to_string();
    wallet.new_did(&keys_name, &kset1, key_threshold as i8, Some(vchain))
}

#[test]
fn test_wallet_test_location_pass() -> SolDidResult<()> {
    let build_wallet = build_test_wallet();
    assert!(build_wallet.is_ok());
    let wallet = build_wallet.unwrap();
    assert_eq!(wallet.keys()?.len(), 0);
    let drop_wallet = remove_test_wallet(wallet);
    assert!(drop_wallet.is_ok());
    Ok(())
}
#[test]
fn test_basic_test_chain_pass() -> SolDidResult<()> {
    let (test_validator, payer, _program_pk) = clean_ledger_setup_validator()?;
    let mchain = SolanaChain::new(test_validator.get_rpc_client(), payer, None);
    let vchain = mchain.version();
    assert_eq!(vchain.major, 1);
    assert_eq!(vchain.minor, 11);
    Ok(())
}

#[test]
fn test_pasta_inception_pass() -> SolDidResult<()> {
    // Get the test validator running
    let (test_validator, payer, _program_pk) = clean_ledger_setup_validator()?;
    // Get the SolanaChain setup
    let mchain = SolanaChain::new(test_validator.get_rpc_client(), payer, None);
    // Initialize an empty wallet
    let mut wallet = build_test_wallet()?;
    // Capture our programs log statements
    // ***************** UNCOMMENT NEXT LINE TO SEE LOGS
    // solana_logger::setup_with_default("solana_runtime::message=debug");

    // Incept keys
    let result = build_and_run_inception(&mchain, &mut wallet, 2i8, 1u64);
    if result.is_err() {
        println!("Failed inception");
    } else {
        let (signature, _, _) = result?;
        sleep(Duration::from_secs(20));
        let sdata = mchain.inception_instructions_from_transaction(&signature);
        if sdata.is_ok() {
            let sdata = sdata?;
            assert_eq!(sdata.len(), 2);
            let sdm_inst = SDMInstruction::try_from_slice(&sdata[1].data)?;
            println!("Incpepted: {:?}", sdm_inst);
        }
    }
    remove_test_wallet(wallet)?;
    Ok(())
}

#[test]
fn test_pasta_rotation_pass() -> SolDidResult<()> {
    // Get the test validator running
    let (test_validator, payer, _program_pk) = clean_ledger_setup_validator()?;
    // Get the SolanaChain setup
    let mchain = SolanaChain::new(test_validator.get_rpc_client(), payer, None);
    // Initialize an empty wallet
    let mut wallet = build_test_wallet()?;

    // Incept keys
    let result = build_and_run_inception(&mchain, &mut wallet, 2i8, 1u64);
    if result.is_err() {
        println!("Failed inception");
    } else {
        let (_signature, prefix, _) = result?;
        let mut barren_ks = PastaKeySet::new_empty();
        sleep(Duration::from_secs(5));
        // Capture our programs log statements
        // ***************** UNCOMMENT NEXT LINE TO SEE LOGS
        // solana_logger::setup_with_default("solana_runtime::message=debug");

        let result = wallet.rotate_did_with_prefix(
            prefix.clone(),
            &mut barren_ks,
            None,
            None,
            Some(&mchain),
        );
        assert!(result.is_ok());
    }
    remove_test_wallet(wallet)?;
    Ok(())
}

#[test]
fn test_pasta_decommission_pass() -> SolDidResult<()> {
    // Get the test validator running
    let (test_validator, payer, _program_pk) = clean_ledger_setup_validator()?;
    // Get the SolanaChain setup
    let mchain = SolanaChain::new(test_validator.get_rpc_client(), payer, None);
    // Initialize an empty wallet
    let mut wallet = build_test_wallet()?;
    // Incept keys
    let result = build_and_run_inception(&mchain, &mut wallet, 2i8, 1u64);
    if result.is_err() {
        println!("Failed inception");
    } else {
        let (_signature, prefix, _) = result?;
        let mut barren_ks = PastaKeySet::new_empty();
        sleep(Duration::from_secs(5));
        // Capture our programs log statements
        // ***************** UNCOMMENT NEXT LINE TO SEE LOGS
        // solana_logger::setup_with_default("solana_runtime::message=debug");
        let result =
            wallet.decommission_did_with_prefix(prefix.clone(), &mut barren_ks, Some(&mchain));
        if result.is_err() {
            println!("Failed decommision");
        }
    }
    remove_test_wallet(wallet)?;
    Ok(())
}
// }

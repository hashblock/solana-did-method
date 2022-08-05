// #[cfg(test)]
// mod tests {
use borsh::BorshDeserialize;
use cli::{
    chain_trait::Chain,
    errors::SolDidResult,
    pkey_wrap::PastaKeySet,
    solana_wrap::schain_wrap::SolanaChain,
    wallet::{init_wallet, Wallet},
};
use hbkr_rs::key_manage::KeySet;
use solana_did_method::{id, instruction::SDMInstruction};
use solana_rpc::rpc::JsonRpcConfig;
use solana_sdk::{
    // ed25519_instruction,
    pubkey::Pubkey,
    signature::Keypair,
};
use solana_test_validator::{TestValidator, TestValidatorGenesis};
use std::{fs, path::PathBuf, str::FromStr, thread::sleep, time::Duration};

/// Location/Name of ProgramTestGenesis ledger
const LEDGER_PATH: &str = "./.ledger";
/// Path to BPF program (*.so)
const PROG_PATH: &str = "../target/deploy/";
/// Program name from program/Cargo.toml
const PROG_NAME: &str = "solana_did_method";

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
            enable_cpi_and_log_storage: true,
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
    wallet.new_did(&kset1, key_threshold, Some(vchain))
}

#[test]
fn test_basic_test_chain_pass() -> SolDidResult<()> {
    let (test_validator, payer, _program_pk) = clean_ledger_setup_validator()?;
    let mchain = SolanaChain::new(test_validator.get_rpc_client(), payer, None);
    let vchain = mchain.version();
    assert_eq!(vchain.major, 1);
    assert_eq!(vchain.minor, 10);
    Ok(())
}

#[test]
fn test_pasta_inception_pass() -> SolDidResult<()> {
    // Get the test validator running
    let (test_validator, payer, _program_pk) = clean_ledger_setup_validator()?;
    // Get the SolanaChain setup
    let mchain = SolanaChain::new(test_validator.get_rpc_client(), payer, None);
    // Initialize an empty wallet
    let mut wallet = init_wallet()?;
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
    fs::remove_dir_all(wallet.full_path().parent().unwrap())?;
    Ok(())
}

#[test]
fn test_pasta_rotation_pass() -> SolDidResult<()> {
    // Get the test validator running
    let (test_validator, payer, _program_pk) = clean_ledger_setup_validator()?;
    // Get the SolanaChain setup
    let mchain = SolanaChain::new(test_validator.get_rpc_client(), payer, None);
    // Initialize an empty wallet
    let mut wallet = init_wallet()?;
    // Capture our programs log statements
    // ***************** UNCOMMENT NEXT LINE TO SEE LOGS
    // solana_logger::setup_with_default("solana_runtime::message=debug");

    // Incept keys
    let result = build_and_run_inception(&mchain, &mut wallet, 2i8, 1u64);
    if result.is_err() {
        println!("Failed inception");
    } else {
        let (_signature, prefix, _) = result?;
        let mut barren_ks = PastaKeySet::new_empty();
        sleep(Duration::from_secs(5));

        let result = wallet.rotate_did(prefix.clone(), &mut barren_ks, None, None, Some(&mchain));
        assert!(result.is_ok());
    }
    fs::remove_dir_all(wallet.full_path().parent().unwrap())?;
    Ok(())
}

#[test]
fn test_pasta_decommission_pass() -> SolDidResult<()> {
    // Get the test validator running
    let (test_validator, payer, _program_pk) = clean_ledger_setup_validator()?;
    // Get the SolanaChain setup
    let mchain = SolanaChain::new(test_validator.get_rpc_client(), payer, None);
    // Initialize an empty wallet
    let mut wallet = init_wallet()?;
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
        let result = wallet.decommission_did(prefix.clone(), &mut barren_ks, Some(&mchain));
        if result.is_err() {
            println!("Failed decommision");
        }
    }
    fs::remove_dir_all(wallet.full_path().parent().unwrap())?;
    Ok(())
}
// }

mod errors;
pub use errors::AppResult;
fn main() -> AppResult<()> {
    println!("Hello, world!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use keri::{
        derivation::{basic::Basic, self_addressing::SelfAddressing},
        event::{
            event_data::InceptionEvent,
            sections::{key_config::nxt_commitment, threshold::SignatureThreshold, KeyConfig},
            EventMessage, SerializationFormats,
        },
        keys::PublicKey,
        prefix::{BasicPrefix, Prefix},
    };
    use solana_client::rpc_client::RpcClient;
    use solana_keri::instruction::ProgramInstruction;
    use solana_rpc::rpc::JsonRpcConfig;
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        message::Message,
        pubkey::Pubkey,
        signature::{Keypair, Signature},
        signer::Signer,
        transaction::Transaction,
    };
    use solana_test_validator::{TestValidator, TestValidatorGenesis};
    use solana_transaction_status::UiTransactionEncoding;
    use std::{collections::BTreeMap, path::PathBuf, str::FromStr, thread::sleep, time::Duration};

    /// Location/Name of ProgramTestGenesis ledger
    const LEDGER_PATH: &str = "./.ledger";
    /// Path to BPF program (*.so)
    const PROG_PATH: &str = "../target/deploy/";
    /// Program name from program Cargo.toml
    /// FILL IN WITH YOUR PROGRAM
    const PROG_NAME: &str = "solana_keri";

    /// Setup the test validator with predefined properties
    pub fn setup_validator() -> AppResult<(TestValidator, Keypair, Pubkey)> {
        let program_id = Pubkey::new_unique();
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
            .add_program(PROG_NAME, program_id)
            // Start the test validator
            .rpc_config(JsonRpcConfig {
                enable_rpc_transaction_history: true,
                enable_cpi_and_log_storage: true,
                // faucet_addr,
                ..JsonRpcConfig::default_for_test()
            })
            .start();
        Ok((test_validator, kp, program_id))
    }

    /// Convenience function to remove existing ledger before TestValidatorGenesis setup
    /// maps to `solana-test-validator ... --reset`
    pub fn clean_ledger_setup_validator() -> AppResult<(TestValidator, Keypair, Pubkey)> {
        if PathBuf::from_str(LEDGER_PATH).unwrap().exists() {
            std::fs::remove_dir_all(LEDGER_PATH).unwrap();
        }
        setup_validator()
    }

    /// Generic function that produces Solana Keypairs and derived KERI BasicPrefixs
    fn get_keys_and_prefix(key_count: usize) -> (Vec<Keypair>, Vec<BasicPrefix>) {
        let mut sol_keys = Vec::<Keypair>::new();
        let mut keri_keys = Vec::<BasicPrefix>::new();

        for _ in 0..key_count {
            let sol_key = Keypair::new();
            let keri_bp = BasicPrefix::new(
                Basic::Ed25519,
                PublicKey::new(sol_key.pubkey().to_bytes().to_vec()),
            );
            sol_keys.push(sol_key);
            keri_keys.push(keri_bp);
        }
        (sol_keys, keri_keys)
    }
    /// Submits a transaction with programs instruction
    fn submit_transaction(
        rpc_client: &RpcClient,
        wallet_signer: &dyn Signer,
        instructions: Vec<Instruction>,
    ) -> AppResult<Signature> {
        let mut transaction =
            Transaction::new_unsigned(Message::new(&instructions, Some(&wallet_signer.pubkey())));
        let recent_blockhash = rpc_client.get_latest_blockhash().unwrap();
        transaction
            .try_sign(&vec![wallet_signer], recent_blockhash)
            .unwrap();
        Ok(rpc_client
            .send_and_confirm_transaction(&transaction)
            .unwrap())
    }
    pub struct InceptionData {
        pub event_message: EventMessage,
        pub key_set_and_prefix: (Vec<Keypair>, Vec<BasicPrefix>),
        pub key_set_and_prefix_next: (Vec<Keypair>, Vec<BasicPrefix>),
    }

    fn create_inception_event(key_count: usize, threshold: u64) -> AppResult<InceptionData> {
        let first_set = get_keys_and_prefix(key_count);
        let next_set = get_keys_and_prefix(key_count);
        let (_, keri_prefix) = &first_set;
        let (_, keri_prefix_next) = &next_set;

        let next_key_hash = nxt_commitment(
            &SignatureThreshold::Simple(threshold),
            &keri_prefix_next,
            &SelfAddressing::Blake3_256,
        );
        let key_config = KeyConfig::new(
            keri_prefix.to_vec(),
            Some(next_key_hash),
            Some(SignatureThreshold::Simple(threshold)),
        );
        Ok(InceptionData {
            event_message: InceptionEvent::new(key_config, None, None)
                .incept_self_addressing(SelfAddressing::Blake3_256, SerializationFormats::JSON)?,
            key_set_and_prefix: first_set,
            key_set_and_prefix_next: next_set,
        })
    }

    #[test]
    fn test_inception_pass() -> AppResult<()> {
        let inception_data = create_inception_event(2, 1)?;
        let prefix = inception_data.event_message.event.prefix.clone();
        assert_eq!(prefix.to_str().len(), 44);
        let sol_keri_did = ["did", "sol", "keri", &prefix.to_str()].join(":");
        let keri_vdr = "did:keri:local_db".to_string();
        let mut keri_ref = BTreeMap::<String, String>::new();
        keri_ref.insert("i".to_string(), sol_keri_did);
        keri_ref.insert("ri".to_string(), keri_vdr);
        println!("Tx doc {:?}", keri_ref);
        Ok(())
    }

    #[test]
    fn test_program_inception_pass() -> AppResult<()> {
        let inception_data = create_inception_event(2, 1)?;
        let prefix = inception_data.event_message.event.prefix.clone();
        assert_eq!(prefix.to_str().len(), 44);
        let sol_keri_did = ["did", "sol", "keri", &prefix.to_str()].join(":");
        let keri_vdr = "did:keri:local_db".to_string();
        let mut keri_ref = BTreeMap::<String, String>::new();
        keri_ref.insert("i".to_string(), sol_keri_did);
        keri_ref.insert("ri".to_string(), keri_vdr);
        // Spawn test validator node
        let (test_validator, payer, program_pk) = clean_ledger_setup_validator()?;
        // Get the RpcClient
        let connection = test_validator.get_rpc_client();
        // Capture our programs log statements
        solana_logger::setup_with_default("solana_runtime::message=debug");
        // This example doesn't require sending any accounts to program
        let accounts = &[AccountMeta::new(payer.pubkey(), true)];
        // Build instruction array and submit transaction
        let txn = submit_transaction(
            &connection,
            &payer,
            // Add two (2) instructions to transaction
            // Replace with instructions that make sense for your program
            [Instruction::new_with_borsh(
                program_pk,
                &ProgramInstruction::InceptionEvent(keri_ref),
                accounts.to_vec(),
            )]
            .to_vec(),
        );
        assert!(txn.is_ok());
        println!("Tx count = {:?}", connection.get_transaction_count());
        let signature = txn.unwrap();
        sleep(Duration::from_millis(40000));
        let tx_post = connection
            .get_transaction(&signature, UiTransactionEncoding::Json)
            .unwrap();
        println!("{:?}", tx_post);

        Ok(())
    }
}

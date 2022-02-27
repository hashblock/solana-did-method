#[cfg(test)]
mod tests {

    use borsh::BorshSerialize;
    use cli::{errors::SolKeriResult, utils::instruction_from_transaction};
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
    use solana_did_method::{id, instruction::SDMInstruction};
    use solana_rpc::rpc::JsonRpcConfig;
    use solana_sdk::{
        ed25519_instruction,
        instruction::{AccountMeta, Instruction},
        message::Message,
        pubkey::Pubkey,
        signature::{Keypair, Signature},
        signer::Signer,
        transaction::Transaction,
    };
    use solana_test_validator::{TestValidator, TestValidatorGenesis};
    use std::{collections::BTreeMap, path::PathBuf, str::FromStr, thread::sleep, time::Duration};

    /// Location/Name of ProgramTestGenesis ledger
    const LEDGER_PATH: &str = "./.ledger";
    /// Path to BPF program (*.so)
    const PROG_PATH: &str = "../target/deploy/";
    /// Program name from program/Cargo.toml
    const PROG_NAME: &str = "solana_did_method";

    /// Setup the test validator with predefined properties
    pub fn setup_validator() -> SolKeriResult<(TestValidator, Keypair, Pubkey)> {
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
                // faucet_addr,
                ..JsonRpcConfig::default_for_test()
            })
            .start();
        Ok((test_validator, kp, id()))
    }

    /// Convenience function to remove existing ledger before TestValidatorGenesis setup
    /// maps to `solana-test-validator ... --reset`
    pub fn clean_ledger_setup_validator() -> SolKeriResult<(TestValidator, Keypair, Pubkey)> {
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
        wallet_payer: &dyn Signer,
        instructions: Vec<Instruction>,
    ) -> SolKeriResult<Signature> {
        let mut transaction =
            Transaction::new_unsigned(Message::new(&instructions, Some(&wallet_payer.pubkey())));
        let recent_blockhash = rpc_client.get_latest_blockhash().unwrap();
        transaction
            .try_sign(&vec![wallet_signer, wallet_payer], recent_blockhash)
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

    fn create_inception_event(key_count: usize, threshold: u64) -> SolKeriResult<InceptionData> {
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
    #[inline]
    fn sign_event(event: &EventMessage, signer: &dyn Signer) -> SolKeriResult<Signature> {
        Ok(signer.sign_message(&event.serialize()?))
    }

    #[test]
    fn test_inception_pass() -> SolKeriResult<()> {
        let inception_data = create_inception_event(2, 1)?;
        // println!("{:?}\n\n", inception_data.event_message);
        let sol_keyp = &inception_data.key_set_and_prefix.0[0];
        let prefix = inception_data.event_message.event.prefix.clone();
        let icp_signature = sign_event(&inception_data.event_message, sol_keyp)?;
        let icp_serialized = inception_data.event_message.serialize()?;
        println!("Sig = {:?}", icp_signature);
        println!(
            "Ver {}",
            icp_signature.verify(&sol_keyp.pubkey().to_bytes(), &icp_serialized)
        );
        assert_eq!(prefix.to_str().len(), 44);
        let sol_keri_did = ["did", "sol", "keri", &prefix.to_str()].join(":");
        let keri_vdr = "did:keri:local_db".to_string();
        let mut keri_ref = BTreeMap::<String, String>::new();
        keri_ref.insert("i".to_string(), sol_keri_did);
        keri_ref.insert("ri".to_string(), keri_vdr);
        println!("Tx doc {:?}", keri_ref);
        println!("Msg = {:?}", bs58::encode(icp_serialized).into_string());
        println!("Signature = {:?}", icp_signature);
        println!("Public Key signer {:?}", sol_keyp.pubkey());

        Ok(())
    }

    #[test]
    fn test_program_inception_pass() -> SolKeriResult<()> {
        print!("Generating inception/DID... ");
        let inception_data = create_inception_event(2, 1)?;
        let prefix = inception_data.event_message.event.prefix.clone();
        assert_eq!(prefix.to_str().len(), 44);
        println!("{:?}", prefix.to_str());

        println!("Creating KERI reference doc");
        let sol_keri_did = ["did", "sol", "keri", &prefix.to_str()].join(":");
        let keri_vdr = "did:keri:local_db".to_string();
        let mut keri_ref = BTreeMap::<String, String>::new();
        keri_ref.insert("i".to_string(), sol_keri_did);
        keri_ref.insert("ri".to_string(), keri_vdr);
        // Spawn test validator node
        println!("Starting local validator node");
        let (test_validator, payer, program_pk) = clean_ledger_setup_validator()?;
        // Setup the signature verification instruction usingthe serialized key event
        let sol_keyp = &inception_data.key_set_and_prefix.0[0];
        let tx_kp = Keypair::new();
        keri_ref.insert("owner".to_string(), tx_kp.pubkey().to_string());
        let privkey = ed25519_dalek::Keypair::from_bytes(&sol_keyp.to_bytes()).unwrap();
        let ix = ed25519_instruction::new_ed25519_instruction(&privkey, &keri_ref.try_to_vec()?);

        // Get the RpcClient
        let connection = test_validator.get_rpc_client();

        // Capture our programs log statements
        // ***************** UNCOMMENT NEXT LINE TO SEE LOGS
        // solana_logger::setup_with_default("solana_runtime::message=debug");

        println!("Submitting Solana-Keri Inception Instruction");

        let accounts = &[
            AccountMeta::new_readonly(tx_kp.pubkey(), true),
            AccountMeta::new_readonly(payer.pubkey(), true),
        ];
        // let accounts = &[AccountMeta::new_readonly(payer.pubkey(), true)];
        // Build instruction array and submit transaction
        let txn = submit_transaction(
            &connection,
            &tx_kp, //payer,
            &payer,
            [
                ix,
                Instruction::new_with_borsh(
                    program_pk,
                    &SDMInstruction::InceptionEvent(keri_ref),
                    accounts.to_vec(),
                ),
            ]
            .to_vec(),
        );
        assert!(txn.is_ok());
        let signature = txn.unwrap();
        println!("Success... tx signature = {:?}", signature);
        println!("Delay 20s for block completion. Should use websocket monitoring");
        sleep(Duration::from_secs(20));
        println!("Fetching transaction for signature {:?}", signature);
        println!(
            "{:?}",
            instruction_from_transaction(&connection, &signature)
        );
        Ok(())
    }

    #[test]
    fn slicing() {
        let accounts = vec![Pubkey::new_unique()];
        for pk in &accounts[1..] {
            println!("{:?}", pk)
        }
    }
}

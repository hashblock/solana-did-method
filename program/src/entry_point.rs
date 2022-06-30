//! @brief Program entry point

// Solana standard program crates
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

// Set by cargo-solana
#[allow(dead_code)]
const NAME: &str = "solana_did_method";

entrypoint!(entry_point);
pub fn entry_point(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Normal processing
    Ok(crate::process::process(
        program_id,
        accounts,
        instruction_data,
    )?)
}

#[cfg(feature = "test-bpf")]
#[cfg(test)]

mod test {

    use crate::{
        id,
        instruction::{InceptionDID, InitializeDidAccount, SDMInstruction, SMDKeyType},
        state::{SDMDid, SDMDidState},
    };

    use super::*;
    use assert_matches::*;

    use solana_program::{
        hash::Hash,
        instruction::{AccountMeta, Instruction},
        native_token::LAMPORTS_PER_SOL,
        pubkey::{Pubkey, PUBKEY_BYTES},
    };
    use solana_program_test::{
        processor,
        tokio::{self},
        BanksClient, ProgramTest,
    };
    use solana_sdk::{
        account::Account, signature::Keypair, signer::Signer, transaction::Transaction,
    };
    use std::str::FromStr;

    /// Sets up the Program test and initializes 'n' program_accounts
    async fn setup(
        program_id: &Pubkey,
        program_accounts: &[Pubkey],
        account_sizes: &[usize],
    ) -> (BanksClient, Keypair, Hash) {
        let mut program_test = ProgramTest::new(NAME, *program_id, processor!(entry_point));
        // Add accounts for testing
        for i in 0..program_accounts.len() {
            program_test.add_account(
                program_accounts[i],
                Account {
                    lamports: 5 * LAMPORTS_PER_SOL,
                    data: vec![0_u8; account_sizes[i]],
                    owner: *program_id,
                    ..Account::default()
                },
            );
        }
        program_test.start().await
    }
    fn get_datasize(my_did: &InceptionDID) -> usize {
        0usize
            .saturating_add(std::mem::size_of::<bool>()) // Initialized
            .saturating_add(std::mem::size_of::<u16>()) // Version
            .saturating_add(std::mem::size_of::<SDMDidState>()) // State
            .saturating_add(PUBKEY_BYTES) // DID Prefix and PDA seed
            .saturating_add(std::mem::size_of::<u8>()) // bump for PDA
            .saturating_add(std::mem::size_of::<u32>())
            .saturating_add(PUBKEY_BYTES * my_did.keys.len())
    }
    #[tokio::test]

    async fn test_inception_progaccount_pass() {
        // Accounts being managed
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
        // Get program
        let program_id = id();
        // Standup runtime testing
        let dummm_faux_pda = Pubkey::new_unique();
        let (mut banks_client, payer, recent_blockhash) = setup(&program_id, &[], &[]).await;

        // println!("Payer {:?}", payer);
        // println!("Owner {:?}", dummm_faux_pda);
        let (pda_key, bump) = Pubkey::find_program_address(&[&dummm_faux_pda.to_bytes()], &id());
        println!("Pda {:?} bump {}", pda_key, bump);
        // Setup instruction payload
        let mut r = [0u8; 32];
        r.copy_from_slice(&dummm_faux_pda.to_bytes()[0..32]);
        let faux_account = InceptionDID {
            prefix: r,
            bump,
            keys,
            keytype: SMDKeyType::Ed25519,
        };
        let data_size = get_datasize(&faux_account);

        let init = InitializeDidAccount {
            rent: 5 * LAMPORTS_PER_SOL,
            storage: data_size as u64,
        };
        let macc = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new(solana_program::system_program::id(), false),
        ];
        // Build the transaction and verify execution
        let ix = [Instruction::new_with_borsh(
            program_id,
            &SDMInstruction::SDMInception(init, faux_account),
            macc,
        )];
        let mut transaction = Transaction::new_with_payer(&ix, Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        let result = banks_client
            .process_transaction_with_preflight(transaction)
            .await;

        assert_matches!(result, Ok(()));

        let account_res = banks_client
            .get_account_data_with_borsh::<SDMDid>(pda_key)
            .await;
        println!("{:?}", account_res);
        assert!(account_res.is_ok());
        println!("{:?}", account_res.unwrap());
    }
    // #[tokio::test]
    // async fn test_inception_pda_pass() {
    //     // Get program
    //     let program_id = id();
    //     let faux_prefix = Pubkey::new_unique();
    //     println!("Faux Prefix {faux_prefix:?}");
    //     let (dummm_faux_pda, bump) =
    //         Pubkey::find_program_address(&[&faux_prefix.to_bytes()], &program_id);
    //     println!("Find_pk {:?} bump {}", dummm_faux_pda, bump);
    //     let faux_pda = Pubkey::create_program_address(
    //         &[b"did:solana:", &[bump], &faux_prefix.to_bytes()],
    //         &program_id,
    //     )
    //     .unwrap();
    //     // let faux_pda = Pubkey::create_with_seed(&faux_prefix, "did:solana:", &program_id).unwrap();
    //     println!("Faux_pk {:?}", faux_pda);
    //     // let dummm_faux_pda = Pubkey::new_unique();
    //     // Fake prefix from KERI
    //     // Accounts being managed
    //     let dummy_pk1 = Pubkey::from_str("FDMHqNqN82QSjEaEuqybmpXsjtX98YuTsX6YCdT99to").unwrap();
    //     let dummy_pk2 = Pubkey::from_str("HDMHqNqN82QSjEaEuqybmpXsjtX98YuTsX6YCdT99to").unwrap();

    //     let mut keys = Vec::<Pubkey>::new();
    //     for i in 0..2 {
    //         if i == 0 {
    //             keys.push(dummy_pk1)
    //         } else {
    //             keys.push(dummy_pk2)
    //         }
    //     }
    //     // Setup instruction payload
    //     let faux_account = InceptionDID {
    //         prefix: [0u8; 32],
    //         keys,
    //     };
    //     let data_size = get_datasize(&faux_account);
    //     // Standup runtime testing
    //     let (mut banks_client, payer, recent_blockhash) =
    //         setup(&program_id, &[faux_pda], &[data_size]).await;
    //     let macc = vec![
    //         AccountMeta::new(payer.pubkey(), true),
    //         AccountMeta::new(faux_pda, false),
    //     ];
    //     // Build the transaction and verify execution
    //     let ix = [Instruction::new_with_borsh(
    //         program_id,
    //         &SDMInstruction::SDMInception(faux_account),
    //         macc,
    //     )];
    //     let mut transaction = Transaction::new_with_payer(&ix, Some(&payer.pubkey()));
    //     transaction.sign(&[&payer], recent_blockhash);
    //     let result = banks_client
    //         .process_transaction_with_preflight(transaction)
    //         .await;

    //     assert_matches!(result, Ok(()));

    //     let account_res = banks_client
    //         .get_account_data_with_borsh::<SDMDid>(faux_pda)
    //         .await;
    //     assert!(account_res.is_ok());
    //     println!("{:?}", account_res.unwrap());
    // }

    // #[tokio::test]
    // async fn test_double_inception_fail() {
    //     let dummm_faux_pda = Pubkey::new_unique();
    //     // Accounts being managed
    //     let dummy_pk1 = Pubkey::from_str("FDMHqNqN82QSjEaEuqybmpXsjtX98YuTsX6YCdT99to").unwrap();
    //     let dummy_pk2 = Pubkey::from_str("HDMHqNqN82QSjEaEuqybmpXsjtX98YuTsX6YCdT99to").unwrap();

    //     let mut keys = Vec::<Pubkey>::new();
    //     for i in 0..2 {
    //         if i == 0 {
    //             keys.push(dummy_pk1)
    //         } else {
    //             keys.push(dummy_pk2)
    //         }
    //     }
    //     // Setup instruction payload
    //     let faux_account = InceptionDID {
    //         prefix: [0u8; 32],
    //         keys,
    //     };
    //     let data_size = get_datasize(&faux_account);
    //     // Get program
    //     let program_id = id();
    //     // Standup runtime testing
    //     let (mut banks_client, payer, recent_blockhash) =
    //         setup(&program_id, &[dummm_faux_pda], &[data_size]).await;
    //     let macc = vec![
    //         AccountMeta::new(payer.pubkey(), true),
    //         AccountMeta::new(dummm_faux_pda, false),
    //     ];
    //     let eacc = macc.clone();
    //     // Build the transaction and verify execution
    //     let ix = [
    //         Instruction::new_with_borsh(
    //             program_id,
    //             &SDMInstruction::SDMInception(faux_account.clone()),
    //             macc,
    //         ),
    //         Instruction::new_with_borsh(
    //             program_id,
    //             &SDMInstruction::SDMInception(faux_account),
    //             eacc,
    //         ),
    //     ];
    //     let mut transaction = Transaction::new_with_payer(&ix, Some(&payer.pubkey()));
    //     transaction.sign(&[&payer], recent_blockhash);
    //     let result = banks_client
    //         .process_transaction_with_preflight(transaction)
    //         .await;

    //     assert!(result.is_err());
    // }

    // #[tokio::test]
    // async fn test_inception_didmember_key_fail() {
    //     // Get program key
    //     let program_id = id();

    //     let dummm_faux_pda = Pubkey::new_unique();
    //     // Accounts being managed
    //     let dummy_pk1 = Pubkey::from_str("FDMHqNqN82QSjEaEuqybmpXsjtX98YuTsX6YCdT99to").unwrap();
    //     let dummy_bad = Pubkey::create_program_address(&[b"foobar", &[1]], &program_id).unwrap();

    //     let mut keys = Vec::<Pubkey>::new();
    //     for i in 0..2 {
    //         if i == 0 {
    //             keys.push(dummy_pk1)
    //         } else {
    //             keys.push(dummy_bad)
    //         }
    //     }
    //     // Setup instruction payload
    //     let faux_account = InceptionDID {
    //         prefix: [0u8; 32],
    //         keys,
    //     };
    //     // Get data size
    //     let data_size = get_datasize(&faux_account);
    //     // Standup runtime testing
    //     let (mut banks_client, payer, recent_blockhash) =
    //         setup(&program_id, &[dummm_faux_pda], &[data_size]).await;
    //     let macc = vec![
    //         AccountMeta::new(payer.pubkey(), true),
    //         AccountMeta::new(dummm_faux_pda, false),
    //     ];
    //     // Build the transaction and verify execution
    //     let ix = [Instruction::new_with_borsh(
    //         program_id,
    //         &SDMInstruction::SDMInception(faux_account),
    //         macc,
    //     )];
    //     let mut transaction = Transaction::new_with_payer(&ix, Some(&payer.pubkey()));
    //     transaction.sign(&[&payer], recent_blockhash);
    //     let result = banks_client
    //         .process_transaction_with_preflight(transaction)
    //         .await;
    //     assert!(result.is_err());
    // }
    // #[tokio::test]
    // async fn test_inception_data_account_fail() {
    //     // Get program key
    //     let program_id = id();

    //     let dummm_faux_pda = Pubkey::new_unique();
    //     // Accounts being managed
    //     let dummy_pk1 = Pubkey::from_str("FDMHqNqN82QSjEaEuqybmpXsjtX98YuTsX6YCdT99to").unwrap();
    //     let dummy_bad = Pubkey::create_program_address(&[b"foobar", &[1]], &program_id).unwrap();

    //     let mut keys = Vec::<Pubkey>::new();
    //     for i in 0..2 {
    //         if i == 0 {
    //             keys.push(dummy_pk1)
    //         } else {
    //             keys.push(dummy_bad)
    //         }
    //     }
    //     // Setup instruction payload
    //     let faux_account = InceptionDID {
    //         prefix: [0u8; 32],
    //         keys,
    //     };
    //     // Standup runtime testing
    //     let (mut banks_client, payer, recent_blockhash) = setup(&program_id, &[], &[]).await;
    //     let macc = vec![
    //         AccountMeta::new(payer.pubkey(), true),
    //         AccountMeta::new(dummm_faux_pda, false),
    //     ];
    //     // Build the transaction and verify execution
    //     let ix = [Instruction::new_with_borsh(
    //         program_id,
    //         &SDMInstruction::SDMInception(faux_account),
    //         macc,
    //     )];
    //     let mut transaction = Transaction::new_with_payer(&ix, Some(&payer.pubkey()));
    //     transaction.sign(&[&payer], recent_blockhash);
    //     let result = banks_client
    //         .process_transaction_with_preflight(transaction)
    //         .await;
    //     assert!(result.is_err());
    // }
}

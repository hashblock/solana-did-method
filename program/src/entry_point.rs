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

#[cfg(test)]
mod test {

    use crate::{
        id,
        instruction::{InceptionDID, SDMInstruction},
        state::{SDMDid, SDMDidState},
    };

    use super::*;
    use assert_matches::*;

    use solana_program::{
        hash::Hash,
        instruction::{AccountMeta, Instruction},
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
                    lamports: 5,
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
            .saturating_add(PUBKEY_BYTES) // Prefix pubkey
            .saturating_add(std::mem::size_of::<u32>())
            .saturating_add(PUBKEY_BYTES * my_did.keys.len())
    }
    #[tokio::test]
    async fn test_inception_pass() {
        let dummm_faux_pda = Pubkey::new_unique();
        // Fake prefix from KERI
        let dummy_pk = Pubkey::from_str("SDMHqNqN82QSjEaEuqybmpXsjtX98YuTsX6YCdT99to").unwrap();
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
        // Setup instruction payload
        let faux_account = InceptionDID {
            prefix: dummy_pk,
            keys,
        };
        let data_size = get_datasize(&faux_account);
        // Get program
        let program_id = id();
        // Standup runtime testing
        let (mut banks_client, payer, recent_blockhash) =
            setup(&program_id, &[dummm_faux_pda], &[data_size]).await;
        let macc = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(dummm_faux_pda, false),
        ];
        // Build the transaction and verify execution
        let ix = [Instruction::new_with_borsh(
            program_id,
            &SDMInstruction::SDMInception(faux_account),
            macc,
        )];
        let mut transaction = Transaction::new_with_payer(&ix, Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        let result = banks_client
            .process_transaction_with_preflight(transaction)
            .await;

        assert_matches!(result, Ok(()));

        let account_res = banks_client
            .get_account_data_with_borsh::<SDMDid>(dummm_faux_pda)
            .await;
        assert!(account_res.is_ok());
        println!("{:?}", account_res.unwrap());
    }

    #[tokio::test]
    async fn test_double_inception_fail() {
        let dummm_faux_pda = Pubkey::new_unique();
        // Fake prefix from KERI
        let dummy_pk = Pubkey::from_str("SDMHqNqN82QSjEaEuqybmpXsjtX98YuTsX6YCdT99to").unwrap();
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
        // Setup instruction payload
        let faux_account = InceptionDID {
            prefix: dummy_pk,
            keys,
        };
        let data_size = get_datasize(&faux_account);
        // Get program
        let program_id = id();
        // Standup runtime testing
        let (mut banks_client, payer, recent_blockhash) =
            setup(&program_id, &[dummm_faux_pda], &[data_size]).await;
        let macc = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(dummm_faux_pda, false),
        ];
        let eacc = macc.clone();
        // Build the transaction and verify execution
        let ix = [
            Instruction::new_with_borsh(
                program_id,
                &SDMInstruction::SDMInception(faux_account.clone()),
                macc,
            ),
            Instruction::new_with_borsh(
                program_id,
                &SDMInstruction::SDMInception(faux_account),
                eacc,
            ),
        ];
        let mut transaction = Transaction::new_with_payer(&ix, Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        let result = banks_client
            .process_transaction_with_preflight(transaction)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_inception_didmember_key_fail() {
        // Get program key
        let program_id = id();

        let dummm_faux_pda = Pubkey::new_unique();
        // Fake prefix from KERI
        let dummy_pk = Pubkey::from_str("SDMHqNqN82QSjEaEuqybmpXsjtX98YuTsX6YCdT99to").unwrap();
        // Accounts being managed
        let dummy_pk1 = Pubkey::from_str("FDMHqNqN82QSjEaEuqybmpXsjtX98YuTsX6YCdT99to").unwrap();
        let dummy_bad = Pubkey::create_program_address(&[b"foobar", &[1]], &program_id).unwrap();

        let mut keys = Vec::<Pubkey>::new();
        for i in 0..2 {
            if i == 0 {
                keys.push(dummy_pk1)
            } else {
                keys.push(dummy_bad)
            }
        }
        // Setup instruction payload
        let faux_account = InceptionDID {
            prefix: dummy_pk,
            keys,
        };
        // Get data size
        let data_size = get_datasize(&faux_account);
        // Standup runtime testing
        let (mut banks_client, payer, recent_blockhash) =
            setup(&program_id, &[dummm_faux_pda], &[data_size]).await;
        let macc = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(dummm_faux_pda, false),
        ];
        // Build the transaction and verify execution
        let ix = [Instruction::new_with_borsh(
            program_id,
            &SDMInstruction::SDMInception(faux_account),
            macc,
        )];
        let mut transaction = Transaction::new_with_payer(&ix, Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        let result = banks_client
            .process_transaction_with_preflight(transaction)
            .await;
        assert!(result.is_err());
    }
    #[tokio::test]
    async fn test_inception_data_account_fail() {
        // Get program key
        let program_id = id();

        let dummm_faux_pda = Pubkey::new_unique();
        // Fake prefix from KERI
        let dummy_pk = Pubkey::from_str("SDMHqNqN82QSjEaEuqybmpXsjtX98YuTsX6YCdT99to").unwrap();
        // Accounts being managed
        let dummy_pk1 = Pubkey::from_str("FDMHqNqN82QSjEaEuqybmpXsjtX98YuTsX6YCdT99to").unwrap();
        let dummy_bad = Pubkey::create_program_address(&[b"foobar", &[1]], &program_id).unwrap();

        let mut keys = Vec::<Pubkey>::new();
        for i in 0..2 {
            if i == 0 {
                keys.push(dummy_pk1)
            } else {
                keys.push(dummy_bad)
            }
        }
        // Setup instruction payload
        let faux_account = InceptionDID {
            prefix: dummy_pk,
            keys,
        };
        // Standup runtime testing
        let (mut banks_client, payer, recent_blockhash) = setup(&program_id, &[], &[]).await;
        let macc = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(dummm_faux_pda, false),
        ];
        // Build the transaction and verify execution
        let ix = [Instruction::new_with_borsh(
            program_id,
            &SDMInstruction::SDMInception(faux_account),
            macc,
        )];
        let mut transaction = Transaction::new_with_payer(&ix, Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        let result = banks_client
            .process_transaction_with_preflight(transaction)
            .await;
        assert!(result.is_err());
    }
    #[tokio::test]
    async fn test_version_fail() {
        let dummm_faux_pda = Pubkey::new_unique();
        // Fake prefix from KERI
        let dummy_pk = Pubkey::from_str("SDMHqNqN82QSjEaEuqybmpXsjtX98YuTsX6YCdT99to").unwrap();
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
        // Setup instruction payload
        let faux_account = InceptionDID {
            prefix: dummy_pk,
            keys,
        };
        let data_size = get_datasize(&faux_account);
        // Get program
        let program_id = id();
        // Standup runtime testing
        let (mut banks_client, payer, recent_blockhash) =
            setup(&program_id, &[dummm_faux_pda], &[data_size]).await;
        let macc = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(dummm_faux_pda, false),
        ];
        let eacc = macc.clone();
        // Build the transaction and verify execution
        let ix = [
            Instruction::new_with_borsh(
                program_id,
                &SDMInstruction::SDMInception(faux_account),
                macc,
            ),
            Instruction::new_with_borsh(program_id, &SDMInstruction::SDMInvalidVersionTest, eacc),
        ];
        let mut transaction = Transaction::new_with_payer(&ix, Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        let result = banks_client
            .process_transaction_with_preflight(transaction)
            .await;
        assert!(result.is_err());
    }
}

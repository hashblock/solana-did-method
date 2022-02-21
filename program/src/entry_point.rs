//! @brief Program entry point

// References program error and core processor
use crate::{error::CustomProgramError, process::process};
// Solana standard program crates
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg,
    program_error::PrintProgramError, pubkey::Pubkey,
};

// Set by cargo-solana
const NAME: &str = "bar";

entrypoint!(entry_point);
pub fn entry_point(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {

    // This is expensive, delete when satisfied
    msg!(
        "Program {} id: {} accounts: {} data: {:?}",
        NAME,
        program_id,
        accounts.len(),
        instruction_data
    );
    // Normal processing
    if let Err(error) = process(program_id, accounts, instruction_data) {
        // catch the error so we can print it
        error.print::<CustomProgramError>();
        return Err(error);
    }

    Ok(())
}

#[cfg(test)]
mod test {

    use crate::instruction::ProgramInstruction;

    use super::*;
    use assert_matches::*;

    use solana_program::{
        hash::Hash,
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
    };
    use solana_program_test::{
        processor,
        tokio::{self},
        BanksClient, ProgramTest,
    };
    use solana_sdk::{
        account::Account, signature::Keypair, signer::Signer, transaction::Transaction,
    };
    use std::time::Duration;

    /// Sets up the Program test and initializes 'n' program_accounts
    async fn setup(
        program_id: &Pubkey,
        program_accounts: &[Pubkey],
    ) -> (BanksClient, Keypair, Hash) {
        let mut program_test = ProgramTest::new(NAME, *program_id, processor!(entry_point));
        // Add accounts for testing
        for account in program_accounts {
            program_test.add_account(
                *account,
                Account {
                    lamports: 5,
                    data: vec![0_u8; 3],
                    owner: *program_id,
                    ..Account::default()
                },
            );
        }
        program_test.start().await
    }

    #[tokio::test]
    async fn test_initialize_pass() {
        let program_id = Pubkey::new_unique();
        let account_pubkey = Pubkey::new_unique();

        // Standup runtime testing
        let (mut banks_client, payer, recent_blockhash) =
            setup(&program_id, &[account_pubkey]).await;

        // Verify account has clean slate
        let acc = banks_client
            .get_account(account_pubkey)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(acc.data[0], 0);
        assert_eq!(acc.data[1], 0);
        assert_eq!(acc.data[2], 0);

        let macc = vec![
            AccountMeta::new(account_pubkey, false),
            AccountMeta::new(payer.pubkey(), true),
        ];
        // Build the transaction and verify execution
        let ix = [Instruction::new_with_borsh(
            program_id,
            &ProgramInstruction::InitializeAccount,
            macc,
        )];
        let mut transaction = Transaction::new_with_payer(&ix, Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        assert_matches!(banks_client.process_transaction(transaction).await, Ok(()));

        // Verify initialized
        let acc = banks_client
            .get_account(account_pubkey)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(acc.data[0], 1);
        assert_eq!(acc.data[1], 1);
        assert_eq!(acc.data[2], 0);
    }

    #[tokio::test]
    async fn test_double_initialize_fail() {
        let program_id = Pubkey::new_unique();
        let account_pubkey = Pubkey::new_unique();

        // Standup runtime testing
        let (mut banks_client, payer, recent_blockhash) =
            setup(&program_id, &[account_pubkey]).await;

        // Setup accounts for program entry point
        let macc = vec![
            AccountMeta::new(account_pubkey, false),
            AccountMeta::new(payer.pubkey(), true),
        ];

        // Setup initialization instruction
        let ix = [Instruction::new_with_borsh(
            program_id,
            &ProgramInstruction::InitializeAccount,
            macc,
        )];

        // Submit initialize instruction
        let mut transaction = Transaction::new_with_payer(&ix, Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        assert_matches!(banks_client.process_transaction(transaction).await, Ok(()));

        // Wait for new blockhash
        tokio::time::sleep(Duration::from_millis(500)).await;
        let new_blockhash = banks_client.get_latest_blockhash().await.unwrap();
        assert_ne!(recent_blockhash, new_blockhash);

        // Submit second transaction which fill fail on already initialized
        let mut transaction = Transaction::new_with_payer(&ix, Some(&payer.pubkey()));
        transaction.sign(&[&payer], new_blockhash);
        let result = banks_client.process_transaction(transaction).await;
        assert!(result.is_err());
    }
    #[tokio::test]
    async fn test_setting_content_pass() {
        let program_id = Pubkey::new_unique();
        let account_pubkey = Pubkey::new_unique();

        // Standup runtime testing
        let (mut banks_client, payer, recent_blockhash) =
            setup(&program_id, &[account_pubkey]).await;

        // Build the transaction and verify execution
        let ix = [Instruction::new_with_borsh(
            program_id,
            &ProgramInstruction::InitializeAccount,
            vec![
                AccountMeta::new(account_pubkey, false),
                AccountMeta::new(payer.pubkey(), true),
            ],
        )];
        let mut transaction = Transaction::new_with_payer(&ix, Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        assert_matches!(banks_client.process_transaction(transaction).await, Ok(()));

        // Verify initialized
        let acc = banks_client
            .get_account(account_pubkey)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(acc.data[0], 1);
        assert_eq!(acc.data[1], 1);
        assert_eq!(acc.data[2], 0);

        // Build the conent setting transaction and verify execution
        let ix = [Instruction::new_with_borsh(
            program_id,
            &ProgramInstruction::SetContent(1u8),
            vec![
                AccountMeta::new(account_pubkey, false),
                AccountMeta::new(payer.pubkey(), true),
            ],
        )];
        let mut transaction = Transaction::new_with_payer(&ix, Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        assert_matches!(banks_client.process_transaction(transaction).await, Ok(()));
        // Verify initialized
        let acc = banks_client
            .get_account(account_pubkey)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(acc.data[0], 1);
        assert_eq!(acc.data[1], 1);
        assert_eq!(acc.data[2], 1);
    }

    #[tokio::test]
    async fn test_setting_content_not_initialized_fail() {
        let program_id = Pubkey::new_unique();
        let account_pubkey = Pubkey::new_unique();

        // Standup runtime testing
        let (mut banks_client, payer, recent_blockhash) =
            setup(&program_id, &[account_pubkey]).await;

        // Build the conent setting transaction and verify execution
        let ix = [Instruction::new_with_borsh(
            program_id,
            &ProgramInstruction::SetContent(1u8),
            vec![
                AccountMeta::new(account_pubkey, false),
                AccountMeta::new(payer.pubkey(), true),
            ],
        )];
        let mut transaction = Transaction::new_with_payer(&ix, Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        let result = banks_client.process_transaction(transaction).await;
        assert!(result.is_err());
    }
}

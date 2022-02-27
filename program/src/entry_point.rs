//! @brief Program entry point

// References program error and core processor
use crate::{error::SDMProgramError, process::process};
// Solana standard program crates
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult,
    program_error::PrintProgramError, pubkey::Pubkey,
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
    if let Err(error) = process(program_id, accounts, instruction_data) {
        // catch the error so we can print it
        error.print::<SDMProgramError>();
        return Err(error);
    }

    Ok(())
}

#[cfg(test)]
mod test {

    use crate::{id, instruction::SDMInstruction};

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
    use std::collections::BTreeMap;

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
    async fn test_inception_pass() {
        // let program_id = Pubkey::new_unique();
        let program_id = id();
        let signer = Keypair::new();
        // Standup runtime testing
        let (mut banks_client, payer, recent_blockhash) = setup(&program_id, &[]).await;

        let sol_keri_did = ["did", "sol", "keri", &payer.pubkey().to_string()].join(":");
        let mut keri_ref = BTreeMap::<String, String>::new();
        keri_ref.insert("i".to_string(), sol_keri_did);
        keri_ref.insert("ri".to_string(), "did:keri:local_db".to_string());
        keri_ref.insert("owner".to_string(), payer.pubkey().to_string());

        let macc = vec![
            AccountMeta::new_readonly(payer.pubkey(), false),
            AccountMeta::new_readonly(signer.pubkey(), false),
        ];
        // Build the transaction and verify execution
        let ix = [Instruction::new_with_borsh(
            program_id,
            &SDMInstruction::InceptionEvent(keri_ref),
            macc,
        )];
        let mut transaction = Transaction::new_with_payer(&ix, Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        let result = banks_client.process_transaction(transaction).await;
        assert_matches!(result, Ok(()));
    }
}

//! Program core processing module

use std::collections::BTreeMap;

use crate::{error::CustomProgramError, instruction::ProgramInstruction};

use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

/// Checks each account to confirm it is owned by our program
/// This function assumes that the program account is always the last
/// in the array
/// Change this to suite your account logic
fn check_account_ownership(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    // Accounts must be owned by the program.
    for account in accounts.iter().take(accounts.len() - 1) {
        if account.owner != program_id {
            msg!(
                "Fail: Account owner is {} and it should be {}.",
                account.owner,
                program_id
            );
            return Err(ProgramError::IncorrectProgramId);
        }
    }
    Ok(())
}

fn verify_inception(did_ref: BTreeMap<String, String>) -> ProgramResult {
    msg!("Processing DID:SOL:KERI Inception");
    if did_ref.keys().len() != 2 {
        Err(CustomProgramError::InvalidDidReference.into())
    } else if did_ref.get(&"i".to_string()).is_none() {
        Err(CustomProgramError::InvalidDidReference.into())
    } else if did_ref.get(&"ri".to_string()).is_none() {
        Err(CustomProgramError::InvalidDidReference.into())
    } else {
        msg!("Valdated DID Reference {:?}", did_ref);
        Ok(())
    }
}

/// Main processing entry point dispatches to specific
/// instruction handlers
pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Check the account for program ownership relation
    if let Err(error) = check_account_ownership(program_id, accounts) {
        return Err(error);
    }

    // Unpack the inbound data, mapping instruction to appropriate function
    match ProgramInstruction::unpack(instruction_data)? {
        ProgramInstruction::InceptionEvent(did_ref) => verify_inception(did_ref),
    }
}

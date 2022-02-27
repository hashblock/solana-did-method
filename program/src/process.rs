//! Program core processing module

use std::{collections::BTreeMap, str::FromStr};

use crate::{error::SDMProgramError, instruction::SDMInstruction};

use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

/// Checks each account to confirm it is owned by our program
/// This function assumes that the program account is always the last
/// in the array
/// Change this to suite your account logic
fn check_account_ownership(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    // First account is wallet so, any subsequent in this example must be owned by the program.
    for account in &accounts[2..] {
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

fn verify_inception(signer: &AccountInfo, did_ref: BTreeMap<String, String>) -> ProgramResult {
    msg!("Processing DID:SOL:KERI Inception");
    if did_ref.keys().len() != 3 {
        Err(SDMProgramError::InvalidDidReference.into())
    } else if !did_ref.contains_key(&"i".to_string()) {
        Err(SDMProgramError::InvalidDidReference.into())
    } else if !did_ref.contains_key(&"ri".to_string()) {
        Err(SDMProgramError::InvalidDidReference.into())
    } else if !did_ref.contains_key(&"owner".to_string()) {
        Err(SDMProgramError::InvalidDidReference.into())
    } else {
        if signer
            .key
            .eq(&Pubkey::from_str(did_ref.get(&"owner".to_string()).unwrap()).unwrap())
        {
            Ok(())
        } else {
            Err(SDMProgramError::OwnerNotSignerError.into())
        }
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
    match SDMInstruction::unpack(instruction_data)? {
        SDMInstruction::InceptionEvent(did_ref) => verify_inception(&accounts[0], did_ref),
    }
}

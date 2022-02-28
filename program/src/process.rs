//! Program core processing module

use crate::{
    instruction::{InceptionDID, SDMInstruction},
    state::{SDMDid, SDMProgramError},
};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

/// Checks each account to confirm it is owned by our program
/// This function assumes that the program account is always the last
/// in the array
/// Change this to suite your account logic
fn check_account_ownership(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    // First account is wallet so, any subsequent in this example must be owned by the program.
    for account in &accounts[1..] {
        if account.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }
    }
    msg!("Accounts validated");
    Ok(())
}

fn sdm_inception(accounts: &[AccountInfo], did: InceptionDID) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    // Skip signer
    next_account_info(account_iter)?;
    // Get the did account
    let pda = next_account_info(account_iter)?;
    let mut my_data = pda.try_borrow_mut_data()?;
    let mut did_doc = SDMDid::unpack_unitialized(&my_data, did)?;
    for key in &did_doc.did_doc.keys {
        if !key.is_on_curve() {
            msg!("Invalid DID key {:?}", key);
            return Err(SDMProgramError::DidInvalidKey.into());
        }
    }
    did_doc.pack(*my_data)?;
    Ok(())
}

fn sdm_test_version_hit(accounts: &[AccountInfo]) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    // Skip signer
    next_account_info(account_iter)?;
    // Get the did account
    let pda = next_account_info(account_iter)?;
    let mut my_data = pda.try_borrow_mut_data()?;
    let mut did_doc = SDMDid::unpack(&my_data)?;
    did_doc.flip_version();
    did_doc.pack(*my_data)?;
    SDMDid::unpack(&my_data)?;
    Ok(())
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
        SDMInstruction::SDMInception(d) => sdm_inception(accounts, d),
        SDMInstruction::SDMInvalidVersionTest => sdm_test_version_hit(accounts),
    }
}

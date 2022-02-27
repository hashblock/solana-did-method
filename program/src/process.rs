//! Program core processing module

use crate::{
    instruction::{InceptionDID, SDMInstruction},
    state::SDMDid,
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
    let pda = next_account_info(account_iter)?;
    msg!("Inception for {:?}", pda.key);
    let mut my_data = pda.try_borrow_mut_data()?;
    let mut did_doc = SDMDid::unpack_unitialized(&my_data, did)?;
    did_doc.set_initialized();
    did_doc.pack(*my_data)?;
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
    }
}

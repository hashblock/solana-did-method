//! Program core processing module

use crate::{
    error::CustomProgramError, instruction::ProgramInstruction, state::ProgramAccountState,
};

use solana_program::{
    account_info::next_account_info,
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
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

/// Initialize the programs account, which is the first in accounts
fn initialize_account(accounts: &[AccountInfo]) -> ProgramResult {
    msg!("Initialize account");
    let account_info_iter = &mut accounts.iter();
    let program_account = next_account_info(account_info_iter)?;
    let mut account_data = program_account.data.borrow_mut();
    // Just using unpack will check to see if initialized and will
    // fail if not so here we use the unpack_unchecked to avoid the error
    let mut account_state = ProgramAccountState::unpack_unchecked(&account_data)?;
    // Where this is a logic error in trying to initialize the same
    // account more than once
    if account_state.is_initialized() {
        Err(CustomProgramError::AccountAlreadyInitializedError.into())
    } else {
        account_state.set_initialized();
        ProgramAccountState::pack(account_state, &mut account_data).unwrap();
        Ok(())
    }
}

// Your program functions go here and invoked vis-a-vis the match
// resoltion in the `process` function below for example:

/// Set content to new value
fn set_content(accounts: &[AccountInfo], new_content: u8) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let program_account = next_account_info(account_info_iter)?;
    let mut account_data = program_account.data.borrow_mut();
    // Just use unpack and it will check to see if initialized and fail if not
    let mut account_state = ProgramAccountState::unpack(&account_data)?;
    // Set the new content
    let previous_content = account_state.set_content(new_content);
    msg!(
        "Previous content {} set to {}",
        previous_content,
        new_content
    );
    ProgramAccountState::pack(account_state, &mut account_data).unwrap();
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
    match ProgramInstruction::unpack(instruction_data)? {
        ProgramInstruction::InitializeAccount => initialize_account(accounts),
        ProgramInstruction::SetContent(new_content) => set_content(accounts, new_content),
    }
}

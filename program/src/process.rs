//! Program core processing module

use crate::{
    instruction::{
        DIDDecommission, DIDInception, DIDRotation, InitializeDidAccount, SDMInstruction,
    },
    state::SDMDid,
};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
};

/// Checks each account to confirm it is owned by our program
/// This function assumes that the program account is always the last
/// in the array
/// Change this to suite your account logic
#[allow(dead_code)]
fn check_account_ownership(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    // First account is wallet so, any subsequent in this example must be owned by the program.
    for account in &accounts[1..] {
        msg!("Accoumt key {:?}", account.key);
        if account.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }
    }
    Ok(())
}

/// Inception event creates and initiates a DID PDA and
/// stores the active public keys
fn sdm_inception(
    accounts: &[AccountInfo],
    program_id: &Pubkey,
    init: InitializeDidAccount,
    did: DIDInception,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    // Signer and payer of PDA for DID
    let authority_account = next_account_info(account_iter)?;
    if !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    // Get the did proposed account
    let pda = next_account_info(account_iter)?;
    // Get the system program
    let sys_prog_id = next_account_info(account_iter)?;

    // Create the PDA for this DID
    let create_pda_ix = &system_instruction::create_account(
        authority_account.key,
        pda.key,
        init.rent,
        init.storage,
        program_id,
    );

    // Create PDA account with storage for DID
    invoke_signed(
        &create_pda_ix,
        &[authority_account.clone(), pda.clone(), sys_prog_id.clone()],
        &[&[&did.prefix, &[did.bump]]],
    )?;
    let mut my_data = pda.try_borrow_mut_data()?;
    let mut did_doc = SDMDid::unpack_unitialized(&my_data, did, authority_account.key)?;
    did_doc.pack(*my_data)?;
    Ok(())
}

/// Rotation verifies the prefix and then stores a new set of public keys
fn sdm_rotation(accounts: &[AccountInfo], _program_id: &Pubkey, did: DIDRotation) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    // Signer and payer of PDA for DID
    let authority_account = next_account_info(account_iter)?;
    if !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    // Get the did proposed account
    let pda = next_account_info(account_iter)?;
    let mut my_data = pda.try_borrow_mut_data()?;
    let mut did_doc = SDMDid::unpack(&my_data)?;
    did_doc.verify_authority(authority_account.key.clone())?;
    did_doc.verify_inbound(did.keytype, did.prefix)?;
    did_doc.rotate_with(did)?;
    did_doc.pack(*my_data)?;
    Ok(())
}

/// Decommission verifies the prefix and then stores a new set of public keys
fn sdm_decommission(
    accounts: &[AccountInfo],
    _program_id: &Pubkey,
    did: DIDDecommission,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    // Signer and payer of PDA for DID
    let authority_account = next_account_info(account_iter)?;
    if !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    // Get the did proposed account
    let pda = next_account_info(account_iter)?;
    let mut my_data = pda.try_borrow_mut_data()?;
    let mut did_doc = SDMDid::unpack(&my_data)?;
    did_doc.verify_authority(authority_account.key.clone())?;
    did_doc.verify_inbound(did.keytype, did.prefix)?;
    did_doc.decommission_with(did)?;
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
    // if let Err(error) = check_account_ownership(program_id, accounts) {
    //     return Err(error);
    // }

    // Unpack the inbound data, mapping instruction to appropriate function
    match SDMInstruction::unpack(instruction_data)? {
        SDMInstruction::SDMInception(init, did_content) => {
            sdm_inception(accounts, program_id, init, did_content)
        }
        SDMInstruction::SDMRotation(input) => sdm_rotation(accounts, program_id, input),
        SDMInstruction::SDMDecommission(input) => sdm_decommission(accounts, program_id, input),
    }
}

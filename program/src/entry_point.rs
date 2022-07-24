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

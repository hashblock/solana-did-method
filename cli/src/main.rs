//! cli for managing sol::keri dids and keys
mod clparse;
pub mod errors;

use cli::{
    errors::{SolDidError, SolDidResult},
    solana_wrap::schain_wrap::SolanaChain,
};

use crate::clparse::command_line;

#[tokio::main]
async fn main() -> SolDidResult<()> {
    // Parse command line
    let cmdline = command_line().get_matches();
    let (_command, _matches) = cmdline.subcommand().unwrap();

    // Load default wallet or use user provided command line wallet path

    // Load chain wrapper
    let _chain = SolanaChain::default();
    Ok(())
}

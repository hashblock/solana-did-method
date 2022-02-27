//! cli for managing sol::keri dids and keys
mod clparse;
mod errors;
mod utils;
pub use errors::SolKeriResult;
pub use utils::instruction_from_transaction;

fn main() -> SolKeriResult<()> {
    println!("Hello, world!");
    Ok(())
}

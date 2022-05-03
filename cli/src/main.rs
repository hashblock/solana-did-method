//! cli for managing sol::keri dids and keys
mod clparse;
mod errors;
mod incp_event;
mod pasta_keys;
mod utils;
pub use errors::SolKeriResult;
use std::env;
pub use utils::instruction_from_transaction;

fn main() -> SolKeriResult<()> {
    println!("Current directory {:?}", env::current_dir());
    Ok(())
}

#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use borsh::BorshSerialize;
    use solana_sdk::pubkey::Pubkey;

    #[derive(BorshSerialize, Debug)]
    struct FauxAccount {
        prefix: Pubkey,
        keys: Vec<Pubkey>,
    }
    #[test]
    fn test_serialization() {
        let dummy_pk = Pubkey::from_str("SDMHqNqN82QSjEaEuqybmpXsjtX98YuTsX6YCdT99to").unwrap();
        let dummy_pk1 = Pubkey::from_str("FDMHqNqN82QSjEaEuqybmpXsjtX98YuTsX6YCdT99to").unwrap();
        let dummy_pk2 = Pubkey::from_str("HDMHqNqN82QSjEaEuqybmpXsjtX98YuTsX6YCdT99to").unwrap();

        let mut keys = Vec::<Pubkey>::new();
        for i in 0..2 {
            if i == 0 {
                keys.push(dummy_pk1)
            } else {
                keys.push(dummy_pk2)
            }
        }
        let faux_account = FauxAccount {
            prefix: dummy_pk,
            keys,
        };
        let y = 33 + (2 * 32) + 1;
        let z = std::mem::size_of_val(&faux_account);
        let saccount = faux_account.try_to_vec().unwrap();
        let w = saccount.len();
        println!("size calc {y} or size mem {z} = {w}");
    }
}

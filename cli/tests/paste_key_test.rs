#[cfg(test)]
mod tests {
    use cli::pasta_keys::{PastaKeyPair, PastaPublicKey};
    use solana_sdk::pubkey::Pubkey;
    #[test]
    fn test_keys() {
        let keypair = PastaKeyPair::new();
        println!("Secret 1 {:?}", keypair);
        let pkey = keypair.public_key();
        println!("Pubkey 1 {}", pkey.to_base58_string());
        let pkey =
            PastaPublicKey::from_base58_string("GybzWZH3QJjAjATn5WozC1TThUZhCnea3pPMWc7KbP1X");
        println!("Pubkey 2 {}", pkey.to_base58_string());
        let spkey = Pubkey::new_from_array(pkey.to_bytes());
        println!("Solana PKey {:?}", spkey);
    }
}

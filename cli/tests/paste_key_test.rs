#[cfg(test)]
mod tests {
    use cli::pasta_keys::{PastaPublicKey, PastaSecretKey};
    use rand::rngs::OsRng;
    use solana_sdk::pubkey::Pubkey;
    #[test]
    fn test_keys() {
        let secret = PastaSecretKey::random(&mut OsRng);
        println!("Secret {:?}", secret);
        let pkey = PastaPublicKey::from_secret_key(&secret);
        println!("Pubkey 1 {}", pkey.to_base58_string());
        let pkey =
            PastaPublicKey::from_base58_string("GybzWZH3QJjAjATn5WozC1TThUZhCnea3pPMWc7KbP1X");
        println!("Pubkey 2 {}", pkey.to_base58_string());
        let spkey = Pubkey::new_from_array(pkey.to_bytes());
        println!("Solana PKey {:?}", spkey);
    }
}

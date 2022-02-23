fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use keri::{
        derivation::{basic::Basic, self_addressing::SelfAddressing},
        event::{
            event_data::InceptionEvent,
            sections::{key_config::nxt_commitment, threshold::SignatureThreshold, KeyConfig},
            SerializationFormats,
        },
        keys::PublicKey,
        prefix::{BasicPrefix, Prefix},
    };
    use solana_sdk::{signature::Keypair, signer::Signer};

    fn get_keys_and_prefix(key_count: usize) -> (Vec<Keypair>, Vec<BasicPrefix>) {
        let mut sol_keys = Vec::<Keypair>::new();
        let mut keri_keys = Vec::<BasicPrefix>::new();

        for _ in 0..key_count {
            let sol_key = Keypair::new();
            let keri_bp = BasicPrefix::new(
                Basic::Ed25519,
                PublicKey::new(sol_key.pubkey().to_bytes().to_vec()),
            );
            sol_keys.push(sol_key);
            keri_keys.push(keri_bp);
        }
        (sol_keys, keri_keys)
    }

    #[test]
    fn test_inception_pass() {
        let (_sol_keys, keri_prefix) = get_keys_and_prefix(2);
        let (_sol_keys_next, keri_prefix_next) = get_keys_and_prefix(2);

        let next_key_hash = nxt_commitment(
            &SignatureThreshold::Simple(2),
            &keri_prefix_next,
            &SelfAddressing::Blake3_256,
        );
        let key_config = KeyConfig::new(
            keri_prefix,
            Some(next_key_hash),
            Some(SignatureThreshold::Simple(1)),
        );
        let icp_data = InceptionEvent::new(key_config, None, None)
            .incept_self_addressing(SelfAddressing::Blake3_256, SerializationFormats::JSON)
            .unwrap();
        let prefix = icp_data.event.prefix.clone();
        assert_eq!(prefix.to_str().len(), 44);
        let sol_keri_did = ["did", "sol", "keri", &prefix.to_str()].join(":");
        let keri_vdr = "did:keri:local_db".to_string();
        let mut keri_ref = BTreeMap::<String, String>::new();
        keri_ref.insert("i".to_string(), sol_keri_did);
        keri_ref.insert("ri".to_string(), keri_vdr);
        println!("Tx doc {:?}", keri_ref);
    }
}

//! Pasta key wrapper

use hbkr_rs::{
    basic::Basic,
    key_manage::{KeySet, PrivKey, Privatekey, Publickey},
};
use hbpasta_rs::Keypair as PastaKP;

#[derive(Clone, Debug)]
pub struct PastaKeySet {
    barren: bool,
    keytype: Basic,
    current: Vec<PastaKP>,
    next: Vec<PastaKP>,
}

impl PastaKeySet {
    /// Create a KeySet for count (1-127) current next KeyPairs
    pub fn new_for(count: i8) -> Self {
        let mut current = Vec::<PastaKP>::new();
        let mut next = Vec::<PastaKP>::new();
        for _ in 0..count {
            current.push(PastaKP::new());
            next.push(PastaKP::new());
        }
        Self {
            barren: false,
            current,
            next,
            keytype: Basic::PASTA,
        }
    }

    /// Create an empty KeySet
    pub fn new_empty() -> Self {
        Self {
            barren: true,
            keytype: Basic::PASTA,
            current: Vec::<PastaKP>::new(),
            next: Vec::<PastaKP>::new(),
        }
    }
}

impl KeySet for PastaKeySet {
    /// is_barren returns true if there are no keys in the keyset
    fn is_barren(&self) -> bool {
        self.barren
    }

    fn from(&mut self, current_ks: Vec<String>, next_ks: Vec<String>) {
        self.current = current_ks
            .iter()
            .map(|s| PastaKP::from_base58_string(s).unwrap())
            .collect::<Vec<PastaKP>>();
        self.next = next_ks
            .iter()
            .map(|s| PastaKP::from_base58_string(s).unwrap())
            .collect::<Vec<PastaKP>>();
        self.barren = false
    }
    fn rotate(&mut self, new_next: Option<Vec<Privatekey>>) -> (Vec<Privatekey>, Vec<Privatekey>) {
        self.current = self.next.clone();
        self.next = match new_next {
            Some(k) => k
                .iter()
                .map(|s| PastaKP::from_base58_string(&s.as_base58_string()).unwrap())
                .collect::<Vec<PastaKP>>(),
            None => self
                .current
                .iter()
                .map(|_| PastaKP::new())
                .collect::<Vec<PastaKP>>(),
        };
        (self.current_private_keys(), self.next_private_keys())
    }
    fn current_private_keys(&self) -> Vec<Privatekey> {
        self.current
            .iter()
            .map(|x| Privatekey::new(x.private_key().to_bytes().to_vec()))
            .collect::<Vec<Privatekey>>()
    }

    fn next_private_keys(&self) -> Vec<Privatekey> {
        self.next
            .iter()
            .map(|x| Privatekey::new(x.private_key().to_bytes().to_vec()))
            .collect::<Vec<Privatekey>>()
    }

    fn current_public_keys(&self) -> Vec<Publickey> {
        self.current
            .iter()
            .map(|x| Publickey::new(x.private_key().pubkey().to_bytes().to_vec()))
            .collect::<Vec<Publickey>>()
    }

    fn next_public_keys(&self) -> Vec<Publickey> {
        self.next
            .iter()
            .map(|x| Publickey::new(x.private_key().pubkey().to_bytes().to_vec()))
            .collect::<Vec<Publickey>>()
    }

    fn key_type(&self) -> Basic {
        self.keytype
    }
}

#[cfg(test)]
mod pasta_key_tests {
    use crate::errors::SolDidResult;

    use super::*;
    #[test]
    fn test_basic_with_pasta_pass() -> SolDidResult<()> {
        //  Keys 1
        let count = 2i8;
        let kset1 = PastaKeySet::new_for(count);
        assert!(!kset1.is_barren());
        assert_eq!(kset1.key_type(), Basic::PASTA);
        assert_eq!(kset1.current_private_keys().len(), 2);
        assert_eq!(kset1.current_public_keys().len(), 2);
        assert_eq!(kset1.next_private_keys().len(), 2);
        assert_eq!(kset1.next_public_keys().len(), 2);
        Ok(())
    }
    #[test]
    fn test_empty_with_pasta_pass() -> SolDidResult<()> {
        let kset1 = PastaKeySet::new_empty();
        assert!(kset1.is_barren());
        assert_eq!(kset1.key_type(), Basic::PASTA);
        assert_eq!(kset1.current_private_keys().len(), 0);
        assert_eq!(kset1.current_public_keys().len(), 0);
        assert_eq!(kset1.next_private_keys().len(), 0);
        assert_eq!(kset1.next_public_keys().len(), 0);
        Ok(())
    }

    #[test]
    fn test_from_wallet_key_pass() -> SolDidResult<()> {
        let current = [
            "BoYfHgWQmEWndXRikjxS8HDVCRma8yScMRTLEE33RjBK".to_string(),
            "5axEqp1kauUVNF5DS17oGooiESoTf58iph1gSSJ7FBfW".to_string(),
        ]
        .map(|s| s)
        .to_vec();
        let next = [
            "FReJayG8gxwupDsYfwo6ZDfNjvWXdfgNhWganZZpWn8a".to_string(),
            "8ceJbBeXg6dDjyU2GYZRk9TKoDGHFG4CnYn174CZ3i7E".to_string(),
        ]
        .map(|s| s)
        .to_vec();
        // let kset1 = keyset_generator(&current, &next)?;
        let mut kset1 = PastaKeySet::new_empty();
        kset1.from(current, next);
        assert!(!kset1.is_barren());
        assert_eq!(kset1.key_type(), Basic::PASTA);
        assert_eq!(kset1.current_private_keys().len(), 2);
        assert_eq!(kset1.current_public_keys().len(), 2);
        assert_eq!(kset1.next_private_keys().len(), 2);
        assert_eq!(kset1.next_public_keys().len(), 2);
        Ok(())
    }
}

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
    /// Create a KeySet for count (1-255) current next KeyPairs
    pub fn new_for(count: u8) -> Self {
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

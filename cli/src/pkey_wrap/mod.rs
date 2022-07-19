//! Pasta key wrapper

use hbkr_rs::{
    basic::Basic,
    key_manage::{KeySet, PrivKey, Privatekey, Publickey},
};
use hbpasta_rs::Keypair as PastaKP;

use crate::errors::SolDidResult;

#[derive(Clone, Debug)]
pub struct PastaKeySet {
    barren: bool,
    keytype: Basic,
    current: Vec<PastaKP>,
    next: Vec<PastaKP>,
}

impl PastaKeySet {
    /// Create a default set for one (1) current and next KeyPairs for KeySet
    // pub fn new() -> Self {
    //     let mut curr_vec = Vec::<PastaKP>::new();
    //     curr_vec.push(PastaKP::new());
    //     let mut next_vec = Vec::<PastaKP>::new();
    //     next_vec.push(PastaKP::new());
    //     Self {
    //         current: curr_vec,
    //         next: next_vec,
    //     }
    // }

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

    // pub fn reconstruct(current: Vec<String>, next: Vec<String>) -> SolDidResult<Self> {
    //     let curr_kps = current
    //         .iter()
    //         .map(|in_str| PastaKP::from_base58_string(&in_str).unwrap())
    //         .collect::<Vec<PastaKP>>();
    //     let next_kps = next
    //         .iter()
    //         .map(|in_str| PastaKP::from_base58_string(&in_str).unwrap())
    //         .collect::<Vec<PastaKP>>();
    //     Ok(PastaKeySet {
    //         barren: false,
    //         keytype: Basic::PASTA,
    //         current: curr_kps,
    //         next: next_kps,
    //     })
    // }

    // pub fn with_current_keypair(in_vec: Vec<PastaKP>) -> Self {
    //     Self {
    //         current: in_vec.clone(),
    //         next: in_vec
    //             .iter()
    //             .map(|_| PastaKP::new())
    //             .collect::<Vec<PastaKP>>(),
    //     }
    // }

    // pub fn with_current_and_next_keypairs(
    //     in_current: Vec<PastaKP>,
    //     in_next: Vec<PastaKP>,
    // ) -> Self {
    //     Self {
    //         current: in_current,
    //         next: in_next,
    //     }
    // }

    // pub fn from_file(fpath: &Path) -> Self {
    //     todo!()
    // }
    // pub fn to_file(&self, fpath: &Path) {
    //     todo!()
    // }
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

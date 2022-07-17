//! Pasta key wrapper

use hbkr_rs::{
    basic::Basic,
    key_manage::{KeySet, Privatekey, Publickey},
};
use hbpasta_rs::Keypair as PastaKP;

#[derive(Clone, Debug)]
pub struct PastaKeySet {
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
            current,
            next,
            keytype: Basic::PASTA,
        }
    }

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
    fn rotate(&mut self) {
        self.current = self.next.clone();
        self.next = self
            .current
            .iter()
            .map(|_| PastaKP::new())
            .collect::<Vec<PastaKP>>()
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

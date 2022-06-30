//! Solana key wrap

use hbkr_rs::key_manage::{KeySet, Privatekey, Publickey};
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};

#[derive(Debug)]
pub struct SolanaKeySet {
    current: Vec<Keypair>,
    next: Vec<Keypair>,
}

impl SolanaKeySet {
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
        let mut current = Vec::<Keypair>::new();
        let mut next = Vec::<Keypair>::new();
        for _ in 0..count {
            current.push(Keypair::new());
            next.push(Keypair::new());
        }
        Self { current, next }
    }
    pub fn get_pubkey(signer: &dyn Signer) -> Pubkey {
        signer.pubkey()
    }

    pub fn clone(kpairs: &Vec<Keypair>) -> Vec<Keypair> {
        kpairs
            .iter()
            .map(|kp| Keypair::from_base58_string(&kp.to_base58_string()))
            .collect::<Vec<Keypair>>()
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

impl KeySet for SolanaKeySet {
    fn rotate(&mut self) {
        self.current = SolanaKeySet::clone(&self.next);
        self.next = self
            .current
            .iter()
            .map(|_| Keypair::new())
            .collect::<Vec<Keypair>>()
    }
    fn current_private_keys(&self) -> Vec<Privatekey> {
        self.current
            .iter()
            .map(|x| Privatekey::new(x.to_bytes().to_vec()))
            .collect::<Vec<Privatekey>>()
    }

    fn next_private_keys(&self) -> Vec<Privatekey> {
        self.next
            .iter()
            .map(|x| Privatekey::new(x.to_bytes().to_vec()))
            .collect::<Vec<Privatekey>>()
    }

    fn current_public_keys(&self) -> Vec<Publickey> {
        self.current
            .iter()
            .map(|x| Publickey::new(SolanaKeySet::get_pubkey(x).to_bytes().to_vec()))
            .collect::<Vec<Publickey>>()
    }

    fn next_public_keys(&self) -> Vec<Publickey> {
        self.next
            .iter()
            .map(|x| Publickey::new(SolanaKeySet::get_pubkey(x).to_bytes().to_vec()))
            .collect::<Vec<Publickey>>()
    }
}

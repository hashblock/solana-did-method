//! Solana Keys Wrapper
use hbkr_rs::{
    basic::Basic,
    key_manage::{KeySet, PrivKey, Privatekey, Publickey},
};
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};

#[derive(Debug)]
pub struct SolanaKeySet {
    barren: bool,
    keytype: Basic,
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
        Self {
            barren: false,
            current,
            next,
            keytype: Basic::ED25519,
        }
    }
    /// Create an empty KeySet
    pub fn new_empty() -> Self {
        Self {
            barren: true,
            keytype: Basic::ED25519,
            current: Vec::<Keypair>::new(),
            next: Vec::<Keypair>::new(),
        }
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
    fn is_barren(&self) -> bool {
        self.barren
    }

    fn from(&mut self, current_ks: Vec<String>, next_ks: Vec<String>) {
        self.current = current_ks
            .iter()
            .map(|s| Keypair::from_base58_string(s))
            .collect::<Vec<Keypair>>();
        self.current = current_ks
            .iter()
            .map(|s| Keypair::from_base58_string(s))
            .collect::<Vec<Keypair>>();
    }

    fn rotate(&mut self, new_next: Option<Vec<Privatekey>>) -> (Vec<Privatekey>, Vec<Privatekey>) {
        self.current = SolanaKeySet::clone(&self.next);
        self.next = match new_next {
            Some(k) => k
                .iter()
                .map(|s| Keypair::from_base58_string(&s.as_base58_string()))
                .collect::<Vec<Keypair>>(),
            None => self
                .current
                .iter()
                .map(|_| Keypair::new())
                .collect::<Vec<Keypair>>(),
        };
        // self.next = self
        //     .current
        //     .iter()
        //     .map(|_| Keypair::new())
        //     .collect::<Vec<Keypair>>()
        (self.current_private_keys(), self.next_private_keys())
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

    fn key_type(&self) -> Basic {
        self.keytype
    }
}

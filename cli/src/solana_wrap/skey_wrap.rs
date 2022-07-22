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
            current: Vec::<Keypair>::new(),
            next: Vec::<Keypair>::new(),
            keytype: Basic::ED25519,
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
}

impl Clone for SolanaKeySet {
    fn clone(&self) -> Self {
        Self {
            barren: self.barren,
            keytype: self.keytype,
            current: SolanaKeySet::clone(&self.current),
            next: SolanaKeySet::clone(&self.next),
        }
    }
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
        self.next = next_ks
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

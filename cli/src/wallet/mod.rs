//! Wallet for local file management

use crate::errors::{SolDidError, SolDidResult};
use borsh::{BorshDeserialize, BorshSerialize};
use hbkr_rs::{
    basic::Basic,
    event::Event,
    event_message::EventMessage,
    key_manage::{KeySet, PrivKey},
    said_event::SaidEvent,
};

use std::{
    collections::HashSet,
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};

static DEFAULT_WALLET_PATH: &str = "/.solwall";
static WALLET_CONFIGURATION: &str = "wallet.bor";
static KEYS_CONFIGURATION: &str = "keys.bor";

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct Wallet {
    #[borsh_skip]
    root_path: PathBuf,
    #[borsh_skip]
    path: PathBuf,
    keynames: HashSet<String>,
    #[borsh_skip]
    keys: Vec<Keys>,
}
#[allow(dead_code)]
impl Wallet {
    /// Add new managed Keys(et) with name
    fn add_keys(&mut self, keysets: Keys) -> SolDidResult<()> {
        let check = keysets.name.clone();
        if self.keynames.contains(&check) {
            Err(SolDidError::KeysExistError(check))
        } else {
            self.keynames.insert(check);
            self.keys.push(keysets);
            self.save()?;
            Ok(())
        }
    }

    /// Instantiate a wallet for first time
    fn new(loc: PathBuf) -> SolDidResult<Wallet> {
        let _ = match loc.exists() {
            false => {
                fs::create_dir(loc.clone())?;
                true
            }
            _ => true,
        };
        let mut wallet_file = loc.clone();
        wallet_file.push(WALLET_CONFIGURATION);
        let mut wallet = Wallet {
            root_path: loc,
            path: wallet_file,
            keynames: HashSet::<String>::new(),
            keys: Vec::<Keys>::new(),
        };
        wallet.save()?;
        Ok(wallet)
    }

    // Helper function
    fn load_key(base: PathBuf, folder: &String) -> SolDidResult<Keys> {
        let mut key_file = base.clone();
        key_file.push(folder);
        Keys::load(&mut key_file)
    }

    /// Load a wallet from a path
    fn read_from_file(loc: PathBuf) -> SolDidResult<Wallet> {
        let mut wallet_file = loc.clone();
        let root_path = wallet_file.clone();
        wallet_file.push(WALLET_CONFIGURATION);
        match wallet_file.exists() {
            true => {
                let mut iw = Wallet::try_from_slice(&fs::read(wallet_file.clone())?)?;
                iw.root_path = root_path;
                iw.path = wallet_file;
                // Iterate through names loading each into keys
                iw.keys = iw
                    .keynames
                    .iter()
                    .map(|kn| Wallet::load_key(loc.clone(), kn).unwrap())
                    .collect::<Vec<Keys>>();
                Ok(iw)
            }
            false => {
                return Err(SolDidError::WalletConfigNotFound(
                    WALLET_CONFIGURATION.to_string(),
                    loc.to_str().unwrap().to_string(),
                ))
            }
        }
    }

    /// Write a wallet configuration to a path
    pub fn save(&mut self) -> SolDidResult<()> {
        let mut file = fs::File::create(self.path.clone())?;
        let wser = self.try_to_vec()?;
        file.write(&wser)?;
        for mkey in &mut self.keys {
            mkey.write(&self.root_path)?;
        }
        Ok(())
    }
}

/// Keys define a named collection of public and private keys
/// represented as strings
#[derive(BorshDeserialize, BorshSerialize, Debug, Default)]
pub struct Keys {
    #[borsh_skip]
    dirty: bool,
    name: String,
    threshold: u64,
    keysets_current: Vec<Key>,
    keysets_next: Vec<Key>,
    keysets_past: Vec<Key>,
}

enum KeyBlock {
    NONE,
    CURRENT,
    NEXT,
    PAST,
}
impl Keys {
    /// Accepts a native keyset that is pre-incepted
    /// distributes current (PreInception) and next NextRotation constructs
    pub fn from_pre_incept_set(
        name: &String,
        key_set: &dyn KeySet,
        key_type: Basic,
        threshold: u64,
    ) -> SolDidResult<Self> {
        let set_type = match key_type {
            Basic::ED25519 => KeyType::ED25519,
            Basic::PASTA => KeyType::PASTA,
            // _ => return Err(SolDidError::UnknownKeyTypeError),
        };
        let keysets_current = key_set
            .current_private_keys()
            .iter()
            .map(|k| Key::new(KeyState::PreInception, set_type, &k.as_base58_string()))
            .collect::<Vec<Key>>();
        let keysets_next = key_set
            .next_private_keys()
            .iter()
            .map(|k| Key::new(KeyState::NextRotation, set_type, &k.as_base58_string()))
            .collect::<Vec<Key>>();
        Ok(Keys {
            dirty: true,
            name: name.clone(),
            threshold,
            keysets_current,
            keysets_next,
            keysets_past: Vec::<Key>::new(),
        })
    }
    pub fn from_post_incept_set(
        name: &String,
        key_set: &dyn KeySet,
        key_type: Basic,
        threshold: u64,
    ) -> SolDidResult<Self> {
        let set_type = match key_type {
            Basic::ED25519 => KeyType::ED25519,
            Basic::PASTA => KeyType::PASTA,
            // _ => return Err(SolDidError::UnknownKeyTypeError),
        };
        let keysets_current = key_set
            .current_private_keys()
            .iter()
            .map(|k| Key::new(KeyState::Incepted, set_type, &k.as_base58_string()))
            .collect::<Vec<Key>>();
        let keysets_next = key_set
            .next_private_keys()
            .iter()
            .map(|k| Key::new(KeyState::NextRotation, set_type, &k.as_base58_string()))
            .collect::<Vec<Key>>();
        Ok(Keys {
            dirty: true,
            name: name.clone(),
            threshold,
            keysets_current,
            keysets_next,
            keysets_past: Vec::<Key>::new(),
        })
    }
    /// key_state_is returns the KeyState for
    /// keys in a keyset
    fn keys_state_is(&self, keyset: &Vec<Key>) -> SolDidResult<KeyState> {
        let res = keyset
            .iter()
            .map(|key| key.key_state)
            .collect::<HashSet<KeyState>>();
        if res.len() > 1 {
            Err(SolDidError::KeySetIncoherence)
        } else if res.len() == 0 {
            Ok(KeyState::PreInception)
        } else {
            let vres = res.into_iter().collect::<Vec<_>>();
            let vres_first = vres.first().unwrap().clone();
            Ok(vres_first)
        }
    }

    fn has_key(&self, key_string: &String) -> SolDidResult<(bool, KeyBlock)> {
        let mut hit = false;
        let mut block = KeyBlock::NONE;
        // Check current
        self.keysets_current.iter().for_each(|n| {
            if key_string == &n.key {
                hit = true;
                block = KeyBlock::CURRENT;
            }
        });
        // Check next
        if !hit {
            self.keysets_next.iter().for_each(|n| {
                if key_string == &n.key {
                    hit = true;
                    block = KeyBlock::NEXT;
                }
            })
        };
        // Check remaining
        if !hit {
            self.keysets_past.iter().for_each(|n| {
                if key_string == &n.key {
                    hit = true;
                    block = KeyBlock::PAST;
                }
            })
        };
        Ok((hit, block))
    }

    /// add_key_to_current can add a key into the current keyset if the
    /// key did not previously exist and the state of keys in
    /// current are in pre-inception
    pub fn add_key_to_current(&mut self, in_key: Key) -> SolDidResult<bool> {
        let (hit, _block) = self.has_key(&in_key.key)?;
        if hit {
            return Err(SolDidError::KeysExistError(in_key.key));
        } else {
            if !(self.keys_state_is(&self.keysets_current)? == KeyState::PreInception) {
                return Err(SolDidError::KeySetIncoherence);
            }
        }
        self.keysets_current.push(in_key);
        self.dirty = true;
        Ok(true)
    }

    /// add_key_to_next can add a key into the current keyset if the
    /// key did not previously exist and the state of keys in
    /// current are in pre-inception
    pub fn add_key_to_next(&mut self, in_key: Key) -> SolDidResult<bool> {
        let (hit, _block) = self.has_key(&in_key.key)?;
        if hit {
            return Err(SolDidError::KeysExistError(in_key.key));
        } else {
            if !(self.keys_state_is(&self.keysets_next)? == KeyState::NextRotation) {
                return Err(SolDidError::KeySetIncoherence);
            }
        }
        self.keysets_current.push(in_key);
        self.dirty = true;
        Ok(true)
    }

    /// inception_event occurs on current keyset being in PreInception
    pub fn inception_event(&mut self) -> SolDidResult<()> {
        if !(self.keys_state_is(&self.keysets_current)? == KeyState::PreInception) {
            return Err(SolDidError::KeySetIncoherence);
        }
        for k in self.keysets_current.iter_mut() {
            k.set_state(KeyState::Incepted)
        }
        Ok(())
    }
    /// rotation_event occurs adding new keys for the next rotation
    /// push the current keyset into the keysets_past
    /// makes the keysets.next into keysets_current
    /// sets the inbound keys to keyset_next
    pub fn rotation_event(&mut self, _new_next_set: Vec<Key>) -> SolDidResult<()> {
        if !(self.keys_state_is(&self.keysets_current)? == KeyState::PreInception) {
            return Err(SolDidError::KeySetIncoherence);
        }
        for k in self.keysets_current.iter_mut() {
            k.set_state(KeyState::Incepted)
        }
        Ok(())
    }

    /// Read keys for wallet from path
    fn load(loc: &mut PathBuf) -> SolDidResult<Keys> {
        loc.push(KEYS_CONFIGURATION);
        match loc.exists() {
            true => {
                let mut keys = Keys::try_from_slice(&fs::read(loc.clone())?)?;
                keys.dirty = false;
                Ok(keys)
            }
            false => {
                return Err(SolDidError::KeyConfigNotFound(
                    KEYS_CONFIGURATION.to_string(),
                    loc.to_str().unwrap().to_string(),
                ))
            }
        }
    }

    /// Write keys to location
    fn write(&mut self, loc: &PathBuf) -> SolDidResult<()> {
        let mut rpath = loc.clone();
        rpath.push(self.name.to_string());
        // If path does not exist, create
        if !rpath.exists() {
            fs::create_dir(rpath.clone())?;
        }
        rpath.push(KEYS_CONFIGURATION);
        let mut file = if !rpath.exists() {
            fs::File::create(rpath)?
        } else {
            fs::File::options().write(true).open(rpath)?
        };
        if self.dirty {
            // let mut file = fs::File::create(rpath)?;
            let wser = self.try_to_vec()?;
            file.write(&wser)?;
            self.dirty = false;
        }

        Ok(())
    }
    pub fn name(&self) -> &String {
        &self.name
    }
}

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Copy, Hash, Eq, PartialEq, PartialOrd)]
pub enum KeyState {
    PreInception,
    Incepted,
    NextRotation,
    Rotated,
    RotatedOut,
    Revoked,
}
#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Copy, Hash, Eq, PartialEq, PartialOrd)]
pub enum KeyType {
    ED25519,
    PASTA,
}

/// Key represents a keypair by encoding the private
/// key to a string. The keytype provider knows how
/// to reconstruct into it's keypair type
#[derive(Debug, BorshDeserialize, BorshSerialize, Hash, Eq, PartialEq, PartialOrd)]
pub struct Key {
    key_state: KeyState,
    key_type: KeyType,
    key: String,
}

impl Key {
    /// Create a new Key
    pub fn new(key_state: KeyState, key_type: KeyType, key: &String) -> Key {
        Key {
            key_state,
            key_type,
            key: key.clone(),
        }
    }
    fn set_state(&mut self, new_state: KeyState) {
        self.key_state = new_state;
    }
}

/// Attempts to initialize a wallet from the default
/// location. It will either create the path and default configuration
/// or read in the existing configuration from the default path
pub fn init_wallet() -> SolDidResult<Wallet> {
    let location = match env::var("HOME") {
        Ok(val) => val + DEFAULT_WALLET_PATH,
        Err(_) => return Err(SolDidError::HomeNotFoundError),
    };
    let wpath = Path::new(&location);
    match wpath.exists() {
        true => Wallet::read_from_file(wpath.to_path_buf()),
        false => Wallet::new(wpath.to_path_buf()),
    }
}

/// Load wallet from path
pub fn load_wallet_from(_: &Path) {}

/// Print to json string
pub fn to_json(title: &str, event: &EventMessage<SaidEvent<Event>>) {
    print!("{title}\n{}\n", serde_json::to_string(event).unwrap());
}

#[cfg(test)]
mod wallet_tests {

    use hbkr_rs::{basic::Basic, incept};

    use crate::pkey_wrap::PastaKeySet;
    use crate::skey_wrap::SolanaKeySet;

    use super::*;

    #[test]
    fn base_new_test_pass() -> SolDidResult<()> {
        let w = init_wallet()?;
        assert!(w.keynames.is_empty());
        fs::remove_dir_all(w.path.parent().unwrap())?;
        Ok(())
    }

    #[test]
    fn base_load_existing_pass() -> SolDidResult<()> {
        let w = init_wallet()?;
        println!("w {:?}", w);
        let w = init_wallet()?;
        println!("w {:?}", w);
        assert!(w.keynames.is_empty());
        fs::remove_dir_all(w.path.parent().unwrap())?;
        Ok(())
    }

    #[test]
    fn add_pasta_keys_test_pass() -> SolDidResult<()> {
        let mut w = init_wallet()?;
        assert!(w.keynames.is_empty());
        let count = 2u8;
        let threshold = 1u64;
        let kset1 = PastaKeySet::new_for(count);
        // Inception
        let _icp_event = incept(&kset1, Basic::PASTA, threshold)?;
        let wkeyset =
            Keys::from_post_incept_set(&"Frank".to_string(), &kset1, Basic::PASTA, threshold)?;
        w.add_keys(wkeyset)?;
        let w = init_wallet()?;
        assert_eq!(w.keynames.len(), 1);
        println!("\nWallet loaded \n{:?}", w);
        fs::remove_dir_all(w.path.parent().unwrap())?;
        Ok(())
    }
    #[test]
    fn add_solana_keys_test_pass() -> SolDidResult<()> {
        let mut w = init_wallet()?;
        assert!(w.keynames.is_empty());
        let count = 2u8;
        let threshold = 1u64;
        let kset1 = SolanaKeySet::new_for(count);
        // Inception
        let _icp_event = incept(&kset1, Basic::ED25519, threshold)?;
        let wkeyset =
            Keys::from_post_incept_set(&"Frank".to_string(), &kset1, Basic::PASTA, threshold)?;
        w.add_keys(wkeyset)?;
        let w = init_wallet()?;
        assert_eq!(w.keynames.len(), 1);
        // println!("\nWallet loaded \n{:?}", w);
        fs::remove_dir_all(w.path.parent().unwrap())?;
        Ok(())
    }
}

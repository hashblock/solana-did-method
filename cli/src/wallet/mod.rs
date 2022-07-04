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

impl Wallet {
    /// Add new Keys with name
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

    /// Convert hbkr KeySet to wallet keys
    pub fn register_keyset(
        &mut self,
        name: &String,
        key_set: &dyn KeySet,
        key_type: Basic,
        current_key_state: KeyState,
    ) -> SolDidResult<()> {
        let set_type = match key_type {
            Basic::ED25519 => KeyType::ED25519,
            Basic::PASTA => KeyType::PASTA,
            // _ => return Err(SolDidError::UnknownKeyTypeError),
        };
        let next_state = match current_key_state {
            KeyState::PreInception | KeyState::Incepted | KeyState::Rotated => {
                KeyState::NextRotation
            }
            KeyState::RotatedOut => todo!(),
            KeyState::Revoked => KeyState::Revoked,
            _ => todo!(),
        };
        let mut keys = Keys::default();
        keys.name = name.to_string();
        for n in key_set.current_private_keys() {
            keys.add_key(Key::new(current_key_state, set_type, &n.as_base58_string()))?;
        }
        for n in key_set.next_private_keys() {
            keys.add_key(Key::new(next_state, set_type, &n.as_base58_string()))?;
        }
        self.add_keys(keys)?;
        Ok(())
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
    fn load_key<'a>(base: PathBuf, folder: &String) -> SolDidResult<Keys> {
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
                    .map(|kn| {
                        Wallet::load_key(loc.clone(), kn).unwrap()
                        // key_file.push(kn);
                        // Keys::read(&key_file).unwrap()
                    })
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
    keysets: Vec<Key>,
}

impl Keys {
    pub fn add_key(&mut self, in_key: Key) -> SolDidResult<bool> {
        if !self.keysets.contains(&in_key) {
            self.keysets.push(in_key);
            self.dirty = true;
            Ok(true)
        } else {
            Err(SolDidError::DuplicateKeyError)
        }
    }

    pub fn change_key_state(&mut self, key_ref: &String, new_state: KeyState) -> SolDidResult<()> {
        let mut found = false;
        for n in self.keysets.iter_mut() {
            if n.key.eq(key_ref) {
                found = true;
                n.set_state(new_state);
                self.dirty = false;
            }
        }
        if found {
            Ok(())
        } else {
            Err(SolDidError::KeyNotFound(key_ref.to_string()))
        }
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
        let kset1 = PastaKeySet::new_for(count);
        // Inception
        let _icp_event = incept(&kset1, Basic::PASTA, 1u64)?;
        w.register_keyset(
            &"Frank".to_string(),
            &kset1,
            Basic::PASTA,
            KeyState::Incepted,
        )?;
        let w = init_wallet()?;
        assert_eq!(w.keynames.len(), 1);
        // println!("\nWallet loaded \n{:?}", w);
        fs::remove_dir_all(w.path.parent().unwrap())?;
        Ok(())
    }
    #[test]
    fn add_solana_keys_test_pass() -> SolDidResult<()> {
        let mut w = init_wallet()?;
        assert!(w.keynames.is_empty());
        let count = 2u8;
        let kset1 = SolanaKeySet::new_for(count);
        // Inception
        let _icp_event = incept(&kset1, Basic::ED25519, 1u64)?;
        w.register_keyset(
            &"Frank".to_string(),
            &kset1,
            Basic::ED25519,
            KeyState::Incepted,
        )?;
        let w = init_wallet()?;
        assert_eq!(w.keynames.len(), 1);
        // println!("\nWallet loaded \n{:?}", w);
        fs::remove_dir_all(w.path.parent().unwrap())?;
        Ok(())
    }
}

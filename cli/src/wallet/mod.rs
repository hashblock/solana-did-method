//! Wallet for local file management

use crate::errors::{SolDidError, SolDidResult};
use borsh::{BorshDeserialize, BorshSerialize};
use hbkr_rs::{
    basic::Basic,
    event::Event,
    event_message::EventMessage,
    key_manage::{KeySet, PrivKey, Privatekey},
    said_event::SaidEvent,
};

use std::{
    collections::{HashMap, HashSet},
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
    full_path: PathBuf,
    prefixes: HashSet<String>,
    #[borsh_skip]
    keys: Vec<Keys>,
}
#[allow(dead_code)]
impl Wallet {
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
            full_path: wallet_file,
            prefixes: HashSet::<String>::new(),
            keys: Vec::<Keys>::new(),
        };
        wallet.save()?;
        Ok(wallet)
    }
    /// Add new managed Keys(et) with name
    fn add_keys(&mut self, keysets: Keys) -> SolDidResult<()> {
        let check = keysets.name.clone();
        if self.prefixes.contains(&check) {
            Err(SolDidError::KeysExistError(check))
        } else {
            self.prefixes.insert(check);
            self.keys.push(keysets);
            self.save()?;
            Ok(())
        }
    }

    /// Incepts a new keyset
    pub fn incept_keys(&mut self, keyset: &dyn KeySet, threshold: u64) -> SolDidResult<()> {
        // Create a pre-inception keyset
        Ok(())
    }
    /// Rotate an existing keyset
    /// Takes
    ///     The prefix of the keyset (DID ID)
    ///     A vector of private key base58 strings for the next rotation
    ///     Optional threshold changes
    pub fn rotate_keys(
        &mut self,
        keyprefix: String,
        new_next_set: Vec<String>,
        threshold: Option<u64>,
    ) -> SolDidResult<()> {
        Ok(())
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
                iw.full_path = wallet_file;
                // Iterate through names loading each into keys
                iw.keys = iw
                    .prefixes
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
        let mut file = fs::File::create(self.full_path.clone())?;
        let wser = self.try_to_vec()?;
        file.write(&wser)?;
        for mkey in &mut self.keys {
            mkey.write(&self.root_path)?;
        }
        Ok(())
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
pub fn load_wallet_from(_location: &Path) {}

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
    chain_events: Vec<ChainEvent>,
}

/// Enum identifying which group a key exists in
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, Hash, Eq, PartialEq, PartialOrd)]
pub enum KeyBlock {
    NONE,
    CURRENT,
    NEXT,
    PAST,
}

/// ChainEven tracks/associates keys to a confirmed signature chain event
/// Signatures are base58 representation
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, Default)]
pub struct ChainEvent {
    pub chain_signature: String,
    pub keysets: HashMap<KeyBlock, Key>,
}

impl Keys {
    /// Accepts a native keyset that is pre-incepted
    /// distributes current (PreInception) and next (NextRotation) keys
    pub fn from_pre_incept_set(
        name: &String,
        key_set: &dyn KeySet,
        threshold: u64,
    ) -> SolDidResult<Self> {
        let set_type = match key_set.key_type() {
            Basic::ED25519 => KeyType::ED25519,
            Basic::PASTA => KeyType::PASTA,
            // _ => return Err(SolDidError::UnknownKeyTypeError),
        };
        Ok(Keys {
            dirty: true,
            name: name.clone(),
            threshold,
            keysets_current: Keys::to_keys_from_private(
                KeyState::PreInception,
                set_type,
                &key_set.current_private_keys(),
            ),
            keysets_next: Keys::to_keys_from_private(
                KeyState::NextRotation,
                set_type,
                &key_set.next_private_keys(),
            ),
            keysets_past: Vec::<Key>::new(),
            chain_events: Vec::<ChainEvent>::new(),
        })
    }

    /// Accepts a native keyset this has been incepted
    /// distributes current (Incepted) and next (NextRotation) keys
    pub fn from_post_incept_set(
        name: &String,
        key_set: &dyn KeySet,
        threshold: u64,
    ) -> SolDidResult<Self> {
        let set_type = match key_set.key_type() {
            Basic::ED25519 => KeyType::ED25519,
            Basic::PASTA => KeyType::PASTA,
            // _ => return Err(SolDidError::UnknownKeyTypeError),
        };
        Ok(Keys {
            dirty: true,
            name: name.clone(),
            threshold,
            keysets_current: Keys::to_keys_from_private(
                KeyState::Incepted,
                set_type,
                &key_set.current_private_keys(),
            ),
            keysets_next: Keys::to_keys_from_private(
                KeyState::NextRotation,
                set_type,
                &key_set.next_private_keys(),
            ),
            keysets_past: Vec::<Key>::new(),
            chain_events: Vec::<ChainEvent>::new(),
        })
    }

    /// Generate Keys from Privatekeys
    fn to_keys_from_private(state: KeyState, ktype: KeyType, pkeys: &Vec<Privatekey>) -> Vec<Key> {
        pkeys
            .iter()
            .map(|pk| Key::new(state, ktype, &pk.as_base58_string()))
            .collect::<Vec<Key>>()
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
            Ok(vres.first().unwrap().clone())
        }
    }

    /// Checks current, next and past to see if key_string
    /// already exists
    fn has_key(&self, key_string: &String) -> (bool, KeyBlock) {
        // Check current
        for n in &self.keysets_current {
            if key_string == &n.key {
                return (true, KeyBlock::CURRENT);
            }
        }
        // Check next
        for n in &self.keysets_next {
            if key_string == &n.key {
                return (true, KeyBlock::NEXT);
            }
        }
        // Check past
        for n in &self.keysets_past {
            if key_string == &n.key {
                return (true, KeyBlock::PAST);
            }
        }
        (false, KeyBlock::NONE)
    }

    /// inception_event occurs on current keyset being in PreInception
    pub fn inception_event(&mut self) -> SolDidResult<()> {
        // Fail when current keyset is not in PreInception
        if !(self.keys_state_is(&self.keysets_current)? == KeyState::PreInception) {
            return Err(SolDidError::KeySetIncoherence);
        }
        // TODO: Execute inception transaction on chain
        // TODO: Check for success
        // Build a ChainEvent
        let mut _chain_event = ChainEvent::default();

        // Mark current keys as Incepted
        for k in self.keysets_current.iter_mut() {
            k.set_state(KeyState::Incepted)
        }
        // TODO: Copy the involved keys in the ChainEvent
        // TODO: Update (push) the keys ChainEvents
        self.dirty = true;
        Ok(())
    }

    /// rotation_event occurs adding new keys for the next rotation
    /// push the current keyset into the keysets_past
    /// makes the keysets.next into keysets_current
    /// sets the inbound keys to keyset_next
    pub fn rotation_event(
        &mut self,
        key_type: Basic,
        new_next_set: Vec<String>,
    ) -> SolDidResult<()> {
        let _ = match self.keys_state_is(&self.keysets_current)? {
            KeyState::PreInception
            | KeyState::Revoked
            | KeyState::RotatedOut
            | KeyState::NextRotation => return Err(SolDidError::KeySetIncoherence),
            _ => true,
        };
        let set_type = match key_type {
            Basic::ED25519 => KeyType::ED25519,
            Basic::PASTA => KeyType::PASTA,
            // _ => return Err(SolDidError::UnknownKeyTypeError),
        };
        let mut _chain_event = ChainEvent::default();
        // Move current to past
        for k in self.keysets_current.iter_mut() {
            self.keysets_past
                .push(Key::new(KeyState::RotatedOut, k.key_type, &k.key))
        }
        // Move next to current
        self.keysets_current.drain(..);
        for k in self.keysets_next.iter() {
            self.keysets_current
                .push(Key::new(KeyState::Rotated, k.key_type, &k.key))
        }
        // New next keyset
        self.keysets_next.drain(..);
        for k in new_next_set.iter() {
            self.keysets_next
                .push(Key::new(KeyState::NextRotation, set_type, k))
        }
        self.dirty = true;
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
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, Hash, Eq, PartialEq, PartialOrd)]
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

/// Print to json string
pub fn to_json(title: &str, event: &EventMessage<SaidEvent<Event>>) {
    print!("{title}\n{}\n", serde_json::to_string(event).unwrap());
}

#[cfg(test)]
mod wallet_tests {

    use hbkr_rs::{basic::Basic, incept};
    use hbpasta_rs::Keypair as PastaKP;

    use crate::pkey_wrap::PastaKeySet;
    use crate::skey_wrap::SolanaKeySet;

    use super::*;

    #[test]
    fn base_wallet_create_test_pass() -> SolDidResult<()> {
        let w = init_wallet()?;
        assert!(w.prefixes.is_empty());
        fs::remove_dir_all(w.full_path.parent().unwrap())?;
        Ok(())
    }

    #[test]
    fn base_load_existing_pass() -> SolDidResult<()> {
        let _ = init_wallet()?;
        let w = init_wallet()?;
        assert!(w.prefixes.is_empty());
        fs::remove_dir_all(w.full_path.parent().unwrap())?;
        Ok(())
    }
    #[test]
    fn has_keys_test_pass() -> SolDidResult<()> {
        let count = 2u8;
        let threshold = 1u64;
        let kset1 = PastaKeySet::new_for(count);
        let wkeyset = Keys::from_pre_incept_set(&"Frank".to_string(), &kset1, threshold)?;
        let one_key = kset1
            .current_private_keys()
            .first()
            .unwrap()
            .as_base58_string();
        let (found, block) = wkeyset.has_key(&one_key);
        assert!(found);
        assert_eq!(block, KeyBlock::CURRENT);
        Ok(())
    }

    #[test]
    fn has_keys_test_fail() -> SolDidResult<()> {
        let count = 2u8;
        let threshold = 1u64;
        let kset1 = PastaKeySet::new_for(count);
        let wkeyset = Keys::from_pre_incept_set(&"Frank".to_string(), &kset1, threshold)?;
        let err_key = PastaKP::new();
        let res = wkeyset.has_key(&err_key.to_base58_string());
        assert!(!res.0);
        Ok(())
    }

    #[test]
    fn inception_event_pass() -> SolDidResult<()> {
        let count = 2u8;
        let threshold = 1u64;
        let kset1 = PastaKeySet::new_for(count);
        let mut wkeyset = Keys::from_pre_incept_set(&"Frank".to_string(), &kset1, threshold)?;
        assert_eq!(
            wkeyset.keys_state_is(&wkeyset.keysets_current)?,
            KeyState::PreInception
        );

        wkeyset.inception_event()?;
        assert_eq!(
            wkeyset.keys_state_is(&wkeyset.keysets_current)?,
            KeyState::Incepted,
        );
        Ok(())
    }
    #[test]
    fn rotation_event_pass() -> SolDidResult<()> {
        let count = 2u8;
        let threshold = 1u64;
        let kset1 = PastaKeySet::new_for(count);
        let mut wkeyset = Keys::from_post_incept_set(&"Frank".to_string(), &kset1, threshold)?;
        assert_eq!(
            wkeyset.keys_state_is(&wkeyset.keysets_current)?,
            KeyState::Incepted
        );
        let kset2 = PastaKeySet::new_for(count);
        let pkeys = kset2
            .current_private_keys()
            .iter()
            .map(|s| s.as_base58_string())
            .collect::<Vec<String>>();

        wkeyset.rotation_event(Basic::PASTA, pkeys)?;
        assert_eq!(
            wkeyset.keys_state_is(&wkeyset.keysets_current)?,
            KeyState::Rotated
        );
        assert_eq!(
            wkeyset.keys_state_is(&wkeyset.keysets_next)?,
            KeyState::NextRotation
        );
        Ok(())
    }
    #[test]
    fn add_incepted_pasta_keys_test_pass() -> SolDidResult<()> {
        let mut w = init_wallet()?;
        assert!(w.prefixes.is_empty());
        let count = 2u8;
        let threshold = 1u64;
        let kset1 = PastaKeySet::new_for(count);
        // Inception
        let _icp_event = incept(&kset1, Basic::PASTA, threshold)?;
        let wkeyset = Keys::from_post_incept_set(&"Frank".to_string(), &kset1, threshold)?;
        let one_key = kset1
            .current_private_keys()
            .first()
            .unwrap()
            .as_base58_string();
        assert!(wkeyset.has_key(&one_key).0);
        w.add_keys(wkeyset)?;
        let w = init_wallet()?;
        assert_eq!(w.prefixes.len(), 1);
        // println!("\nWallet loaded \n{:?}", w);
        fs::remove_dir_all(w.full_path.parent().unwrap())?;
        Ok(())
    }
    #[test]
    fn add_incepted_solana_keys_test_pass() -> SolDidResult<()> {
        let mut w = init_wallet()?;
        assert!(w.prefixes.is_empty());
        let count = 2u8;
        let threshold = 1u64;
        let kset1 = SolanaKeySet::new_for(count);
        // Inception
        let _icp_event = incept(&kset1, Basic::ED25519, threshold)?;
        let wkeyset = Keys::from_post_incept_set(&"Frank".to_string(), &kset1, threshold)?;
        w.add_keys(wkeyset)?;
        let w = init_wallet()?;
        assert_eq!(w.prefixes.len(), 1);
        // println!("\nWallet loaded \n{:?}", w);
        fs::remove_dir_all(w.full_path.parent().unwrap())?;
        Ok(())
    }
}

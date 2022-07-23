//! Wallet for local file management

pub mod chain_event;
pub mod wallet_enums;

use crate::{
    chain_trait::Chain,
    errors::{SolDidError, SolDidResult},
};
use borsh::{BorshDeserialize, BorshSerialize};
use chain_event::{ChainEvent, ChainEventType, KeyBlock};
use hbkr_rs::{
    event::Event,
    event_message::EventMessage,
    inception,
    key_manage::{KeySet, PrivKey, Privatekey},
    rotation,
    said_event::SaidEvent,
    Prefix,
};

use std::{
    collections::HashSet,
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};

use self::wallet_enums::{KeyState, KeyType};

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
        let check = keysets.prefix.clone();
        if self.prefixes.contains(&check) {
            Err(SolDidError::KeysExistError(check))
        } else {
            self.prefixes.insert(check);
            self.keys.push(keysets);
            self.save()?;
            Ok(())
        }
    }

    /// Creates a new DID with keyset
    pub fn new_did(
        &mut self,
        keyset: &dyn KeySet,
        threshold: u64,
        chain: Option<&dyn Chain>,
    ) -> SolDidResult<String> {
        let (keys, prefix) = Keys::incept_keys(chain, keyset, threshold)?;
        self.add_keys(keys)?;
        Ok(prefix)
    }
    /// Rotate a DID
    /// Takes
    ///     The prefix (DID ID)
    ///     A barren keyset
    ///     Optional vector of private keys to use as the next rotation
    ///     Optional new threshold to set for keyset
    ///     Optional chain to commit to
    pub fn rotate_did(
        &mut self,
        keyprefix: String,
        keyset: &mut dyn KeySet,
        new_next_set: Option<Vec<Privatekey>>,
        threshold: Option<u64>,
        chain: Option<&dyn Chain>,
    ) -> SolDidResult<()> {
        // Validate keyset is barren
        if !keyset.is_barren() {
            return Err(SolDidError::KeySetIncoherence);
        }
        // Get the prefix Keys
        match self.keys.iter_mut().find(|k| k.prefix == keyprefix) {
            Some(k) => k.rotate_keys(keyset, new_next_set, threshold, chain),
            None => Err(SolDidError::KeySetIncoherence),
        }
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
pub fn load_wallet_from(location: &PathBuf) -> SolDidResult<Wallet> {
    let mut wallet_path = location.clone();
    wallet_path.push(WALLET_CONFIGURATION);
    Wallet::read_from_file(wallet_path.to_path_buf())
}

/// Keys define a named collection of public and private keys
/// represented as strings
#[derive(BorshDeserialize, BorshSerialize, Debug, Default)]
pub struct Keys {
    #[borsh_skip]
    dirty: bool,
    prefix: String,
    threshold: u64,
    keysets_current: Vec<Key>,
    keysets_next: Vec<Key>,
    keysets_past: Vec<Key>,
    chain_events: Vec<ChainEvent>,
}

impl Keys {
    /// Accepts a native keyset this has been incepted
    /// distributes current (Incepted) and next (NextRotation) keys
    /// and stores the chain event initiating this function call
    fn incept_keys(
        chain: Option<&dyn Chain>,
        key_set: &dyn KeySet,
        threshold: u64,
    ) -> SolDidResult<(Self, String)> {
        // Create an inception event
        let icp_event = inception(key_set, threshold)?;
        // Optionally store on chain
        let signature = match chain {
            Some(chain) => chain.inception_inst(&icp_event)?,
            None => "sol_did_signature".to_string(),
        };

        // Covert Type
        let set_type = KeyType::from(key_set.key_type());
        // Setup the chain event
        let mut chain_event = ChainEvent::from(&icp_event);
        let prefix = icp_event.event.get_prefix().to_str();
        chain_event.km_keytype = set_type;
        chain_event.did_signature = signature.clone();

        // Convert keyset current keys and next keys to Key
        let keysets_current = Keys::to_keys_from_private(
            KeyState::Incepted,
            set_type,
            &key_set.current_private_keys(),
        );
        let keysets_next = Keys::to_keys_from_private(
            KeyState::NextRotation,
            set_type,
            &key_set.next_private_keys(),
        );
        // Duplicate into chain_event
        chain_event
            .keysets
            .insert(KeyBlock::CURRENT, keysets_current.clone());
        chain_event
            .keysets
            .insert(KeyBlock::NEXT, keysets_next.clone());
        // Create a event store and push chain_event
        let mut chain_vec = Vec::<ChainEvent>::new();
        chain_vec.push(chain_event);
        // Return Self
        Ok((
            Keys {
                dirty: true,
                prefix,
                threshold,
                keysets_current,
                keysets_next,
                keysets_past: Vec::<Key>::new(),
                chain_events: chain_vec,
            },
            signature,
        ))
    }

    /// Generate Keys from Privatekeys
    fn to_keys_from_private(state: KeyState, ktype: KeyType, pkeys: &Vec<Privatekey>) -> Vec<Key> {
        pkeys
            .iter()
            .map(|pk| Key::new(state, ktype, &pk.as_base58_string()))
            .collect::<Vec<Key>>()
    }

    /// rotate_keys creates a new rotation event and
    /// optionally commits to blockchain and
    /// then syncs current state and updates the chainevents
    fn rotate_keys(
        &mut self,
        barren_ks: &mut dyn KeySet,
        new_next_set: Option<Vec<Privatekey>>,
        threshold: Option<u64>,
        chain: Option<&dyn Chain>,
    ) -> SolDidResult<()> {
        // Validate state
        if self.chain_events.len() == 0 {
            return Err(SolDidError::RotationIncoherence);
        }
        // Validate ability to rotate
        let last_event = self.chain_events.last().unwrap();
        if !ChainEventType::can_rotate(last_event.event_type) {
            return Err(SolDidError::RotationIncompatible);
        }
        // Re-hydrate the keystate
        barren_ks.from(
            last_event
                .get_keys_for(KeyBlock::CURRENT)?
                .iter()
                .map(|k| k.key.clone())
                .collect::<Vec<String>>(),
            last_event
                .get_keys_for(KeyBlock::NEXT)?
                .iter()
                .map(|k| k.key.clone())
                .collect::<Vec<String>>(),
        );
        // Default rotation of keys should create equivalent count of keysets for next
        let (ncurr, nnext) = barren_ks.rotate(new_next_set);
        // Rotate event
        let rot_event = rotation(
            &self.prefix,
            &last_event.km_digest,
            last_event.km_sn + 1,
            barren_ks,
            match threshold {
                Some(t) => {
                    self.threshold = t;
                    t
                }
                None => self.threshold,
            },
        )?;
        // Optionally store on chain
        let signature = match chain {
            Some(chain) => chain.rotation_inst_fn(&rot_event)?,
            None => "sol_did_signature".to_string(),
        };
        // Repopulate our keysets
        let keytype = KeyType::from(barren_ks.key_type());
        let last_curr = self.keysets_current.clone();
        self.keysets_current = Keys::to_keys_from_private(KeyState::Rotated, keytype, &ncurr);
        self.keysets_next = Keys::to_keys_from_private(KeyState::NextRotation, keytype, &nnext);
        self.dirty = true;
        // Create the chain event
        let mut chain_event = ChainEvent::from(&rot_event);
        chain_event.event_type = ChainEventType::Rotation;
        chain_event.km_keytype = keytype;
        chain_event.did_signature = signature.clone();
        // Capture key states
        chain_event.keysets.insert(KeyBlock::PAST, last_curr);
        chain_event
            .keysets
            .insert(KeyBlock::CURRENT, self.keysets_current.clone());
        chain_event
            .keysets
            .insert(KeyBlock::NEXT, self.keysets_next.clone());

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
        rpath.push(self.prefix.to_string());
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
    pub fn prefix(&self) -> &String {
        &self.prefix
    }
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
}

/// Print to json string
pub fn to_json(title: &str, event: &EventMessage<SaidEvent<Event>>) {
    print!("{title}\n{}\n", serde_json::to_string(event).unwrap());
}

#[cfg(test)]
mod wallet_tests {

    use super::*;
    use crate::pkey_wrap::PastaKeySet;

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
    fn inception_pasta_keys_test_pass() -> SolDidResult<()> {
        let mut w = init_wallet()?;
        assert!(w.prefixes.is_empty());
        let count = 2i8;
        let threshold = 1u64;
        let kset1 = PastaKeySet::new_for(count);
        let prefix = w.new_did(&kset1, threshold, None)?;
        assert_eq!("sol_did_signature".to_string(), prefix);
        let w = init_wallet()?;
        assert_eq!(w.prefixes.len(), 1);
        fs::remove_dir_all(w.full_path.parent().unwrap())?;
        Ok(())
    }

    #[test]
    fn rotation_pasta_keys_test_pass() -> SolDidResult<()> {
        let mut w = init_wallet()?;
        assert!(w.prefixes.is_empty());
        let count = 2i8;
        let threshold = 1u64;
        let kset1 = PastaKeySet::new_for(count);
        let _prefix = w.new_did(&kset1, threshold, None)?;
        let w = init_wallet()?;
        assert_eq!(w.prefixes.len(), 1);
        // Target prefix we want to rotation
        let new_first = w.keys.first().unwrap().clone();
        let prefix = new_first.prefix().to_string();
        // Rotate
        let mut w = init_wallet()?;
        let mut barren_ks = PastaKeySet::new_empty();
        let _ = w.rotate_did(prefix.clone(), &mut barren_ks, None, None, None)?;
        // Observe
        let rot_keys = w.keys.first().unwrap();
        let rot_prefix = rot_keys.prefix();
        assert_eq!(rot_prefix, &prefix);
        assert_eq!(
            new_first.keysets_next.first().unwrap().key,
            rot_keys.keysets_current.first().unwrap().key
        );
        fs::remove_dir_all(w.full_path.parent().unwrap())?;
        Ok(())
    }
}

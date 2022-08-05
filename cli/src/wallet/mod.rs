//! Wallet for local file management

pub mod chain_event;
pub mod generic_keys;
pub mod wallet_enums;

use crate::{
    chain_trait::Chain,
    errors::{SolDidError, SolDidResult},
};
use borsh::{BorshDeserialize, BorshSerialize};

use hbkr_rs::{
    event::Event,
    event_message::EventMessage,
    key_manage::{KeySet, Privatekey},
    said_event::SaidEvent,
};

use std::{
    collections::HashSet,
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};

use self::generic_keys::Keys;

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
    /// Get the full path for the wallet
    pub fn full_path(&self) -> &PathBuf {
        &self.full_path
    }
    /// Add new managed Keys(et) with name
    fn add_keys(&mut self, keysets: Keys) -> SolDidResult<()> {
        let check = keysets.prefix();
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
    ) -> SolDidResult<(String, String, Vec<u8>)> {
        let (keys, signature, prefix, digest) = Keys::incept_keys(chain, keyset, threshold)?;
        self.add_keys(keys)?;
        Ok((signature, prefix, digest))
    }
    /// Rotate a DID
    /// Takes
    ///     The prefix (DID ID)
    ///     A barren keyset
    ///     Optional vector of private keys to use as the next rotation
    ///     Optional new threshold to set for keyset
    ///     Optional chain to commit to
    /// Returns Transaction Signature and Rotation digest
    pub fn rotate_did(
        &mut self,
        keyprefix: String,
        keyset: &mut dyn KeySet,
        new_next_set: Option<Vec<Privatekey>>,
        threshold: Option<u64>,
        chain: Option<&dyn Chain>,
    ) -> SolDidResult<(String, Vec<u8>)> {
        // Validate keyset is barren
        if !keyset.is_barren() {
            Err(SolDidError::KeySetIncoherence)
        } else {
            // Get the prefix Keys
            match self.keys.iter_mut().find(|k| k.prefix() == keyprefix) {
                Some(k) => {
                    let result = k.rotate_keys(keyset, new_next_set, threshold, chain);
                    if result.is_ok() {
                        self.save()?;
                    }
                    result
                }
                None => Err(SolDidError::KeySetIncoherence),
            }
        }
    }

    /// Decommission a did by rotating in an empty vector of Privatekeys
    pub fn decommission_did(
        &mut self,
        keyprefix: String,
        keyset: &mut dyn KeySet,
        chain: Option<&dyn Chain>,
    ) -> SolDidResult<(String, Vec<u8>)> {
        if !keyset.is_barren() {
            Err(SolDidError::KeySetIncoherence)
        } else {
            match self.keys.iter_mut().find(|k| k.prefix() == keyprefix) {
                Some(k) => {
                    let result = k.decommission_keys(keyset, chain);
                    if result.is_ok() {
                        self.save()?;
                    }
                    result
                }
                None => Err(SolDidError::KeySetIncoherence),
            }
        }
    }
    /// Returns keyset Keys for prefix
    pub fn keys_for(&self, prefix: &String) -> SolDidResult<&Keys> {
        // Get the prefix Keys
        match self.keys.iter().find(|k| &k.prefix() == prefix) {
            Some(k) => Ok(k),
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

/// Print to json string
pub fn to_json(title: &str, event: &EventMessage<SaidEvent<Event>>) {
    print!("{title}\n{}\n", serde_json::to_string(event).unwrap());
}

#[cfg(test)]
mod wallet_tests {

    use super::*;
    use crate::{pkey_wrap::PastaKeySet, wallet::chain_event::KeyBlock};

    #[test]
    /// Test wallet simple creation
    fn test_base_wallet_create_pass() -> SolDidResult<()> {
        let w = init_wallet()?;
        assert!(w.prefixes.is_empty());
        fs::remove_dir_all(w.full_path.parent().unwrap())?;
        Ok(())
    }

    #[test]
    /// Test wallet simple load
    fn test_base_load_existing_pass() -> SolDidResult<()> {
        let _ = init_wallet()?;
        let w = init_wallet()?;
        assert!(w.prefixes.is_empty());
        fs::remove_dir_all(w.full_path.parent().unwrap())?;
        Ok(())
    }

    #[test]
    /// Test an inception event
    fn test_inception_pasta_keys_pass() -> SolDidResult<()> {
        let mut w = init_wallet()?;
        assert!(w.prefixes.is_empty());
        assert!(w.keys.is_empty());
        let count = 2i8;
        let threshold = 1u64;
        let kset1 = PastaKeySet::new_for(count);
        let (signature, prefix, digest) = w.new_did(&kset1, threshold, None)?;
        assert_eq!("sol_did_signature".to_string(), signature);
        assert!(!digest.is_empty());
        let k = w.keys_for(&prefix)?;
        assert_eq!(prefix, k.prefix());
        let w = init_wallet()?;
        assert_eq!(w.prefixes.len(), 1);
        assert_eq!(w.keys.len(), 1);
        fs::remove_dir_all(w.full_path.parent().unwrap())?;
        Ok(())
    }

    #[test]
    /// Test rotation event to default keys
    fn test_rotation_pasta_keys_pass() -> SolDidResult<()> {
        let mut w = init_wallet()?;
        assert!(w.prefixes.is_empty());
        let count = 2i8;
        let threshold = 1u64;
        let kset1 = PastaKeySet::new_for(count);
        let (_signature, _prefix, _digest) = w.new_did(&kset1, threshold, None)?;
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
        assert_eq!(rot_prefix, prefix);
        // assert_eq!(
        //     new_first.keysets_next.first().unwrap().key,
        //     rot_keys.keysets_current.first().unwrap().key
        // );
        fs::remove_dir_all(w.full_path.parent().unwrap())?;
        Ok(())
    }

    #[test]
    /// Test rotation to different keys than default
    fn test_rotation_to_different_pasta_keys_pass() -> SolDidResult<()> {
        let mut w = init_wallet()?;
        assert!(w.prefixes.is_empty());
        let count = 2i8;
        let threshold = 1u64;
        let kset1 = PastaKeySet::new_for(count);
        let (_signature, _prefix, _digest) = w.new_did(&kset1, threshold, None)?;
        let new_first = w.keys.first().unwrap().prefix().to_string();
        assert_eq!(w.keys_for(&new_first)?.chain_event_len(), 1);
        // Rotate
        let mut w = init_wallet()?;
        let mut barren_ks = PastaKeySet::new_empty();
        let kset2 = PastaKeySet::new_for(count);
        let new_next_set = kset2.current_private_keys();
        let _ = w.rotate_did(
            new_first.clone(),
            &mut barren_ks,
            Some(new_next_set.clone()),
            None,
            None,
        )?;
        let chain_events = w.keys_for(&new_first)?.chain_events();
        assert_eq!(chain_events.len(), 2);
        let next_privates = chain_events
            .last()
            .unwrap()
            .get_keys_as_private_for(KeyBlock::NEXT)?;
        assert_eq!(new_next_set, next_privates);
        fs::remove_dir_all(w.full_path.parent().unwrap())?;
        Ok(())
    }

    #[test]
    /// Rotate to empty vector of next keeys fails
    /// as this is a decommission event
    fn test_rotate_to_empty_vector_fail() -> SolDidResult<()> {
        let mut w = init_wallet()?;
        assert!(w.prefixes.is_empty());
        let count = 2i8;
        let threshold = 1u64;
        let kset1 = PastaKeySet::new_for(count);
        let (_signature, _prefix, _digest) = w.new_did(&kset1, threshold, None)?;
        let new_first = w.keys.first().unwrap().prefix().to_string();
        assert_eq!(w.keys_for(&new_first)?.chain_event_len(), 1);
        // Rotate to empty
        let mut w = init_wallet()?;
        let mut barren_ks = PastaKeySet::new_empty();
        let new_next_set = Vec::<Privatekey>::new();
        let result = w.rotate_did(
            new_first.clone(),
            &mut barren_ks,
            Some(new_next_set.clone()),
            None,
            None,
        );
        assert!(result.is_err());
        fs::remove_dir_all(w.full_path.parent().unwrap())?;
        Ok(())
    }

    #[test]
    fn test_decommission_pass() -> SolDidResult<()> {
        let mut w = init_wallet()?;
        assert!(w.prefixes.is_empty());
        let count = 2i8;
        let threshold = 1u64;
        let kset1 = PastaKeySet::new_for(count);
        let (_signature, _prefix, _digest) = w.new_did(&kset1, threshold, None)?;
        let new_first = w.keys.first().unwrap().prefix().to_string();
        assert_eq!(w.keys_for(&new_first)?.chain_event_len(), 1);
        // Decommission keys
        let mut w = init_wallet()?;
        let mut barren_ks = PastaKeySet::new_empty();
        let result = w.decommission_did(new_first.clone(), &mut barren_ks, None);
        assert!(result.is_ok());
        fs::remove_dir_all(w.full_path.parent().unwrap())?;
        Ok(())
    }
}

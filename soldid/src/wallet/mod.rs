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
        let check = keysets.prefix().clone();
        if self.prefixes.contains(&check) {
            Err(SolDidError::KeysPrefixExistError(check))
        } else {
            self.prefixes.insert(check);
            self.keys.push(keysets);
            self.save()?;
            Ok(())
        }
    }

    /// Check if named keyset exists or not
    fn key_name_exists(&self, name: &String) -> bool {
        for keys in &self.keys {
            if keys.name() == name {
                return true;
            }
        }
        false
    }

    /// Creates a new DID with keyset
    pub fn new_did(
        &mut self,
        name: &String,
        keyset: &dyn KeySet,
        threshold: i8,
        chain: Option<&dyn Chain>,
    ) -> SolDidResult<(String, String, Vec<u8>)> {
        if self.key_name_exists(name) {
            return Err(SolDidError::KeysNameExistError(name.to_string()));
        }
        let (keys, signature, prefix, digest) = Keys::incept_keys(name, chain, keyset, threshold)?;
        self.add_keys(keys)?;
        Ok((signature, prefix, digest))
    }
    /// Rotate a DID using a prefix
    /// Takes
    ///     The prefix (DID ID)
    ///     A barren keyset
    ///     Optional vector of private keys to use as the next rotation
    ///     Optional new threshold to set for keyset
    ///     Optional chain to commit to
    /// Returns Transaction Signature and Rotation digest
    pub fn rotate_did_with_prefix(
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
            match self.keys.iter_mut().find(|k| k.prefix() == &keyprefix) {
                Some(k) => {
                    let result = k.rotate_keys(keyset, new_next_set, threshold, chain);
                    if result.is_ok() {
                        self.save()?;
                    }
                    result
                }
                None => Err(SolDidError::PrefixNotFound(keyprefix)),
            }
        }
    }
    /// Rotate a DID using a prefix
    /// Takes
    ///     The prefix (DID ID)
    ///     A barren keyset
    ///     Optional vector of private keys to use as the next rotation
    ///     Optional new threshold to set for keyset
    ///     Optional chain to commit to
    /// Returns Transaction Signature and Rotation digest
    pub fn rotate_did_with_name(
        &mut self,
        keyname: String,
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
            match self.keys.iter_mut().find(|k| k.name() == &keyname) {
                Some(k) => {
                    let result = k.rotate_keys(keyset, new_next_set, threshold, chain);
                    if result.is_ok() {
                        self.save()?;
                    }
                    result
                }
                None => Err(SolDidError::NameNotFound(keyname)),
            }
        }
    }
    /// Decommission a did by rotating in an empty vector of Privatekeys
    pub fn decommission_did_with_prefix(
        &mut self,
        keyprefix: String,
        keyset: &mut dyn KeySet,
        chain: Option<&dyn Chain>,
    ) -> SolDidResult<(String, Vec<u8>)> {
        if !keyset.is_barren() {
            Err(SolDidError::KeySetIncoherence)
        } else {
            match self.keys.iter_mut().find(|k| k.prefix() == &keyprefix) {
                Some(k) => {
                    let result = k.decommission_keys(keyset, chain);
                    if result.is_ok() {
                        self.save()?;
                    }
                    result
                }
                None => Err(SolDidError::PrefixNotFound(keyprefix)),
            }
        }
    }

    /// Decommission a did by rotating in an empty vector of Privatekeys
    pub fn decommission_did_with_name(
        &mut self,
        keyname: String,
        keyset: &mut dyn KeySet,
        chain: Option<&dyn Chain>,
    ) -> SolDidResult<(String, Vec<u8>)> {
        if !keyset.is_barren() {
            Err(SolDidError::KeySetIncoherence)
        } else {
            match self.keys.iter_mut().find(|k| k.name() == &keyname) {
                Some(k) => {
                    let result = k.decommission_keys(keyset, chain);
                    if result.is_ok() {
                        self.save()?;
                    }
                    result
                }
                None => Err(SolDidError::NameNotFound(keyname)),
            }
        }
    }

    /// Return all keysets
    pub fn keys(&self) -> SolDidResult<&Vec<Keys>> {
        Ok(&self.keys)
    }
    /// Returns keyset Keys for prefix
    pub fn keys_for_prefix(&self, prefix: &String) -> SolDidResult<&Keys> {
        // Get the prefix Keys
        match self.keys.iter().find(|k| k.prefix() == prefix) {
            Some(k) => Ok(k),
            None => Err(SolDidError::PrefixNotFound(prefix.to_string())),
        }
    }
    /// Returns keyset Keys for name
    pub fn keys_for_name(&self, name: &String) -> SolDidResult<&Keys> {
        // Get the prefix Keys
        match self.keys.iter().find(|k| k.name() == name) {
            Some(k) => Ok(k),
            None => Err(SolDidError::NameNotFound(name.to_string())),
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
        let root_path = if wallet_file.file_name().unwrap() == WALLET_CONFIGURATION {
            wallet_file.parent().unwrap().to_path_buf()
        } else {
            wallet_file.push(WALLET_CONFIGURATION);
            wallet_file.parent().unwrap().to_path_buf()
        };
        // let root_path = wallet_file.clone();

        // wallet_file.push(WALLET_CONFIGURATION);
        // If the wallet configuration already exists, then load it
        match wallet_file.exists() {
            true => {
                let mut iw = Wallet::try_from_slice(&fs::read(wallet_file.clone())?)?;
                iw.root_path = root_path.clone();
                iw.full_path = wallet_file;
                // Iterate through names loading each into keys
                iw.keys = iw
                    .prefixes
                    .iter()
                    .map(|kn| Wallet::load_key(iw.root_path.clone(), kn).unwrap())
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
    let wallet_path = location.clone();
    match wallet_path.exists() {
        true => {
            let mut wpath = wallet_path.clone();
            wpath.push(WALLET_CONFIGURATION);
            if wpath.exists() {
                Wallet::read_from_file(wpath.to_path_buf())
            } else {
                Wallet::new(wallet_path.to_path_buf())
            }
        }
        false => Wallet::new(wallet_path.to_path_buf()),
    }
}

/// Print to json string
pub fn to_json(title: &str, event: &EventMessage<SaidEvent<Event>>) {
    print!("{title}\n{}\n", serde_json::to_string(event).unwrap());
}

#[cfg(test)]
mod wallet_tests {

    use hbkr_rs::key_manage::{KeySet, Privatekey};

    use super::{load_wallet_from, Wallet};
    use crate::{
        errors::{SolDidError, SolDidResult},
        pkey_wrap::PastaKeySet,
        wallet::chain_event::KeyBlock,
    };
    use std::{env, fs, path::Path};

    /// Test wallet core path
    const TEST_WALLET_LOCATION: &str = "/.solwall_test";
    // Generate a test wallet
    fn build_test_wallet() -> SolDidResult<Wallet> {
        let location = match env::var("HOME") {
            Ok(val) => val + TEST_WALLET_LOCATION,
            Err(_) => return Err(SolDidError::HomeNotFoundError),
        };
        let wpath = Path::new(&location).to_path_buf();
        load_wallet_from(&wpath)
    }

    // remove a test wallet
    fn remove_test_wallet(wallet: Wallet) -> SolDidResult<()> {
        fs::remove_dir_all(wallet.full_path().parent().unwrap())?;
        Ok(())
    }

    //     use super::*;
    //     use crate::{pkey_wrap::PastaKeySet, wallet::chain_event::KeyBlock};

    #[test]
    /// Test wallet simple creation
    fn test_base_wallet_create_pass() -> SolDidResult<()> {
        let wallet = build_test_wallet()?;
        assert!(wallet.prefixes.is_empty());
        remove_test_wallet(wallet)?;
        Ok(())
    }

    #[test]
    /// Test wallet simple load
    fn test_base_load_existing_pass() -> SolDidResult<()> {
        let _ = build_test_wallet()?;
        let wallet = build_test_wallet()?;
        assert!(wallet.prefixes.is_empty());
        remove_test_wallet(wallet)?;
        Ok(())
    }

    #[test]
    /// Test an inception event
    fn test_inception_pasta_keys_pass() -> SolDidResult<()> {
        let mut wallet = build_test_wallet()?;
        assert!(wallet.prefixes.is_empty());
        assert!(wallet.keys.is_empty());
        let count = 2i8;
        let threshold = 1i8;
        let kset1 = PastaKeySet::new_for(count);
        let keys_name = "Alice".to_string();
        let (signature, prefix, digest) = wallet.new_did(&keys_name, &kset1, threshold, None)?;
        assert_eq!("sol_did_signature".to_string(), signature);
        assert!(!digest.is_empty());
        let k = wallet.keys_for_prefix(&prefix)?;
        assert_eq!(prefix, *k.prefix());
        let wallet = build_test_wallet()?;
        assert_eq!(wallet.prefixes.len(), 1);
        assert_eq!(wallet.keys.len(), 1);
        let k = wallet.keys_for_name(&keys_name);
        assert!(k.is_ok());
        remove_test_wallet(wallet)?;
        Ok(())
    }

    #[test]
    /// Test keys finders
    fn test_keys_finder_pass() -> SolDidResult<()> {
        let mut wallet = build_test_wallet()?;
        assert!(wallet.prefixes.is_empty());
        assert!(wallet.keys.is_empty());
        let count = 2i8;
        let threshold = 1i8;
        let kset1 = PastaKeySet::new_for(count);
        let keys_name = "Alice".to_string();
        let (_signature, prefix, _digest) = wallet.new_did(&keys_name, &kset1, threshold, None)?;
        let k = wallet.keys_for_prefix(&prefix)?;
        assert_eq!(prefix, *k.prefix());
        let k = wallet.keys_for_name(&keys_name)?;
        assert_eq!(keys_name, *k.name());
        remove_test_wallet(wallet)?;
        Ok(())
    }
    #[test]
    /// Test keys finders
    fn test_keys_finder_fail() -> SolDidResult<()> {
        let wallet = build_test_wallet()?;
        assert!(wallet.prefixes.is_empty());
        assert!(wallet.keys.is_empty());
        let keys_name = "Franks First".to_string();
        assert!(wallet.keys_for_name(&keys_name).is_err());
        assert!(wallet.keys_for_prefix(&keys_name).is_err());
        remove_test_wallet(wallet)?;
        Ok(())
    }

    #[test]
    /// Test rotation event to default keys
    fn test_rotation_pasta_keys_pass() -> SolDidResult<()> {
        let mut wallet = build_test_wallet()?;
        assert!(wallet.prefixes.is_empty());
        let count = 2i8;
        let threshold = 1i8;
        let kset1 = PastaKeySet::new_for(count);
        let keys_name = "Franks First".to_string();
        let (_signature, _prefix, _digest) = wallet.new_did(&keys_name, &kset1, threshold, None)?;
        let wallet = build_test_wallet()?;
        assert_eq!(wallet.prefixes.len(), 1);
        // Target prefix we want to rotation
        let new_first = wallet.keys.first().unwrap().clone();
        let prefix = new_first.prefix().to_string();
        // Rotate
        let mut wallet = build_test_wallet()?;
        let mut barren_ks = PastaKeySet::new_empty();
        let _ = wallet.rotate_did_with_name(keys_name, &mut barren_ks, None, None, None)?;
        // Observe
        let rot_keys = wallet.keys.first().unwrap();
        let rot_prefix = rot_keys.prefix();
        assert_eq!(*rot_prefix, prefix);
        remove_test_wallet(wallet)?;
        Ok(())
    }

    #[test]
    /// Test rotation to different keys than default
    fn test_rotation_to_different_pasta_keys_pass() -> SolDidResult<()> {
        let mut wallet = build_test_wallet()?;
        assert!(wallet.prefixes.is_empty());
        let count = 2i8;
        let threshold = 1i8;
        let kset1 = PastaKeySet::new_for(count);
        let keys_name = "Franks First".to_string();
        let (_signature, _prefix, _digest) = wallet.new_did(&keys_name, &kset1, threshold, None)?;
        let new_first = wallet.keys.first().unwrap().prefix().to_string();
        assert_eq!(wallet.keys_for_prefix(&new_first)?.chain_event_len(), 1);
        // Rotate
        let mut wallet = build_test_wallet()?;
        let mut barren_ks = PastaKeySet::new_empty();
        let kset2 = PastaKeySet::new_for(count);
        let new_next_set = kset2.current_private_keys();
        let _ = wallet.rotate_did_with_name(
            keys_name,
            &mut barren_ks,
            Some(new_next_set.clone()),
            None,
            None,
        )?;
        let chain_events = wallet.keys_for_prefix(&new_first)?.chain_events();
        assert_eq!(chain_events.len(), 2);
        let next_privates = chain_events
            .last()
            .unwrap()
            .get_keys_as_private_for(KeyBlock::NEXT)?;
        assert_eq!(new_next_set, next_privates);
        remove_test_wallet(wallet)?;
        Ok(())
    }

    #[test]
    /// Rotate to empty vector of next keeys fails
    /// as this is a decommission event
    fn test_rotate_to_empty_vector_fail() -> SolDidResult<()> {
        let mut wallet = build_test_wallet()?;
        assert!(wallet.prefixes.is_empty());
        let count = 2i8;
        let threshold = 1i8;
        let kset1 = PastaKeySet::new_for(count);
        let keys_name = "Franks First".to_string();
        let (_signature, _prefix, _digest) = wallet.new_did(&keys_name, &kset1, threshold, None)?;
        let new_first = wallet.keys.first().unwrap().prefix().to_string();
        assert_eq!(wallet.keys_for_prefix(&new_first)?.chain_event_len(), 1);
        // Rotate to empty
        let mut wallet = build_test_wallet()?;
        let mut barren_ks = PastaKeySet::new_empty();
        let new_next_set = Vec::<Privatekey>::new();
        let result = wallet.rotate_did_with_prefix(
            new_first.clone(),
            &mut barren_ks,
            Some(new_next_set.clone()),
            None,
            None,
        );
        assert!(result.is_err());
        remove_test_wallet(wallet)?;
        Ok(())
    }

    #[test]
    fn test_decommission_pass() -> SolDidResult<()> {
        let mut wallet = build_test_wallet()?;
        assert!(wallet.prefixes.is_empty());
        let count = 2i8;
        let threshold = 1i8;
        let kset1 = PastaKeySet::new_for(count);
        let keys_name = "Franks First".to_string();
        let (_signature, _prefix, _digest) = wallet.new_did(&keys_name, &kset1, threshold, None)?;
        let new_first = wallet.keys.first().unwrap().prefix().to_string();
        assert_eq!(wallet.keys_for_prefix(&new_first)?.chain_event_len(), 1);
        // Decommission keys
        let mut wallet = build_test_wallet()?;
        let mut barren_ks = PastaKeySet::new_empty();
        let result = wallet.decommission_did_with_prefix(new_first.clone(), &mut barren_ks, None);
        assert!(result.is_ok());
        remove_test_wallet(wallet)?;
        Ok(())
    }
}

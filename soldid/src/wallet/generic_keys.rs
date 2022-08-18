//! Generic Keys for Wallet

use borsh::{BorshDeserialize, BorshSerialize};

use crate::{
    chain_trait::Chain,
    errors::{SolDidError, SolDidResult},
};

use super::{
    chain_event::{ChainEvent, ChainEventType, KeyBlock},
    wallet_enums::{KeyState, KeyType},
    KEYS_CONFIGURATION,
};
use hbkr_rs::{
    inception,
    key_manage::{KeySet, PrivKey, Privatekey, Publickey},
    rotation,
    said::SelfAddressingPrefix,
    Prefix,
};
use std::{fs, io::Write, path::PathBuf, str::FromStr};

/// Keys define a named collection of public and private keys
/// represented as strings
#[derive(BorshDeserialize, BorshSerialize, Debug, Default)]
pub struct Keys {
    #[borsh_skip]
    dirty: bool,
    name: String,
    prefix: String,
    account: Publickey,
    threshold: i8,
    chain_events: Vec<ChainEvent>,
}

impl Keys {
    /// Get the keys chain events
    pub fn chain_events(&self) -> &Vec<ChainEvent> {
        &self.chain_events
    }
    /// Get number of events in chain VDR
    pub fn chain_event_len(&self) -> usize {
        self.chain_events().len()
    }
    /// Get the prefix
    pub fn prefix(&self) -> &String {
        &self.prefix
    }

    /// Get the key descriptive name
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Get the account key
    pub fn account(&self) -> &Publickey {
        &self.account
    }
    /// Accepts a native keyset this has been incepted
    /// distributes current (Incepted) and next (NextRotation) keys
    /// and stores the chain event initiating this function call
    pub fn incept_keys(
        name: &String,
        chain: Option<&dyn Chain>,
        key_set: &dyn KeySet,
        threshold: i8,
    ) -> SolDidResult<(Self, String, String, Vec<u8>)> {
        // Create an inception event
        let icp_event = inception(key_set, threshold as u64)?;
        let prefix = icp_event.event.get_prefix().to_str();
        // Optionally store on chain
        let (signature, account) = match chain {
            Some(chain) => chain.inception_inst(key_set, &icp_event)?,
            None => ("sol_did_signature".to_string(), Publickey::default()),
        };

        // Covert Type
        let set_type = KeyType::from(key_set.key_type());
        // Setup the chain event
        let mut chain_event = ChainEvent::from(&icp_event);

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
            .insert(KeyBlock::CURRENT, keysets_current);
        chain_event.keysets.insert(KeyBlock::NEXT, keysets_next);
        // Create a event store and push chain_event
        let mut chain_vec = Vec::<ChainEvent>::new();
        chain_vec.push(chain_event);
        // Return Self
        Ok((
            Keys {
                dirty: true,
                name: name.to_string(),
                prefix: prefix.clone(),
                account,
                threshold,
                chain_events: chain_vec,
            },
            signature,
            prefix,
            icp_event.get_digest().digest,
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
    pub fn rotate_keys(
        &mut self,
        barren_ks: &mut dyn KeySet,
        new_next_set: Option<Vec<Privatekey>>,
        threshold: Option<u64>,
        chain: Option<&dyn Chain>,
    ) -> SolDidResult<(String, Vec<u8>)> {
        // Validate state
        if self.chain_events.len() == 0 {
            Err(SolDidError::RotationIncoherence)
        } else {
            let new_next_clone = new_next_set.clone();
            if new_next_set.is_some() && new_next_set.unwrap().len() == 0 {
                Err(SolDidError::RotationToEmptyError)
            } else {
                // Validate ability to rotate
                let last_event = self.chain_events.last().unwrap();
                if !ChainEventType::can_rotate(last_event.event_type) {
                    return Err(SolDidError::RotationIncompatible);
                }
                // Re-hydrate the keystate
                let last_current = last_event.get_keys_as_strings_for(KeyBlock::CURRENT)?;
                let last_next = last_event.get_keys_as_strings_for(KeyBlock::NEXT)?;
                barren_ks.from(last_current.clone(), last_next);
                // Default rotation of keys should create equivalent count of keysets for next
                let (ncurr, nnext) = barren_ks.rotate(new_next_clone);
                // Rotate event
                let rot_event = rotation(
                    &self.prefix,
                    &last_event.km_digest,
                    last_event.km_sn + 1,
                    barren_ks,
                    match threshold {
                        Some(t) => {
                            self.threshold = t as i8;
                            t
                        }
                        None => self.threshold as u64,
                    },
                )?;
                // Optionally store on chain
                let signature = match chain {
                    Some(chain) => {
                        let incp_ce = self.chain_events.first().unwrap();
                        let incp_digest = SelfAddressingPrefix::from_str(&incp_ce.km_digest)?;
                        chain.rotation_inst(&incp_digest.digest, barren_ks, &rot_event)?
                    }
                    None => "sol_did_signature".to_string(),
                };
                // Create the event keysets
                let keytype = KeyType::from(barren_ks.key_type());
                // Create the chain event
                let mut chain_event = ChainEvent::from(&rot_event);
                chain_event.km_keytype = keytype;
                chain_event.did_signature = signature.clone();
                // Build the key state map
                chain_event.keysets.insert(
                    KeyBlock::CURRENT,
                    ncurr
                        .iter()
                        .map(|k| Key::new(KeyState::Rotated, keytype, &k.as_base58_string()))
                        .collect::<Vec<Key>>(),
                );
                chain_event.keysets.insert(
                    KeyBlock::NEXT,
                    nnext
                        .iter()
                        .map(|k| Key::new(KeyState::NextRotation, keytype, &k.as_base58_string()))
                        .collect::<Vec<Key>>(),
                );
                chain_event.keysets.insert(
                    KeyBlock::PAST,
                    last_current
                        .iter()
                        .map(|s| Key::new(KeyState::RotatedOut, keytype, s))
                        .collect::<Vec<Key>>(),
                );
                self.chain_events.push(chain_event);
                self.dirty = true;
                Ok((signature, rot_event.get_digest().digest))
            }
        }
    }

    /// Decommission this key set
    pub fn decommission_keys(
        &mut self,
        barren_ks: &mut dyn KeySet,
        chain: Option<&dyn Chain>,
    ) -> SolDidResult<(String, Vec<u8>)> {
        if self.chain_events.len() == 0 {
            Err(SolDidError::RotationIncoherence)
        } else {
            let last_event = self.chain_events.last().unwrap();
            if !ChainEventType::can_rotate(last_event.event_type) {
                return Err(SolDidError::RotationIncompatible);
            }
            // Rotate event with empty keyset
            // TODO: Check that barren is just that
            let rot_event = rotation(
                &self.prefix,
                &last_event.km_digest,
                last_event.km_sn + 1,
                barren_ks,
                0,
            )?;
            // Optionally store on chain
            let signature = match chain {
                Some(chain) => {
                    let incp_ce = self.chain_events.first().unwrap();
                    let incp_digest = SelfAddressingPrefix::from_str(&incp_ce.km_digest)?;
                    chain.decommission_inst(&incp_digest.digest, &rot_event)?
                }
                None => "sol_did_signature".to_string(),
            };
            let keytype = KeyType::from(barren_ks.key_type());
            let last_current = last_event.get_keys_as_strings_for(KeyBlock::CURRENT)?;
            let last_next = last_event.get_keys_as_strings_for(KeyBlock::NEXT)?;

            let mut event_past = last_current
                .iter()
                .map(|s| Key::new(KeyState::Decommisioined, keytype, s))
                .collect::<Vec<Key>>();
            event_past.extend(
                last_next
                    .iter()
                    .map(|s| Key::new(KeyState::Decommisioined, keytype, s))
                    .collect::<Vec<Key>>(),
            );

            // Set decommissioned chain event
            let mut chain_event = ChainEvent::from(&rot_event);
            chain_event.km_keytype = keytype;
            chain_event.did_signature = signature.clone();
            chain_event.event_type = ChainEventType::Decommissioned;
            // Capture key states
            chain_event.keysets.insert(KeyBlock::PAST, event_past);
            chain_event
                .keysets
                .insert(KeyBlock::CURRENT, Vec::<Key>::new());
            chain_event
                .keysets
                .insert(KeyBlock::NEXT, Vec::<Key>::new());
            self.chain_events.push(chain_event);
            self.dirty = true;
            Ok((signature, rot_event.get_digest().digest))
        }
    }

    /// Read keys for wallet from path
    pub fn load(loc: &mut PathBuf) -> SolDidResult<Keys> {
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
    pub fn write(&mut self, loc: &PathBuf) -> SolDidResult<()> {
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
    pub fn key(&self) -> String {
        self.key.clone()
    }
}

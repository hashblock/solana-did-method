//! Various enum types for wallet and keys

use borsh::{BorshDeserialize, BorshSerialize};
use hbkr_rs::basic::Basic;
#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Copy, Hash, Eq, PartialEq, PartialOrd)]
pub enum KeyState {
    PreInception,
    Incepted,
    NextRotation,
    Rotated,
    RotatedOut,
    Decommisioined,
    Revoked,
}
#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, Copy, Hash, Eq, PartialEq, PartialOrd)]
pub enum KeyType {
    ED25519,
    PASTA,
}

impl Default for KeyType {
    fn default() -> Self {
        KeyType::ED25519
    }
}

impl From<Basic> for KeyType {
    fn from(basic_type: Basic) -> Self {
        match basic_type {
            Basic::ED25519 => KeyType::ED25519,
            Basic::PASTA => KeyType::PASTA,
        }
    }
}

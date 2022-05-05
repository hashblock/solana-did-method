use std::fmt;

use pasta_curves::group::ff::Field;
use pasta_curves::group::ff::PrimeField;
use pasta_curves::group::Group;
use pasta_curves::group::GroupEncoding;
use pasta_curves::{pallas::Scalar, Ep};
use rand::rngs::OsRng;
use rand::CryptoRng;
use rand::RngCore;

/// Fqp is a pallas::Scalar
pub type Fqp = Scalar;

#[derive(Clone, Copy)]
pub struct PastaKeyPair(pub Fqp);
impl PastaKeyPair {
    /// Constructs a new, random `Keypair` using a caller-proveded RNG
    fn generate<R>(csprng: &mut R) -> Self
    where
        R: CryptoRng + RngCore,
    {
        Self(Fqp::random(csprng))
    }

    /// Constructs a new, random `Keypair` using `OsRng`
    pub fn new() -> Self {
        let mut rng = OsRng::default();
        Self::generate(&mut rng)
    }

    pub fn secret(&self) -> Fqp {
        self.0
    }

    pub fn public_key(&self) -> PastaPublicKey {
        PastaPublicKey(Ep::generator() * self.0)
    }
}

/// For debugging but we need to get to the 64 bytes?
impl fmt::Debug for PastaKeyPair {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tmp = self.0.to_repr();
        write!(f, "{}", bs58::encode(tmp).into_string())?;
        Ok(())
    }
}
#[derive(Debug, Clone, Copy)]
pub struct PastaPublicKey(pub Ep);

impl PastaPublicKey {
    pub fn new_random() -> Self {
        let mut rng = OsRng::default();
        PastaKeyPair::generate(&mut rng).public_key()
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    pub fn to_base58_string(&self) -> String {
        bs58::encode(self.to_bytes()).into_string()
    }

    pub fn from_base58_string(s: &str) -> Self {
        let mut bff = [0u8; 32];
        let _ = bs58::decode(s).into(&mut bff).unwrap();
        Self(Ep::from_bytes(&bff).unwrap())
    }
}

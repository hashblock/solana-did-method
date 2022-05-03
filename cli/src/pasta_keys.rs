use pasta_curves::group::ff::Field;
use pasta_curves::group::Group;
use pasta_curves::group::GroupEncoding;
use pasta_curves::{pallas::Scalar, Ep};
use rand::RngCore;

/// Fqp is a pallas::Scalar
pub type Fqp = Scalar;

#[derive(Debug, Clone, Copy)]
pub struct PastaSecretKey(pub Fqp);

impl PastaSecretKey {
    pub fn random(mut rng: impl RngCore) -> Self {
        Self(Fqp::random(&mut rng))
    }
}
#[derive(Debug, Clone, Copy)]
pub struct PastaPublicKey(pub Ep);

impl PastaPublicKey {
    pub fn from_secret_key(s: &PastaSecretKey) -> Self {
        Self(Ep::generator() * s.0)
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

#[derive(Debug, Clone, Copy)]
pub struct PastaKeyPair;

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Serialize, Serializer};
use std::fmt;

use crate::{Digest, PublicIdentity, VertexError, VertexResult, canonical_bytes};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SignatureBytes([u8; 64]);

impl SignatureBytes {
    pub const fn new(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }

    pub const fn bytes(self) -> [u8; 64] {
        self.0
    }
}

pub fn signing_digest<T: Serialize>(domain: &str, payload: &T) -> VertexResult<Digest> {
    let bytes = canonical_bytes(payload)?;
    Ok(Digest::from_parts(domain, &[&bytes]))
}

pub fn verify_signature<T: Serialize>(
    identity: PublicIdentity,
    signature: SignatureBytes,
    domain: &str,
    payload: &T,
) -> VertexResult<()> {
    identity.verify_consistency()?;
    let key = VerifyingKey::from_bytes(&identity.public_key)
        .map_err(|_| VertexError::InvalidPublicKey)?;
    let sig = Signature::from_bytes(&signature.bytes());
    let digest = signing_digest(domain, payload)?;
    key.verify(&digest.bytes(), &sig)
        .map_err(|_| VertexError::InvalidSignature)
}

impl Serialize for SignatureBytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(self.0))
    }
}

impl fmt::Display for SignatureBytes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", hex::encode(self.0))
    }
}

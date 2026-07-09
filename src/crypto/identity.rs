use ed25519_dalek::{Signer, SigningKey};
use serde::Serialize;

use crate::{
    AccountId, Digest, SignatureBytes, VertexError, VertexResult, crypto::signature::signing_digest,
};

#[derive(Clone)]
pub struct KeyPair {
    signing_key: SigningKey,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PublicIdentity {
    pub account: AccountId,
    pub public_key: [u8; 32],
}

impl KeyPair {
    pub fn from_seed(seed: [u8; 32]) -> Self {
        Self {
            signing_key: SigningKey::from_bytes(&seed),
        }
    }

    pub fn public_identity(&self) -> PublicIdentity {
        let public_key = self.signing_key.verifying_key().to_bytes();
        PublicIdentity {
            account: AccountId::from_public_key(public_key),
            public_key,
        }
    }

    pub fn sign<T: Serialize>(&self, domain: &str, payload: &T) -> VertexResult<SignatureBytes> {
        let digest: Digest = signing_digest(domain, payload)?;
        Ok(SignatureBytes::new(
            self.signing_key.sign(&digest.bytes()).to_bytes(),
        ))
    }
}

impl PublicIdentity {
    pub fn verify_consistency(self) -> VertexResult<()> {
        let expected = AccountId::from_public_key(self.public_key);
        if expected != self.account {
            return Err(VertexError::IdentityMismatch(self.account));
        }
        Ok(())
    }
}

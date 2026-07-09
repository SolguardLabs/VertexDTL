use serde::{Serialize, Serializer};
use std::fmt;

use crate::{VertexResult, canonical_bytes};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Digest([u8; 32]);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct AccountId([u8; 32]);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct AssetId([u8; 32]);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TicketId([u8; 32]);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TxId([u8; 32]);

impl Digest {
    pub fn from_parts(domain: &str, parts: &[&[u8]]) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(domain.as_bytes());
        hasher.update(&[0]);
        for part in parts {
            hasher.update(&(part.len() as u64).to_be_bytes());
            hasher.update(part);
        }
        Self(*hasher.finalize().as_bytes())
    }

    pub fn from_serializable<T: Serialize>(domain: &str, value: &T) -> VertexResult<Self> {
        let bytes = canonical_bytes(value)?;
        Ok(Self::from_parts(domain, &[&bytes]))
    }

    pub const fn zero() -> Self {
        Self([0u8; 32])
    }

    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

impl AccountId {
    pub fn from_public_key(public_key: [u8; 32]) -> Self {
        Self(Digest::from_parts("vertex-account-v1", &[&public_key]).bytes())
    }

    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl AssetId {
    pub fn native() -> Self {
        Self(Digest::from_parts("vertex-native-asset-v1", &[b"vtx"]).bytes())
    }
}

impl TicketId {
    pub fn derive(
        network_id: u32,
        payer: AccountId,
        beneficiary: AccountId,
        nonce: u64,
        salt: Digest,
    ) -> Self {
        Self(
            Digest::from_parts(
                "vertex-ticket-id-v2",
                &[
                    &network_id.to_be_bytes(),
                    &payer.0,
                    &beneficiary.0,
                    &nonce.to_be_bytes(),
                    &salt.bytes(),
                ],
            )
            .bytes(),
        )
    }
}

impl TxId {
    pub fn from_serializable<T: Serialize>(domain: &str, value: &T) -> VertexResult<Self> {
        Ok(Self(Digest::from_serializable(domain, value)?.bytes()))
    }
}

macro_rules! impl_hex {
    ($name:ident) => {
        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&hex::encode(self.0))
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "{}", hex::encode(self.0))
            }
        }
    };
}

impl_hex!(Digest);
impl_hex!(AccountId);
impl_hex!(AssetId);
impl_hex!(TicketId);
impl_hex!(TxId);

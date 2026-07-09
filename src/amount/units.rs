use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{VertexError, VertexResult};

pub const MAX_LEDGER_UNITS: u128 = 100_000_000_000_000;
pub const MAX_BPS: u16 = 10_000;

#[derive(
    Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Amount(u128);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Bps(u16);

impl Amount {
    pub fn new(value: u128) -> VertexResult<Self> {
        if value > MAX_LEDGER_UNITS {
            return Err(VertexError::AmountOutOfRange {
                value,
                max: MAX_LEDGER_UNITS,
            });
        }
        Ok(Self(value))
    }

    pub const fn zero() -> Self {
        Self(0)
    }

    pub const fn units(self) -> u128 {
        self.0
    }

    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    pub fn checked_add(self, rhs: Self) -> VertexResult<Self> {
        let value = self
            .0
            .checked_add(rhs.0)
            .ok_or(VertexError::AmountOverflow)?;
        Self::new(value)
    }

    pub fn checked_sub(self, rhs: Self) -> VertexResult<Self> {
        let value = self
            .0
            .checked_sub(rhs.0)
            .ok_or(VertexError::AmountUnderflow)?;
        Self::new(value)
    }

    pub fn checked_mul_bps(self, bps: Bps) -> VertexResult<Self> {
        let scaled = self
            .0
            .checked_mul(u128::from(bps.value()))
            .ok_or(VertexError::AmountOverflow)?;
        Self::new(scaled / u128::from(MAX_BPS))
    }
}

impl Bps {
    pub fn new(value: u16) -> VertexResult<Self> {
        if value > MAX_BPS {
            return Err(VertexError::Policy("basis points exceed 100%".to_owned()));
        }
        Ok(Self(value))
    }

    pub const fn value(self) -> u16 {
        self.0
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl fmt::Display for Bps {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

use serde::Serialize;

use crate::{Amount, Bps, VertexError, VertexResult};

const BPS_SCALE: u128 = 10_000;
const CUBIC_SCALE: u128 = BPS_SCALE * BPS_SCALE * BPS_SCALE;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RouteMarket {
    pub capacity: Amount,
    pub settled: Amount,
    pub slope_bps: Bps,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CongestionQuote {
    pub settled_before: Amount,
    pub projected_settled: Amount,
    pub utilization_bps: u32,
    pub surcharge: Amount,
}

impl RouteMarket {
    pub fn new(capacity: Amount, slope_bps: Bps) -> VertexResult<Self> {
        if capacity.is_zero() {
            return Err(VertexError::ZeroAmount);
        }
        Ok(Self {
            capacity,
            settled: Amount::zero(),
            slope_bps,
        })
    }

    pub fn quote(self, gross: Amount) -> VertexResult<CongestionQuote> {
        if gross.is_zero() {
            return Err(VertexError::ZeroAmount);
        }
        let projected = self.settled.checked_add(gross)?;
        if projected > self.capacity {
            return Err(VertexError::Policy("route capacity exceeded".to_owned()));
        }
        let utilization = projected
            .units()
            .checked_mul(BPS_SCALE)
            .ok_or(VertexError::AmountOverflow)?
            / self.capacity.units();
        let numerator = gross
            .units()
            .checked_mul(u128::from(self.slope_bps.value()))
            .and_then(|value| value.checked_mul(utilization))
            .and_then(|value| value.checked_mul(utilization))
            .ok_or(VertexError::AmountOverflow)?;
        Ok(CongestionQuote {
            settled_before: self.settled,
            projected_settled: projected,
            utilization_bps: utilization as u32,
            surcharge: Amount::new(numerator / CUBIC_SCALE)?,
        })
    }
}

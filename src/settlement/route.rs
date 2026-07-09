use serde::Serialize;

use crate::{AccountId, Amount, Bps, Digest, VertexError, VertexResult};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RoutePolicy {
    pub venue: Digest,
    pub max_charge_bps: Bps,
    pub settlement_epoch: u64,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RoutePlan {
    pub operator: AccountId,
    pub fee_recipient: AccountId,
    pub operator_fee: Amount,
    pub rebate_recipient: Option<AccountId>,
    pub rebate_amount: Amount,
    pub liquidity_memo: Digest,
    pub routing_epoch: u64,
}

impl RoutePolicy {
    pub fn new(venue: Digest, max_charge_bps: Bps, settlement_epoch: u64) -> Self {
        Self {
            venue,
            max_charge_bps,
            settlement_epoch,
        }
    }
}

impl RoutePlan {
    pub fn direct(operator: AccountId, epoch: u64) -> Self {
        Self {
            operator,
            fee_recipient: operator,
            operator_fee: Amount::zero(),
            rebate_recipient: None,
            rebate_amount: Amount::zero(),
            liquidity_memo: Digest::from_parts("vertex-direct-route-v1", &[&epoch.to_be_bytes()]),
            routing_epoch: epoch,
        }
    }

    pub fn charges(self) -> VertexResult<Amount> {
        self.operator_fee.checked_add(self.rebate_amount)
    }

    pub fn route_digest(self) -> VertexResult<Digest> {
        Digest::from_serializable("vertex-route-plan-v3", &self)
    }

    pub fn validate(self, policy: RoutePolicy, amount: Amount) -> VertexResult<()> {
        let charges = self.charges()?;
        if charges > amount {
            return Err(VertexError::Policy(
                "route charges exceed ticket amount".to_owned(),
            ));
        }

        if self.routing_epoch >= policy.settlement_epoch {
            let allowed = amount.checked_mul_bps(policy.max_charge_bps)?;
            if charges > allowed {
                return Err(VertexError::Policy(
                    "route charges exceed policy ceiling".to_owned(),
                ));
            }
        }

        Ok(())
    }
}

use serde::Serialize;

use crate::{Amount, RoutePlan, VertexResult};

/// Read-only distribution implied by a concrete settlement route.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RouteAllocation {
    pub gross: Amount,
    pub beneficiary: Amount,
    pub operator_fee: Amount,
    pub congestion_surcharge: Amount,
    pub rebate: Amount,
}

impl RouteAllocation {
    pub fn from_plan(gross: Amount, plan: RoutePlan) -> VertexResult<Self> {
        Self::with_surcharge(gross, plan, Amount::zero())
    }

    pub fn with_surcharge(gross: Amount, plan: RoutePlan, surcharge: Amount) -> VertexResult<Self> {
        let charges = plan.charges()?.checked_add(surcharge)?;
        let beneficiary = gross.checked_sub(charges)?;
        Ok(Self {
            gross,
            beneficiary,
            operator_fee: plan.operator_fee,
            congestion_surcharge: surcharge,
            rebate: plan.rebate_amount,
        })
    }

    pub fn recomposed_total(self) -> VertexResult<Amount> {
        self.beneficiary
            .checked_add(self.operator_fee)?
            .checked_add(self.congestion_surcharge)?
            .checked_add(self.rebate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Digest, KeyPair};

    #[test]
    fn quote_recomposes_all_destinations() {
        let operator = KeyPair::from_seed([3; 32]).public_identity().account;
        let mut plan = RoutePlan::direct(operator, 20);
        plan.operator_fee = Amount::new(9).unwrap();
        plan.rebate_amount = Amount::new(3).unwrap();
        plan.liquidity_memo = Digest::zero();
        let quote = RouteAllocation::from_plan(Amount::new(1_000).unwrap(), plan).unwrap();
        assert_eq!(quote.beneficiary.units(), 988);
        assert_eq!(quote.recomposed_total().unwrap().units(), 1_000);
    }

    #[test]
    fn quote_rejects_charges_above_gross() {
        let operator = KeyPair::from_seed([4; 32]).public_identity().account;
        let mut plan = RoutePlan::direct(operator, 20);
        plan.operator_fee = Amount::new(101).unwrap();
        assert!(RouteAllocation::from_plan(Amount::new(100).unwrap(), plan).is_err());
    }
}

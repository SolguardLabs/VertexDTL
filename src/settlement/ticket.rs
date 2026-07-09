use serde::Serialize;

use crate::{
    AccountId, Amount, AssetId, Digest, KeyPair, PublicIdentity, RoutePlan, RoutePolicy,
    SignatureBytes, TicketId, TxId, VertexError, VertexResult, verify_signature,
};

pub const TICKET_DOMAIN: &str = "vertex-ticket-open-v3";

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TicketTerms {
    pub network_id: u32,
    pub ticket_id: TicketId,
    pub payer: AccountId,
    pub beneficiary: AccountId,
    pub asset: AssetId,
    pub amount: Amount,
    pub payer_nonce: u64,
    pub expires_at_epoch: u64,
    pub route_policy: RoutePolicy,
    pub salt: Digest,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TicketAuthorizationView {
    network_id: u32,
    ticket_id: TicketId,
    payer: AccountId,
    beneficiary: AccountId,
    asset: AssetId,
    amount: Amount,
    payer_nonce: u64,
    expires_at_epoch: u64,
    route_policy: RoutePolicy,
    salt: Digest,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SignedTicket {
    pub payer: PublicIdentity,
    pub terms: TicketTerms,
    pub route_plan: RoutePlan,
    pub signature: SignatureBytes,
}

impl TicketTerms {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        network_id: u32,
        payer: AccountId,
        beneficiary: AccountId,
        asset: AssetId,
        amount: Amount,
        payer_nonce: u64,
        expires_at_epoch: u64,
        route_policy: RoutePolicy,
        salt: Digest,
    ) -> VertexResult<Self> {
        if amount.is_zero() {
            return Err(VertexError::ZeroAmount);
        }

        let ticket_id = TicketId::derive(network_id, payer, beneficiary, payer_nonce, salt);
        Ok(Self {
            network_id,
            ticket_id,
            payer,
            beneficiary,
            asset,
            amount,
            payer_nonce,
            expires_at_epoch,
            route_policy,
            salt,
        })
    }

    pub fn authorization_view(self) -> TicketAuthorizationView {
        TicketAuthorizationView {
            network_id: self.network_id,
            ticket_id: self.ticket_id,
            payer: self.payer,
            beneficiary: self.beneficiary,
            asset: self.asset,
            amount: self.amount,
            payer_nonce: self.payer_nonce,
            expires_at_epoch: self.expires_at_epoch,
            route_policy: self.route_policy,
            salt: self.salt,
        }
    }
}

impl SignedTicket {
    pub fn sign(
        terms: TicketTerms,
        route_plan: RoutePlan,
        key_pair: &KeyPair,
    ) -> VertexResult<Self> {
        let payer = key_pair.public_identity();
        if payer.account != terms.payer {
            return Err(VertexError::UnauthorizedTicketSigner {
                expected: terms.payer,
                received: payer.account,
            });
        }

        let signature = key_pair.sign(TICKET_DOMAIN, &terms.authorization_view())?;
        Ok(Self {
            payer,
            terms,
            route_plan,
            signature,
        })
    }

    pub fn verify(&self) -> VertexResult<()> {
        if self.payer.account != self.terms.payer {
            return Err(VertexError::UnauthorizedTicketSigner {
                expected: self.terms.payer,
                received: self.payer.account,
            });
        }

        verify_signature(
            self.payer,
            self.signature,
            TICKET_DOMAIN,
            &self.terms.authorization_view(),
        )
    }

    pub fn tx_id(&self) -> VertexResult<TxId> {
        TxId::from_serializable("vertex-signed-ticket-v3", self)
    }
}

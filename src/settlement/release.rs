use serde::Serialize;

use crate::{
    AccountId, Digest, KeyPair, PublicIdentity, SignatureBytes, TicketId, TxId, VertexError,
    VertexResult, verify_signature,
};

pub const RELEASE_DOMAIN: &str = "vertex-ticket-release-v3";

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ReleaseRequest {
    pub network_id: u32,
    pub ticket_id: TicketId,
    pub beneficiary: AccountId,
    pub release_nonce: u64,
    pub observed_route_digest: Digest,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ReleaseAuthorizationView {
    network_id: u32,
    ticket_id: TicketId,
    beneficiary: AccountId,
    release_nonce: u64,
    observed_route_digest: Digest,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SignedRelease {
    pub signer: PublicIdentity,
    pub request: ReleaseRequest,
    pub signature: SignatureBytes,
}

impl ReleaseRequest {
    pub fn new(
        network_id: u32,
        ticket_id: TicketId,
        beneficiary: AccountId,
        release_nonce: u64,
        observed_route_digest: Digest,
    ) -> Self {
        Self {
            network_id,
            ticket_id,
            beneficiary,
            release_nonce,
            observed_route_digest,
        }
    }

    fn authorization_view(self) -> ReleaseAuthorizationView {
        ReleaseAuthorizationView {
            network_id: self.network_id,
            ticket_id: self.ticket_id,
            beneficiary: self.beneficiary,
            release_nonce: self.release_nonce,
            observed_route_digest: self.observed_route_digest,
        }
    }
}

impl SignedRelease {
    pub fn sign(request: ReleaseRequest, key_pair: &KeyPair) -> VertexResult<Self> {
        let signer = key_pair.public_identity();
        if signer.account != request.beneficiary {
            return Err(VertexError::UnauthorizedReleaseSigner {
                expected: request.beneficiary,
                received: signer.account,
            });
        }

        let signature = key_pair.sign(RELEASE_DOMAIN, &request.authorization_view())?;
        Ok(Self {
            signer,
            request,
            signature,
        })
    }

    pub fn verify(&self) -> VertexResult<()> {
        if self.signer.account != self.request.beneficiary {
            return Err(VertexError::UnauthorizedReleaseSigner {
                expected: self.request.beneficiary,
                received: self.signer.account,
            });
        }

        verify_signature(
            self.signer,
            self.signature,
            RELEASE_DOMAIN,
            &self.request.authorization_view(),
        )
    }

    pub fn tx_id(&self) -> VertexResult<TxId> {
        TxId::from_serializable("vertex-signed-release-v3", self)
    }
}

use serde::Serialize;

use crate::{AccountId, Amount, PublicIdentity};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AccountState {
    pub id: AccountId,
    pub identity: PublicIdentity,
    pub balance: Amount,
    pub next_ticket_nonce: u64,
    pub next_release_nonce: u64,
}

impl AccountState {
    pub fn new(identity: PublicIdentity) -> Self {
        Self {
            id: identity.account,
            identity,
            balance: Amount::zero(),
            next_ticket_nonce: 0,
            next_release_nonce: 0,
        }
    }
}

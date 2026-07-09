use thiserror::Error;

use crate::{AccountId, Amount, AssetId, Digest, TicketId, TxId};

pub type VertexResult<T> = Result<T, VertexError>;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum VertexError {
    #[error("amount overflow")]
    AmountOverflow,

    #[error("amount underflow")]
    AmountUnderflow,

    #[error("amount {value} exceeds ledger maximum {max}")]
    AmountOutOfRange { value: u128, max: u128 },

    #[error("zero amount")]
    ZeroAmount,

    #[error("invalid public key")]
    InvalidPublicKey,

    #[error("invalid signature")]
    InvalidSignature,

    #[error("serialization failed: {0}")]
    Serialization(String),

    #[error("identity mismatch for account {0}")]
    IdentityMismatch(AccountId),

    #[error("account not found: {0}")]
    AccountNotFound(AccountId),

    #[error("account already exists: {0}")]
    AccountAlreadyExists(AccountId),

    #[error("ticket not found: {0}")]
    TicketNotFound(TicketId),

    #[error("ticket already exists: {0}")]
    TicketAlreadyExists(TicketId),

    #[error("ticket already settled: {0}")]
    TicketSettled(TicketId),

    #[error("unauthorized ticket signer: expected {expected}, received {received}")]
    UnauthorizedTicketSigner {
        expected: AccountId,
        received: AccountId,
    },

    #[error("unauthorized release signer: expected {expected}, received {received}")]
    UnauthorizedReleaseSigner {
        expected: AccountId,
        received: AccountId,
    },

    #[error("nonce mismatch for {account}: expected {expected}, received {received}")]
    NonceMismatch {
        account: AccountId,
        expected: u64,
        received: u64,
    },

    #[error("nonce overflow")]
    NonceOverflow,

    #[error("duplicate transaction: {0}")]
    DuplicateTransaction(TxId),

    #[error("insufficient funds for {account}: available {available}, required {required}")]
    InsufficientFunds {
        account: AccountId,
        available: Amount,
        required: Amount,
    },

    #[error("asset mismatch: expected {expected}, received {received}")]
    AssetMismatch {
        expected: AssetId,
        received: AssetId,
    },

    #[error(
        "route digest mismatch for ticket {ticket_id}: expected {expected}, received {received}"
    )]
    RouteDigestMismatch {
        ticket_id: TicketId,
        expected: Digest,
        received: Digest,
    },

    #[error("policy violation: {0}")]
    Policy(String),

    #[error("conservation failure for {asset}: expected {expected}, observed {observed}")]
    Conservation {
        asset: AssetId,
        expected: Amount,
        observed: Amount,
    },
}

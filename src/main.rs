mod amount;
mod codec;
mod crypto;
mod error;
mod ids;
mod ledger;
mod runtime;
mod settlement;

pub use amount::{Amount, Bps};
pub use codec::canonical_bytes;
pub use crypto::{KeyPair, PublicIdentity, SignatureBytes, verify_signature};
pub use error::{VertexError, VertexResult};
pub use ids::{AccountId, AssetId, Digest, TicketId, TxId};
pub use ledger::{AccountState, JournalEntry, JournalOp, TicketState, VertexLedger};
pub use runtime::ScenarioReport;
pub use settlement::{
    ReleaseAuthorizationView, ReleaseRequest, RoutePlan, RoutePolicy, SignedRelease, SignedTicket,
    TicketAuthorizationView, TicketTerms,
};

fn main() {
    if let Err(error) = runtime::run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

use serde::Serialize;

use crate::{AccountId, Amount, TicketId, TxId};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub enum JournalOp {
    GenesisCredit {
        account: AccountId,
        amount: Amount,
    },
    TicketDebit {
        account: AccountId,
        ticket_id: TicketId,
        amount: Amount,
    },
    TicketSettlement {
        ticket_id: TicketId,
        beneficiary: AccountId,
        beneficiary_amount: Amount,
        operator: AccountId,
        operator_amount: Amount,
        rebate_recipient: AccountId,
        rebate_amount: Amount,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct JournalEntry {
    pub tx_id: TxId,
    pub op: JournalOp,
}

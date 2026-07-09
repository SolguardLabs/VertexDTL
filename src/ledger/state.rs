use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    AccountId, AccountState, Amount, AssetId, Digest, JournalEntry, JournalOp, PublicIdentity,
    SignedRelease, SignedTicket, TicketId, TxId, VertexError, VertexResult,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TicketState {
    pub id: TicketId,
    pub terms: crate::TicketTerms,
    pub route_plan: crate::RoutePlan,
    pub settled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct VertexLedger {
    network_id: u32,
    asset: AssetId,
    accounts: BTreeMap<AccountId, AccountState>,
    tickets: BTreeMap<TicketId, TicketState>,
    seen_transactions: BTreeSet<TxId>,
    total_supply: Amount,
    journal: Vec<JournalEntry>,
}

impl VertexLedger {
    pub fn new(network_id: u32, asset: AssetId) -> Self {
        Self {
            network_id,
            asset,
            accounts: BTreeMap::new(),
            tickets: BTreeMap::new(),
            seen_transactions: BTreeSet::new(),
            total_supply: Amount::zero(),
            journal: Vec::new(),
        }
    }

    pub fn network_id(&self) -> u32 {
        self.network_id
    }

    pub fn asset(&self) -> AssetId {
        self.asset
    }

    pub fn register_account(&mut self, identity: PublicIdentity) -> VertexResult<()> {
        identity.verify_consistency()?;
        if self.accounts.contains_key(&identity.account) {
            return Err(VertexError::AccountAlreadyExists(identity.account));
        }
        self.accounts
            .insert(identity.account, AccountState::new(identity));
        Ok(())
    }

    pub fn credit_genesis(&mut self, account: AccountId, amount: Amount) -> VertexResult<TxId> {
        self.ensure_account(account)?;
        let mut candidate = self.clone();
        candidate.credit(account, amount)?;
        candidate.total_supply = candidate.total_supply.checked_add(amount)?;
        let tx_id = TxId::from_serializable(
            "vertex-genesis-credit-v1",
            &(account, amount, candidate.journal.len()),
        )?;
        candidate.journal.push(JournalEntry {
            tx_id,
            op: JournalOp::GenesisCredit { account, amount },
        });
        candidate.verify_conservation()?;
        *self = candidate;
        Ok(tx_id)
    }

    pub fn balance_of(&self, account: AccountId) -> VertexResult<Amount> {
        Ok(self.ensure_account(account)?.balance)
    }

    pub fn ticket_nonce(&self, account: AccountId) -> VertexResult<u64> {
        Ok(self.ensure_account(account)?.next_ticket_nonce)
    }

    pub fn release_nonce(&self, account: AccountId) -> VertexResult<u64> {
        Ok(self.ensure_account(account)?.next_release_nonce)
    }

    pub fn total_supply(&self) -> Amount {
        self.total_supply
    }

    pub fn ticket(&self, ticket_id: TicketId) -> VertexResult<&TicketState> {
        self.tickets
            .get(&ticket_id)
            .ok_or(VertexError::TicketNotFound(ticket_id))
    }

    pub fn state_digest(&self) -> VertexResult<Digest> {
        Digest::from_serializable(
            "vertex-ledger-state-v3",
            &(
                self.network_id,
                self.asset,
                &self.accounts,
                &self.tickets,
                &self.seen_transactions,
                self.total_supply,
            ),
        )
    }

    pub fn open_ticket(&mut self, signed: &SignedTicket) -> VertexResult<TxId> {
        let mut candidate = self.clone();
        let tx_id = candidate.open_ticket_inner(signed)?;
        candidate.verify_conservation()?;
        *self = candidate;
        Ok(tx_id)
    }

    pub fn settle_ticket(&mut self, signed: &SignedRelease) -> VertexResult<TxId> {
        let mut candidate = self.clone();
        let tx_id = candidate.settle_ticket_inner(signed)?;
        candidate.verify_conservation()?;
        *self = candidate;
        Ok(tx_id)
    }

    fn open_ticket_inner(&mut self, signed: &SignedTicket) -> VertexResult<TxId> {
        signed.verify()?;
        let terms = signed.terms;
        let plan = signed.route_plan;

        if terms.network_id != self.network_id {
            return Err(VertexError::Policy("wrong network".to_owned()));
        }

        if terms.asset != self.asset {
            return Err(VertexError::AssetMismatch {
                expected: self.asset,
                received: terms.asset,
            });
        }

        if self.tickets.contains_key(&terms.ticket_id) {
            return Err(VertexError::TicketAlreadyExists(terms.ticket_id));
        }

        self.ensure_account(terms.beneficiary)?;
        self.ensure_account(plan.operator)?;
        self.ensure_account(plan.fee_recipient)?;
        if let Some(rebate_recipient) = plan.rebate_recipient {
            self.ensure_account(rebate_recipient)?;
        }

        plan.validate(terms.route_policy, terms.amount)?;

        let payer = self.ensure_account(terms.payer)?;
        if payer.next_ticket_nonce != terms.payer_nonce {
            return Err(VertexError::NonceMismatch {
                account: terms.payer,
                expected: payer.next_ticket_nonce,
                received: terms.payer_nonce,
            });
        }

        if payer.balance < terms.amount {
            return Err(VertexError::InsufficientFunds {
                account: terms.payer,
                available: payer.balance,
                required: terms.amount,
            });
        }

        let tx_id = signed.tx_id()?;
        if self.seen_transactions.contains(&tx_id) {
            return Err(VertexError::DuplicateTransaction(tx_id));
        }

        let next_balance = payer.balance.checked_sub(terms.amount)?;
        let payer_state = self
            .accounts
            .get_mut(&terms.payer)
            .ok_or(VertexError::AccountNotFound(terms.payer))?;
        payer_state.balance = next_balance;
        payer_state.next_ticket_nonce = payer_state
            .next_ticket_nonce
            .checked_add(1)
            .ok_or(VertexError::NonceOverflow)?;

        self.tickets.insert(
            terms.ticket_id,
            TicketState {
                id: terms.ticket_id,
                terms,
                route_plan: plan,
                settled: false,
            },
        );
        self.seen_transactions.insert(tx_id);
        self.journal.push(JournalEntry {
            tx_id,
            op: JournalOp::TicketDebit {
                account: terms.payer,
                ticket_id: terms.ticket_id,
                amount: terms.amount,
            },
        });

        Ok(tx_id)
    }

    fn settle_ticket_inner(&mut self, signed: &SignedRelease) -> VertexResult<TxId> {
        signed.verify()?;

        if signed.request.network_id != self.network_id {
            return Err(VertexError::Policy("wrong network".to_owned()));
        }

        let ticket = self.ticket(signed.request.ticket_id)?.clone();
        if ticket.settled {
            return Err(VertexError::TicketSettled(ticket.id));
        }

        if signed.signer.account != ticket.terms.beneficiary {
            return Err(VertexError::UnauthorizedReleaseSigner {
                expected: ticket.terms.beneficiary,
                received: signed.signer.account,
            });
        }

        let expected_route_digest = ticket.route_plan.route_digest()?;
        if signed.request.observed_route_digest != expected_route_digest {
            return Err(VertexError::RouteDigestMismatch {
                ticket_id: ticket.id,
                expected: expected_route_digest,
                received: signed.request.observed_route_digest,
            });
        }

        let release_nonce = self
            .ensure_account(signed.signer.account)?
            .next_release_nonce;
        if release_nonce != signed.request.release_nonce {
            return Err(VertexError::NonceMismatch {
                account: signed.signer.account,
                expected: release_nonce,
                received: signed.request.release_nonce,
            });
        }

        let tx_id = signed.tx_id()?;
        if self.seen_transactions.contains(&tx_id) {
            return Err(VertexError::DuplicateTransaction(tx_id));
        }

        let plan = ticket.route_plan;
        let route_charges = plan.charges()?;
        let beneficiary_amount = ticket.terms.amount.checked_sub(route_charges)?;
        let rebate_recipient = plan.rebate_recipient.unwrap_or(ticket.terms.beneficiary);

        self.credit(ticket.terms.beneficiary, beneficiary_amount)?;
        self.credit(plan.fee_recipient, plan.operator_fee)?;
        self.credit(rebate_recipient, plan.rebate_amount)?;

        self.tickets
            .get_mut(&ticket.id)
            .ok_or(VertexError::TicketNotFound(ticket.id))?
            .settled = true;
        self.accounts
            .get_mut(&signed.signer.account)
            .ok_or(VertexError::AccountNotFound(signed.signer.account))?
            .next_release_nonce = release_nonce
            .checked_add(1)
            .ok_or(VertexError::NonceOverflow)?;
        self.seen_transactions.insert(tx_id);
        self.journal.push(JournalEntry {
            tx_id,
            op: JournalOp::TicketSettlement {
                ticket_id: ticket.id,
                beneficiary: ticket.terms.beneficiary,
                beneficiary_amount,
                operator: plan.fee_recipient,
                operator_amount: plan.operator_fee,
                rebate_recipient,
                rebate_amount: plan.rebate_amount,
            },
        });

        Ok(tx_id)
    }

    fn credit(&mut self, account: AccountId, amount: Amount) -> VertexResult<()> {
        self.ensure_account(account)?;
        let current = self.balance_of(account)?;
        self.accounts
            .get_mut(&account)
            .ok_or(VertexError::AccountNotFound(account))?
            .balance = current.checked_add(amount)?;
        Ok(())
    }

    fn ensure_account(&self, account: AccountId) -> VertexResult<&AccountState> {
        self.accounts
            .get(&account)
            .ok_or(VertexError::AccountNotFound(account))
    }

    pub fn verify_conservation(&self) -> VertexResult<()> {
        let mut liquid = Amount::zero();
        for account in self.accounts.values() {
            liquid = liquid.checked_add(account.balance)?;
        }

        let mut locked = Amount::zero();
        for ticket in self.tickets.values() {
            if !ticket.settled {
                locked = locked.checked_add(ticket.terms.amount)?;
            }
        }

        let observed = liquid.checked_add(locked)?;
        if observed != self.total_supply {
            return Err(VertexError::Conservation {
                asset: self.asset,
                expected: self.total_supply,
                observed,
            });
        }

        Ok(())
    }
}

use serde::Serialize;

use crate::{
    AccountId, Amount, AssetId, Bps, Digest, KeyPair, ReleaseRequest, RoutePlan, RoutePolicy,
    SignedRelease, SignedTicket, TicketTerms, VertexLedger, VertexResult,
};

#[derive(Serialize)]
pub struct ScenarioReport {
    pub scenario: &'static str,
    pub network_id: u32,
    pub asset: AssetId,
    pub ticket_id: Option<crate::TicketId>,
    pub open_tx: Option<crate::TxId>,
    pub release_tx: Option<crate::TxId>,
    pub balances: BalanceReport,
    pub total_supply: Amount,
    pub state_digest: Digest,
    pub conservation_ok: bool,
}

#[derive(Serialize)]
pub struct BalanceReport {
    pub payer: Amount,
    pub beneficiary: Amount,
    pub operator: Amount,
    pub integrator: Amount,
    pub reserve: Amount,
}

struct Fixture {
    ledger: VertexLedger,
    payer: KeyPair,
    beneficiary: KeyPair,
    operator: KeyPair,
    integrator: KeyPair,
    reserve: KeyPair,
    network_id: u32,
    asset: AssetId,
}

impl Fixture {
    fn new() -> VertexResult<Self> {
        let network_id = 4_204;
        let asset = AssetId::native();
        let payer = KeyPair::from_seed([11u8; 32]);
        let beneficiary = KeyPair::from_seed([22u8; 32]);
        let operator = KeyPair::from_seed([33u8; 32]);
        let integrator = KeyPair::from_seed([44u8; 32]);
        let reserve = KeyPair::from_seed([55u8; 32]);
        let mut ledger = VertexLedger::new(network_id, asset);

        for identity in [
            payer.public_identity(),
            beneficiary.public_identity(),
            operator.public_identity(),
            integrator.public_identity(),
            reserve.public_identity(),
        ] {
            ledger.register_account(identity)?;
        }

        ledger.credit_genesis(payer.public_identity().account, Amount::new(5_000)?)?;
        Ok(Self {
            ledger,
            payer,
            beneficiary,
            operator,
            integrator,
            reserve,
            network_id,
            asset,
        })
    }

    fn route_policy(
        &self,
        max_charge_bps: u16,
        settlement_epoch: u64,
    ) -> VertexResult<RoutePolicy> {
        Ok(RoutePolicy::new(
            Digest::from_parts("vertex-demo-venue-v1", &[b"primary"]),
            Bps::new(max_charge_bps)?,
            settlement_epoch,
        ))
    }

    fn balances(&self) -> VertexResult<BalanceReport> {
        Ok(BalanceReport {
            payer: self
                .ledger
                .balance_of(self.payer.public_identity().account)?,
            beneficiary: self
                .ledger
                .balance_of(self.beneficiary.public_identity().account)?,
            operator: self
                .ledger
                .balance_of(self.operator.public_identity().account)?,
            integrator: self
                .ledger
                .balance_of(self.integrator.public_identity().account)?,
            reserve: self
                .ledger
                .balance_of(self.reserve.public_identity().account)?,
        })
    }

    fn report(
        &self,
        scenario: &'static str,
        ticket_id: Option<crate::TicketId>,
        open_tx: Option<crate::TxId>,
        release_tx: Option<crate::TxId>,
    ) -> VertexResult<ScenarioReport> {
        Ok(ScenarioReport {
            scenario,
            network_id: self.network_id,
            asset: self.asset,
            ticket_id,
            open_tx,
            release_tx,
            balances: self.balances()?,
            total_supply: self.ledger.total_supply(),
            state_digest: self.ledger.state_digest()?,
            conservation_ok: self.ledger.verify_conservation().is_ok(),
        })
    }
}

pub fn direct() -> VertexResult<ScenarioReport> {
    let mut fixture = Fixture::new()?;
    let policy = fixture.route_policy(50, 10)?;
    let operator = fixture.operator.public_identity().account;
    let route = RoutePlan::direct(operator, 10);
    open_and_release(&mut fixture, "direct", Amount::new(750)?, policy, route)
}

pub fn routed() -> VertexResult<ScenarioReport> {
    let mut fixture = Fixture::new()?;
    let policy = fixture.route_policy(150, 20)?;
    let route = RoutePlan {
        operator: fixture.operator.public_identity().account,
        fee_recipient: fixture.operator.public_identity().account,
        operator_fee: Amount::new(9)?,
        rebate_recipient: Some(fixture.integrator.public_identity().account),
        rebate_amount: Amount::new(3)?,
        liquidity_memo: Digest::from_parts("vertex-route-memo-v1", &[b"routed"]),
        routing_epoch: 20,
    };
    open_and_release(&mut fixture, "routed", Amount::new(1_000)?, policy, route)
}

pub fn batch() -> VertexResult<ScenarioReport> {
    let mut fixture = Fixture::new()?;
    let policy = fixture.route_policy(200, 30)?;
    let route_one = RoutePlan {
        operator: fixture.operator.public_identity().account,
        fee_recipient: fixture.operator.public_identity().account,
        operator_fee: Amount::new(4)?,
        rebate_recipient: Some(fixture.integrator.public_identity().account),
        rebate_amount: Amount::new(1)?,
        liquidity_memo: Digest::from_parts("vertex-route-memo-v1", &[b"batch-one"]),
        routing_epoch: 30,
    };
    let first = open_and_release_internal(&mut fixture, Amount::new(500)?, policy, route_one)?;

    let route_two = RoutePlan {
        operator: fixture.operator.public_identity().account,
        fee_recipient: fixture.reserve.public_identity().account,
        operator_fee: Amount::new(6)?,
        rebate_recipient: Some(fixture.integrator.public_identity().account),
        rebate_amount: Amount::new(2)?,
        liquidity_memo: Digest::from_parts("vertex-route-memo-v1", &[b"batch-two"]),
        routing_epoch: 31,
    };
    let _second = open_and_release_internal(&mut fixture, Amount::new(600)?, policy, route_two)?;

    fixture.report("batch", Some(first.0), Some(first.1), Some(first.2))
}

pub fn snapshot() -> VertexResult<ScenarioReport> {
    let fixture = Fixture::new()?;
    fixture.report("snapshot", None, None, None)
}

fn open_and_release(
    fixture: &mut Fixture,
    scenario: &'static str,
    amount: Amount,
    policy: RoutePolicy,
    route: RoutePlan,
) -> VertexResult<ScenarioReport> {
    let (ticket_id, open_tx, release_tx) =
        open_and_release_internal(fixture, amount, policy, route)?;
    fixture.report(scenario, Some(ticket_id), Some(open_tx), Some(release_tx))
}

fn open_and_release_internal(
    fixture: &mut Fixture,
    amount: Amount,
    policy: RoutePolicy,
    route: RoutePlan,
) -> VertexResult<(crate::TicketId, crate::TxId, crate::TxId)> {
    let payer = fixture.payer.public_identity().account;
    let beneficiary = fixture.beneficiary.public_identity().account;
    let payer_nonce = fixture.ledger.ticket_nonce(payer)?;
    let salt = Digest::from_parts(
        "vertex-demo-ticket-salt-v1",
        &[&payer_nonce.to_be_bytes(), &amount.units().to_be_bytes()],
    );
    let terms = TicketTerms::new(
        fixture.network_id,
        payer,
        beneficiary,
        fixture.asset,
        amount,
        payer_nonce,
        500,
        policy,
        salt,
    )?;
    let signed_ticket = SignedTicket::sign(terms, route, &fixture.payer)?;
    let open_tx = fixture.ledger.open_ticket(&signed_ticket)?;
    let digest = route.route_digest()?;
    let release = ReleaseRequest::new(
        fixture.network_id,
        terms.ticket_id,
        beneficiary,
        fixture.ledger.release_nonce(beneficiary)?,
        digest,
    );
    let signed_release = SignedRelease::sign(release, &fixture.beneficiary)?;
    let release_tx = fixture.ledger.settle_ticket(&signed_release)?;
    Ok((terms.ticket_id, open_tx, release_tx))
}

#[allow(dead_code)]
fn account_from_seed(seed: u8) -> AccountId {
    KeyPair::from_seed([seed; 32]).public_identity().account
}

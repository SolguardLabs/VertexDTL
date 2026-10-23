use vertex_dtl::{
    Amount, AssetId, Bps, Digest, KeyPair, ReleaseRequest, RouteMarket, RoutePlan, RoutePolicy,
    SignedRelease, SignedTicket, TicketTerms, VertexError, VertexLedger,
};

fn settle(ledger: &mut VertexLedger, party: &KeyPair, operator: &KeyPair, salt: &[u8]) -> u128 {
    let account = party.public_identity().account;
    let route = RoutePlan::direct(operator.public_identity().account, 10);
    let policy = RoutePolicy::new(Digest::zero(), Bps::new(1_000).unwrap(), 10);
    let terms = TicketTerms::new(
        77,
        account,
        account,
        AssetId::native(),
        Amount::new(500_000).unwrap(),
        ledger.ticket_nonce(account).unwrap(),
        100,
        policy,
        Digest::from_parts("vertex-route-test", &[salt]),
    )
    .unwrap();
    let signed = SignedTicket::sign(terms, route, party).unwrap();
    ledger.open_ticket(&signed).unwrap();
    let quote = ledger.ticket_allocation(terms.ticket_id).unwrap();
    let release = ReleaseRequest::new(
        77,
        terms.ticket_id,
        account,
        ledger.release_nonce(account).unwrap(),
        route.route_digest().unwrap(),
    );
    ledger
        .settle_ticket(&SignedRelease::sign(release, party).unwrap())
        .unwrap();
    quote.congestion_surcharge.units()
}

#[test]
fn route_market_reconciles_sequential_ticket_settlements() {
    let first = KeyPair::from_seed([51; 32]);
    let second = KeyPair::from_seed([52; 32]);
    let operator = KeyPair::from_seed([53; 32]);
    let mut ledger = VertexLedger::new(77, AssetId::native());
    for identity in [&first, &second, &operator] {
        ledger.register_account(identity.public_identity()).unwrap();
    }
    for payer in [&first, &second] {
        ledger
            .credit_genesis(
                payer.public_identity().account,
                Amount::new(500_000).unwrap(),
            )
            .unwrap();
    }
    ledger
        .configure_route_market(
            operator.public_identity().account,
            Amount::new(1_000_000).unwrap(),
            Bps::new(1_000).unwrap(),
        )
        .unwrap();
    assert_eq!(settle(&mut ledger, &first, &operator, b"one"), 12_500);
    assert_eq!(settle(&mut ledger, &second, &operator, b"two"), 50_000);
    assert_eq!(
        ledger
            .balance_of(first.public_identity().account)
            .unwrap()
            .units(),
        487_500
    );
    assert_eq!(
        ledger
            .balance_of(second.public_identity().account)
            .unwrap()
            .units(),
        450_000
    );
    assert_eq!(
        ledger
            .balance_of(operator.public_identity().account)
            .unwrap()
            .units(),
        62_500
    );
    assert_eq!(
        ledger
            .route_market(operator.public_identity().account)
            .unwrap()
            .settled
            .units(),
        1_000_000
    );
    ledger.verify_conservation().unwrap();
}

#[test]
fn route_market_enforces_capacity_and_signature_binding() {
    let payer = KeyPair::from_seed([61; 32]);
    let operator = KeyPair::from_seed([62; 32]);
    let market = RouteMarket::new(Amount::new(100).unwrap(), Bps::new(1_000).unwrap()).unwrap();
    assert!(matches!(
        market.quote(Amount::new(101).unwrap()),
        Err(VertexError::Policy(_))
    ));

    let mut ledger = VertexLedger::new(77, AssetId::native());
    ledger.register_account(payer.public_identity()).unwrap();
    ledger.register_account(operator.public_identity()).unwrap();
    ledger
        .credit_genesis(payer.public_identity().account, Amount::new(500).unwrap())
        .unwrap();
    let policy = RoutePolicy::new(Digest::zero(), Bps::new(1_000).unwrap(), 10);
    let route = RoutePlan::direct(operator.public_identity().account, 10);
    let terms = TicketTerms::new(
        77,
        payer.public_identity().account,
        payer.public_identity().account,
        AssetId::native(),
        Amount::new(500).unwrap(),
        0,
        100,
        policy,
        Digest::zero(),
    )
    .unwrap();
    let signed = SignedTicket::sign(terms, route, &payer).unwrap();
    let mut altered = signed.clone();
    altered.route_plan.operator_fee = Amount::new(1).unwrap();
    assert_eq!(
        ledger.open_ticket(&altered),
        Err(VertexError::InvalidSignature)
    );
    let mut stale = route;
    stale.routing_epoch = 9;
    let signed_stale = SignedTicket::sign(terms, stale, &payer).unwrap();
    assert!(matches!(
        ledger.open_ticket(&signed_stale),
        Err(VertexError::Policy(_))
    ));
}

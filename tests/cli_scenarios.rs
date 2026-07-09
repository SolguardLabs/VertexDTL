use serde_json::Value;
use std::process::Command;

fn scenario(name: &str) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_vertex_dtl"))
        .arg(name)
        .output()
        .expect("el proceso del escenario debe ejecutarse");

    assert!(
        output.status.success(),
        "el escenario {name} fallo: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("el escenario debe emitir json")
}

fn assert_hex32(value: &Value) {
    let text = value.as_str().expect("el valor debe ser una cadena");
    assert_eq!(text.len(), 64);
    assert!(text.chars().all(|ch| ch.is_ascii_hexdigit()));
}

fn assert_common(payload: &Value, name: &str) {
    assert_eq!(payload["scenario"], name);
    assert_eq!(payload["network_id"], 4_204);
    assert_eq!(payload["total_supply"], 5_000);
    assert_eq!(payload["conservation_ok"], true);
    assert_hex32(&payload["asset"]);
    assert_hex32(&payload["state_digest"]);
}

#[test]
fn escenario_directo_liquida_saldos_esperados() {
    let payload = scenario("direct");
    assert_common(&payload, "direct");
    assert_hex32(&payload["ticket_id"]);
    assert_hex32(&payload["open_tx"]);
    assert_hex32(&payload["release_tx"]);
    assert_eq!(payload["balances"]["payer"], 4_250);
    assert_eq!(payload["balances"]["beneficiary"], 750);
    assert_eq!(payload["balances"]["operator"], 0);
    assert_eq!(payload["balances"]["integrator"], 0);
    assert_eq!(payload["balances"]["reserve"], 0);
}

#[test]
fn escenario_enrutado_aplica_importes_de_ruta() {
    let payload = scenario("routed");
    assert_common(&payload, "routed");
    assert_hex32(&payload["ticket_id"]);
    assert_eq!(payload["balances"]["payer"], 4_000);
    assert_eq!(payload["balances"]["beneficiary"], 988);
    assert_eq!(payload["balances"]["operator"], 9);
    assert_eq!(payload["balances"]["integrator"], 3);
    assert_eq!(payload["balances"]["reserve"], 0);
}

#[test]
fn escenario_batch_acumula_varias_liquidaciones() {
    let payload = scenario("batch");
    assert_common(&payload, "batch");
    assert_hex32(&payload["ticket_id"]);
    assert_eq!(payload["balances"]["payer"], 3_900);
    assert_eq!(payload["balances"]["beneficiary"], 1_087);
    assert_eq!(payload["balances"]["operator"], 4);
    assert_eq!(payload["balances"]["integrator"], 3);
    assert_eq!(payload["balances"]["reserve"], 6);
}

#[test]
fn escenario_snapshot_no_tiene_actividad_de_ticket() {
    let payload = scenario("snapshot");
    assert_common(&payload, "snapshot");
    assert!(payload["ticket_id"].is_null());
    assert!(payload["open_tx"].is_null());
    assert!(payload["release_tx"].is_null());
    assert_eq!(payload["balances"]["payer"], 5_000);
    assert_eq!(payload["balances"]["beneficiary"], 0);
    assert_eq!(payload["balances"]["operator"], 0);
    assert_eq!(payload["balances"]["integrator"], 0);
    assert_eq!(payload["balances"]["reserve"], 0);
}

#[test]
fn digests_de_escenarios_son_deterministas_y_distintos() {
    let first = scenario("routed");
    let second = scenario("routed");
    let direct = scenario("direct");

    assert_eq!(first["state_digest"], second["state_digest"]);
    assert_ne!(first["state_digest"], direct["state_digest"]);
}

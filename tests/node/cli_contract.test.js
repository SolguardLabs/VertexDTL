import assert from "node:assert/strict";
import test from "node:test";

import { assertCommon, assertHex32, scenario } from "../helpers/scenario_helpers.js";

test("los escenarios exponen el contrato superior esperado", () => {
    for (const name of ["direct", "routed", "batch", "snapshot"]) {
        const payload = scenario(name);
        assertCommon(payload, name);
        for (const key of [
            "scenario",
            "network_id",
            "asset",
            "ticket_id",
            "open_tx",
            "release_tx",
            "balances",
            "total_supply",
            "state_digest",
            "conservation_ok",
        ]) {
            assert.ok(Object.hasOwn(payload, key), `${name} no incluye ${key}`);
        }
    }
});

test("los escenarios liquidados serializan identificadores de 32 bytes", () => {
    for (const name of ["direct", "routed", "batch"]) {
        const payload = scenario(name);
        assertHex32(payload.ticket_id);
        assertHex32(payload.open_tx);
        assertHex32(payload.release_tx);
    }
});

test("snapshot no incluye actividad de ticket", () => {
    const payload = scenario("snapshot");
    assert.equal(payload.ticket_id, null);
    assert.equal(payload.open_tx, null);
    assert.equal(payload.release_tx, null);
});

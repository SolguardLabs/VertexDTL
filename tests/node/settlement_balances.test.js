import assert from "node:assert/strict";
import test from "node:test";

import { assertCommon, scenario } from "../helpers/scenario_helpers.js";

test("direct conserva suministro y aplica balance neto directo", () => {
    const payload = scenario("direct");
    assertCommon(payload, "direct");
    assert.deepEqual(payload.balances, {
        payer: 4_250,
        beneficiary: 750,
        operator: 0,
        integrator: 0,
        reserve: 0,
    });
});

test("routed conserva suministro y aplica importes de ruta", () => {
    const payload = scenario("routed");
    assertCommon(payload, "routed");
    assert.deepEqual(payload.balances, {
        payer: 4_000,
        beneficiary: 988,
        operator: 9,
        integrator: 3,
        reserve: 0,
    });
});

test("batch acumula dos liquidaciones en el mismo ledger", () => {
    const payload = scenario("batch");
    assertCommon(payload, "batch");
    assert.deepEqual(payload.balances, {
        payer: 3_900,
        beneficiary: 1_087,
        operator: 4,
        integrator: 3,
        reserve: 6,
    });
});

test("snapshot conserva el estado inicial financiado", () => {
    const payload = scenario("snapshot");
    assertCommon(payload, "snapshot");
    assert.deepEqual(payload.balances, {
        payer: 5_000,
        beneficiary: 0,
        operator: 0,
        integrator: 0,
        reserve: 0,
    });
});

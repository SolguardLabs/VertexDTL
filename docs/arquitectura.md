# Arquitectura y flujo de estados

VertexDTL mantiene un ledger local determinista. `AccountState` conserva identidad pública, saldo y nonces; `VertexLedger` administra tickets, transacciones observadas, suministro y journal. La serialización canónica aporta bytes estables a firmas, identificadores y digest de estado. El ejecutable expone escenarios JSON para integradores y pruebas de regresión.

La biblioteca `src/lib.rs` expone el mismo núcleo usado por la CLI. Esto
permite que un servicio de integración utilice tipos y transiciones sin
depender de la presentación JSON de `src/main.rs`.

```mermaid
stateDiagram-v2
    [*] --> Preparado: términos y cuentas
    Preparado --> Bloqueado: open_ticket
    Bloqueado --> Liquidado: settle_ticket
    Liquidado --> [*]
```

Al abrir, el ledger valida identidad, red, activo, cuentas de destino, política de ruta, nonce y saldo; después debita al pagador y conserva el importe en el ticket. Al liquidar, valida la firma del beneficiario, su nonce y el digest de ruta observado. Finalmente acredita beneficiario y destinatarios de cargos, marca el ticket liquidado y verifica conservación en un estado candidato antes de confirmarlo.

Si existe un `RouteMarket` para el operador, la vista `CongestionQuote` lee
capacidad, uso confirmado y pendiente. `ticket_allocation` compone cargo base,
congestión y rebate y vuelve a imponer el techo firmado sobre la suma. El uso
confirmado solo aumenta en el mismo estado candidato que acredita los saldos.

```mermaid
flowchart TB
    T[Ticket + plan firmados] --> A[Admisión]
    A --> L[Ticket bloqueado]
    M[Mercado por operador] --> Q[Cotización de capacidad]
    L --> Q
    Q --> P{Techo de política}
    P -->|válido| S[Liquidación candidata]
    P -->|rechazo| E[Estado intacto]
    S --> C{Conservación}
    C -->|correcta| F[Commit + journal + carga]
    C -->|fallo| E
```

La separación entre dominios de firma de ticket y liberación evita confundir las dos autorizaciones. `RouteAllocation` presenta una cotización aritmética de la distribución; no firma, abre ni liquida tickets.

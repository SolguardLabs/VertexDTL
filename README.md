# Vertex DTL

![VertexDTL — Solguard Labs](./assets/banner.png)

Versión vigente: `Production 2.0.0` (`v2.0.0`).

Vertex DTL es un motor de liquidacion determinista escrito en Rust para flujos
de pago diferido entre cuentas autenticadas. El protocolo modela tickets
firmados, rutas de ejecucion, cargos operativos y liberaciones autorizadas sobre
un ledger local con conservacion de suministro. El mercado de capacidad de la
ruta permite cotizar una tarifa de congestión no lineal al liquidar.

El proyecto esta pensado como una implementacion compacta de referencia para
validar reglas de liquidacion, serializacion canonica y consistencia de estado
en entornos de integracion Web3.

```mermaid
sequenceDiagram
    participant P as Pagador
    participant L as Ledger
    participant B as Beneficiario
    participant R as Ruta
    P->>L: TicketTerms + firma
    L->>L: valida identidad, nonce y saldo
    L->>L: bloquea el importe
    B->>L: ReleaseRequest + firma
    L->>R: distribuye beneficiario, operador y rebate
    L->>L: verifica conservación y digest
```

## Caracteristicas principales

- Ledger determinista con digest de estado reproducible.
- Cuentas derivadas desde identidades publicas verificables.
- Tickets de pago firmados por el pagador.
- La firma del ticket vincula el plan concreto de ruta y su política.
- Solicitudes de liberacion firmadas por el beneficiario.
- Rutas de liquidacion con operador, receptor de comision y receptor de rebate.
- Nonces independientes para apertura de tickets y liberaciones.
- Deteccion de transacciones duplicadas mediante identificadores derivados.
- Comprobacion de conservacion de suministro tras cada mutacion critica.
- Reportes JSON estables para escenarios de integracion.
- Mercados de capacidad por operador con contador de uso, pendiente en bps y
  cotización de la asignación completa antes de liquidar.

## Arquitectura

El codigo esta organizado por dominios funcionales:

- `src/amount`: tipos de importe y calculo en basis points.
- `src/codec`: serializacion canonica usada para digests e identificadores.
- `src/crypto`: identidades, claves y verificacion de firmas Ed25519.
- `src/error`: errores de dominio del protocolo.
- `src/ids`: identificadores de cuentas, activos, tickets, transacciones y
  digests.
- `src/ledger`: estado contable, journal, cuentas y transiciones de tickets.
- `src/settlement`: terminos de tickets, politicas de ruta y liberaciones.
- `src/settlement/congestion`: capacidad, utilización proyectada y recargo.
- `RouteAllocation`: cotizacion de la distribucion de una ruta concreta sin mutar el ledger.
- `src/runtime`: CLI y escenarios reproducibles.

## Flujo de liquidacion

1. El ledger se inicializa con un `network_id`, un activo nativo y cuentas
   registradas.
2. El pagador crea un `TicketTerms` con importe, beneficiario, activo, nonce y
   politica de ruta.
3. El ticket y su ruta concreta se firman dentro del dominio
   `vertex-ticket-open-v4`.
4. El ledger abre el ticket, debita al pagador y bloquea el importe.
5. El beneficiario observa la ruta, firma un `ReleaseRequest` dentro del dominio
   `vertex-ticket-release-v3` y solicita la liquidacion.
6. El ledger valida firmas, nonces, digest de ruta, estado del ticket y
   conservacion de suministro.
7. La liquidacion cotiza también el uso de capacidad, aplica el techo de
   cargos firmado y acredita beneficiario, operador y receptor de rebate.

Para un ticket de 1.000 unidades con 9 de cargo operativo y 3 de rebate, la
distribucion es 988 al beneficiario, 9 al receptor del cargo y 3 al receptor
del rebate. `RouteAllocation::recomposed_total` confirma 988 + 9 + 3 = 1.000.
La cotizacion es una lectura; la aceptacion final depende del estado, firmas,
nonces y reglas del ledger.

### Mercado de capacidad

El operador puede registrar una capacidad y una pendiente de congestión en
puntos básicos. La cotización informa carga previa, carga proyectada,
utilización y recargo. `ticket_allocation` integra ese recargo con el cargo
base y el rebate, y rechaza una distribución que supere el techo firmado.
La carga se actualiza únicamente tras confirmar la liquidación. Por ejemplo,
con capacidad de 1.000.000 unidades y pendiente máxima de 1.000 bps, dos
tickets secuenciales de 500.000 alcanzan utilizaciones proyectadas del 50 %
y el 100 %, respectivamente; los importes siguen conciliando con el suministro.

```mermaid
flowchart LR
    T[Ticket firmado] --> Q[Cotización de ruta]
    C[Capacidad y carga confirmada] --> Q
    P[Techo de cargos firmado] --> V{Validación}
    Q --> V
    V -->|aceptada| B[Beneficiario neto]
    V -->|aceptada| O[Operador: base + congestión]
    V -->|aceptada| R[Destinatario de rebate]
    B --> S[Conciliación]
    O --> S
    R --> S
```

## Escenarios disponibles

El binario acepta un argumento opcional de escenario. Si no se indica ninguno,
ejecuta `routed`.

```bash
cargo run --locked -- direct
cargo run --locked -- routed
cargo run --locked -- batch
cargo run --locked -- snapshot
```

Cada escenario emite un reporte JSON con:

- `scenario`: nombre del escenario ejecutado.
- `network_id`: identificador de red.
- `asset`: activo liquidado.
- `ticket_id`: identificador del ticket, cuando aplica.
- `open_tx`: transaccion de apertura, cuando aplica.
- `release_tx`: transaccion de liberacion, cuando aplica.
- `balances`: saldos finales por rol.
- `total_supply`: suministro total registrado.
- `state_digest`: digest final del ledger.
- `conservation_ok`: resultado de la comprobacion contable.

## Requisitos

- Rust `1.96.0`.
- Bun `1.3.0` o superior.
- Node.js `24` recomendado para herramientas auxiliares.

## Instalacion

```bash
bun install --frozen-lockfile
cargo build --all-targets --locked
```

## Comandos de desarrollo

```bash
bun run test        # tests JavaScript sobre la CLI
bun run test:rust   # tests Rust
bun run test:all    # tests Rust y JavaScript
bun run fmt         # formato JavaScript
bun run fmt:check   # verificacion de formato JavaScript
bun run build       # verificacion de sintaxis JavaScript
bun run ci          # pipeline local completo
```

Tambien se incluyen scripts POSIX equivalentes:

```bash
bash scripts/tests.sh
bash scripts/ci.sh
```

En Windows, se recomienda ejecutarlos desde Git Bash. Desde PowerShell, `bun run
ci` y `bun run test:all` son las rutas mas directas.

## Calidad y CI

El workflow de GitHub Actions ejecuta:

- `bun install --frozen-lockfile`
- `cargo fmt --all -- --check`
- `cargo build --all-targets --locked`
- `cargo test --locked`
- `cargo clippy --all-targets --all-features --locked -- -D warnings`
- `bun run fmt:check`
- `bun run build`
- `bun test --timeout 30000 ./tests/node`

Dependabot esta configurado para revisar dependencias de Cargo, Bun y GitHub
Actions semanalmente.

## Estado del proyecto

El release identifica codigo reproducible y sus controles de CI. No implica
despliegue en una red externa, custodia de fondos ni auditoria independiente.

## Documentacion

- [Arquitectura y flujo de estados](./docs/arquitectura.md)
- [Modelo economico](./docs/modelo-economico.md)
- [Integracion y CLI](./docs/integracion.md)
- [Operacion y controles](./docs/operacion.md)
- [Riesgo de ruta y controles](./docs/riesgos.md)
- [Politica de seguridad](./SECURITY.md)

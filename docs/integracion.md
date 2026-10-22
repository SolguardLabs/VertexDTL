# Integración y CLI

Los escenarios `direct`, `routed`, `batch` y `snapshot` ejecutan el mismo motor y emiten un objeto JSON por salida estándar. El ejecutable puede invocarse con `cargo run --locked -- <scenario>`. `network_id`, `asset`, saldos, `total_supply`, `state_digest` y `conservation_ok` permiten validar un resultado sin analizar logs humanos.

```bash
cargo run --locked -- routed
```

El escenario `routed` parte de 5.000 unidades y liquida un ticket de 1.000: 4.000 quedan al pagador, 988 al beneficiario, 9 al operador y 3 al integrador. `state_digest` cambia si cambia el estado, y debe compararse solo entre ejecuciones con la misma configuración y versión.

Para integrar firmas, el cliente debe obtener la identidad pública de una clave, construir `TicketTerms` con red, activo, beneficiario, nonce, vencimiento y política, y firmar en el dominio de apertura. El beneficiario construye `ReleaseRequest` con el digest de la ruta que acepta, su nonce y el dominio de liberación. No se deben reutilizar nonces ni asumir que una firma de liberación equivale a una firma de apertura.

`RouteAllocation::from_plan` devuelve una descomposición de importes que un cliente puede mostrar antes de ejecutar. No es un simulador completo de `open_ticket`: faltan saldo, nonces, cuentas registradas y estado de duplicados.

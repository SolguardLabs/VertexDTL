# Operación y controles

El pipeline ejecuta `cargo fmt --check`, build y tests con lockfile, Clippy sin advertencias, formato y sintaxis JavaScript, y pruebas de integración de escenarios. La CI de GitHub valida solicitudes de cambio y referencias `main`, `production` y tags. `release-integrity` comprueba que `main`, `production` y `v2.0.0^{}` apunten al mismo commit.

```bash
bun install --frozen-lockfile
bash scripts/ci.sh
```

La rama y el release identifican código; no demuestran despliegue de un servicio remoto. Para una operación persistente se necesitarían custodia de claves, almacenamiento transaccional, recuperación ante fallo, gestión de secretos y telemetría, fuera de este repositorio.

Para observabilidad, conservar `network_id`, ticket y transacción, nonce esperado, digest de ruta y estado, distribución por rol y resultado de conservación. No registrar material secreto. Ante divergencia, detener la ingestión de nuevos tickets, guardar el estado previo y reproducir la transición con la misma versión; no corregir saldos manualmente sin reconciliar el journal.

El mercado de capacidad requiere métricas por operador de capacidad,
`settled`, utilización proyectada, recargo cotizado y recargo finalmente
acreditado. La configuración inicial de `configure_route_market` pertenece
al plano de control autorizado del integrador; el crate no implementa
autenticación administrativa para esa llamada. Alertar si una cotización
rebasa la capacidad o el techo firmado. Al reproducir el journal, conservar
el orden de liquidación: la tarifa depende de la carga ya confirmada.

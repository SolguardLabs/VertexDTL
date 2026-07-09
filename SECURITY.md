# Politica de seguridad

La seguridad de Vertex DTL se apoya en controles criptograficos, validaciones de
dominio y comprobaciones contables deterministas. Este documento resume el
alcance de seguridad del proyecto, las garantias esperadas y el proceso de
gestion de hallazgos.

## Alcance

Se consideran dentro de alcance:

- Logica de apertura y liquidacion de tickets.
- Validacion de firmas y dominios de autorizacion.
- Gestion de nonces de pagador y beneficiario.
- Derivacion de identificadores y digests.
- Conservacion de suministro del ledger.
- Reglas de ruta, comisiones y rebates.
- Escenarios reproducibles expuestos por la CLI.
- Tests Rust y JavaScript incluidos en el repositorio.
- Workflows de CI y configuracion de dependencias.

Quedan fuera de alcance:

- Custodia real de claves privadas.
- Integracion con redes publicas o contratos desplegados.
- Persistencia externa, bases de datos o servicios remotos.
- Gestion de usuarios, permisos de sistema operativo o infraestructura cloud.
- Riesgos introducidos por modificaciones locales no revisadas.

## Modelo de confianza

El protocolo asume que:

- Las claves privadas se generan y almacenan fuera del ledger.
- Cada cuenta registrada corresponde a una identidad publica coherente.
- Los firmantes protegen sus claves y no delegan firmas fuera de los dominios
  previstos.
- Los integradores ejecutan la version exacta del binario que ha sido validada
  por CI.
- Los reportes JSON se tratan como salida de verificacion, no como fuente
  externa de verdad.

## Controles implementados

### Firmas y dominios

Los tickets se autorizan mediante firmas Ed25519 sobre una vista canonica de los
terminos. Las liberaciones usan un dominio de firma separado. Esta separacion
reduce el riesgo de reutilizacion de firmas entre acciones con significado
distinto.

### Identificadores deterministas

Los identificadores de tickets, transacciones y estado se derivan a partir de
datos canonicos y dominios explicitos. Esto permite reproducir resultados entre
ejecuciones y detectar cambios no esperados en la transicion de estado.

### Nonces

El ledger mantiene nonces independientes para apertura de tickets y liberacion.
Cada transicion valida el nonce esperado antes de modificar el estado, lo que
protege contra reenvios simples y ejecuciones duplicadas.

### Conservacion de suministro

Las mutaciones criticas se aplican sobre un candidato de estado y solo se
confirman si la suma de saldos liquidos mas importes bloqueados coincide con el
suministro total registrado.

### Deteccion de duplicados

Las transacciones procesadas se almacenan por identificador. Cualquier intento
de reaplicar una transaccion ya observada se rechaza antes de persistir cambios.

### Validacion de rutas

Las rutas de liquidacion declaran operador, receptores, importes asociados y un
memo de liquidez. La liberacion comprueba que el digest observado de la ruta
coincide con el plan registrado durante la apertura del ticket.

## Practicas de desarrollo

Todo cambio deberia pasar por:

```bash
bun run ci
```

El pipeline verifica formato, compilacion, tests Rust, Clippy, formato
JavaScript, sintaxis JavaScript y tests de integracion desde Node/Bun.

Para revisiones rapidas:

```bash
bun run test:all
```

## Gestion de dependencias

El repositorio mantiene lockfiles para Rust y Bun. Las actualizaciones se
gestionan mediante Dependabot para:

- Cargo.
- Bun.
- GitHub Actions.

Las actualizaciones de dependencias deben revisarse con el mismo criterio que un
cambio de codigo, especialmente cuando afectan a criptografia, serializacion o
tooling de CI.

## Reporte responsable

Los hallazgos de seguridad deben reportarse de forma privada al equipo
mantenedor antes de cualquier divulgacion publica. Un reporte util debe incluir:

- Version o commit analizado.
- Sistema operativo y versiones de Rust/Bun utilizadas.
- Pasos de reproduccion.
- Impacto tecnico estimado.
- Evidencia minima necesaria para validar el hallazgo.
- Recomendacion de mitigacion, si aplica.

No se deben publicar pruebas de concepto, trazas completas ni datos que permitan
reproducir un impacto economico sin coordinacion previa.

## Criterios de severidad

- Critica: perdida de fondos, creacion no autorizada de suministro,
  liquidaciones no autorizadas o ruptura de invariantes contables.
- Alta: bypass de firmas, reuso de autorizaciones, duplicacion de transacciones
  o corrupcion persistente del estado.
- Media: denegacion de servicio local, validaciones incompletas sin impacto
  contable directo o fallos reproducibles en escenarios soportados.
- Baja: problemas de tooling, documentacion, mensajes de error o hardening sin
  impacto directo en la liquidacion.

## Limitaciones

Vertex DTL no incluye gestion de claves en produccion, persistencia tolerante a
fallos, consenso distribuido, integracion on-chain ni monitorizacion operativa.
Cualquier uso fuera de entornos controlados requiere una revision de
arquitectura, auditoria externa y validacion de invariantes especifica para el
entorno de despliegue.

# Decisiones de Arquitectura (ADR)

Los ADR (*Architecture Decision Records*) registran las decisiones técnicas importantes y su **PORQUÉ**, para que código y contexto no se pierdan entre sesiones. Formato inspirado en Michael Nygard.

## Archivos

| ADR | Decisión |
|-----|----------|
| `ADR-001-architecture-ipc-persistence.md` | Arquitectura general, persistencia e IPC |
| `ADR-002-sqlite-schema.md` | Esquema SQLite |
| `ADR-003-hexagonal-ports.md` | Puertos/adaptadores (arquitectura hexagonal) |
| `ADR-004-clock-modes-and-penalties.md` | Modos de reloj, contrato y penalizaciones |
| `ADR-005-tpt-cli-crate.md` | Crate `tpt-cli` (control/report/import/daemon) |
| `ADR-006-branching-and-protection.md` | Branching (`dev`/`main`) y protección de ramas |
| `ADR-007-timer-first-pivot.md` | Pivote a *timer-first* (retira el tracking pasivo) |
| `ADR-008-presence-and-penalties.md` | Presencia: idle, pantalla apagada y suspensión |

## Cuándo escribir un ADR

- Decisión técnica con impacto duradero (arquitectura, persistencia, seguridad, contrato de producto).
- Cuando hay alternativas reales que descartamos.
- Cuando un futuro cambio podría "deshacer" la decisión sin contexto.

## Reglas

- Una decisión = un archivo `ADR-0NN-nombre.md`.
- Numeración secuencial; **no reescribir** ADRs pasados (son historia).
- Si una decisión cambia, se crea un ADR nuevo con Estado "Reemplazado por ADR-0NN".
- Estados: `Propuesto` → `Aceptado` → `Reemplazado`.
- Usar `template-adr.md`.

## Reestructuración de 2026-09

El pivote a *timer-first* (`ADR-007`) retiró los ADRs del diseño anterior y renumeró el resto:

| Antes | Ahora |
|-------|-------|
| ADR-005 Inactividad y blocklist | **retirado** (sin tracking no hay blocklist) |
| ADR-006 Retención y BTRFS | **retirado** (sin flood de eventos no hay tiers) |
| ADR-007 Crate `tpt-cli` | renumerado → `ADR-005` |
| ADR-008 Modelo de tracking en 3 capas | **retirado** (el tracking se abandonó) |

## Resumen en architecture

`docs/architecture.md` mantiene una tabla resumen con links a estos ADRs. El detalle vive aquí.

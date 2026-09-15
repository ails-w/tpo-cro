# Decisiones de Arquitectura (ADR)

Los ADR (*Architecture Decision Records*) registran las decisiones técnicas importantes y su **PORQUÉ**, para que código y contexto no se pierdan entre sesiones. Formato inspirado en Michael Nygard.

## Archivos

| ADR | Decisión |
|-----|----------|
| `ADR-001-architecture-ipc-persistence.md` | Arquitectura general, persistencia e IPC |
| `ADR-002-sqlite-schema.md` | Esquema SQLite (categorías, tareas, blocklist, sesiones, agregados) |
| `ADR-003-hexagonal-ports.md` | Puertos/adaptadores (arquitectura hexagonal) |
| `ADR-004-clock-modes-and-penalties.md` | Modos de reloj (Pomodoro/Focus/Flowtime) y penalizaciones |
| `ADR-005-inactivity-and-blocklist.md` | Inactividad no penalizada + blocklist por tarea |
| `ADR-006-data-retention-btrfs.md` | Retención modular y estrategia BTRFS |
| `ADR-007-tpt-cli-crate.md` | Crate `tpt-cli` (control/report/import) |
| `ADR-008-tracking-layers.md` | Modelo de tracking en 3 capas (categorización, asignación, blocklist) |

## Cuándo escribir un ADR

- Decisión técnica con impacto duradero (framework, IPC, base de datos, seguridad).
- Cuando hay alternativas reales que descartamos.
- Cuando un futuro cambio podría "deshacer" la decisión sin contexto.

## Reglas

- Una decisión = un archivo `ADR-0NN-nombre.md`.
- Numeración secuencial; **no reescribir** ADRs pasados (son historia).
- Si una decisión cambia, se crea un ADR nuevo con Estado "Reemplazado por ADR-0NN".
- Estados: `Propuesto` → `Aceptado` → `Reemplazado`.
- Usar `template-adr.md`.

## Resumen en architecture

`docs/architecture.md` mantiene una tabla resumen con links a estos ADRs. El detalle vive aquí.
# ADR-006: Retención Modular de Datos y Estrategia BTRFS

- **Estado**: Aceptado
- **Fecha**: 2026-09-10

## Contexto

El filesystem es BTRFS (CoW). SQLite en WAL sobre BTRFS sin configuración sufre write amplification por copy-on-write y fragmentación. Además `window_activity_logs` crecería sin límite, y los snapshots BTRFS de una DB con WAL activo son inconsistentes. El presupuesto <15 MB RAM exige lecturas eficientes.

## Decisión

**Tiers de retención modular (D17):**
- `raw_events` (`window_activity_logs`) → 30 días (default).
- `hourly_aggregates` → 90 días (default).
- `daily_aggregates` → permanente.
- Retención, cadencia de agregación y tamaño máx configurables en `config.toml`; una tarea de mantenimiento en el daemon las ejecuta.

**Estrategia BTRFS (D18):**
- `chattr +C` (nodatacow) en el directorio de la DB **antes** de crearla.
- WAL: `synchronous=NORMAL` + `wal_checkpoint(TRUNCATE)` periódico para no dejar el WAL crecer.
- `auto_vacuum=INCREMENTAL` + `PRAGMA incremental_vacuum(N)`; **nunca** `VACUUM` completo (reescribe todo el archivo; pésimo en CoW).
- Snapshots BTRFS: excluir la DB o usar backup consistente (API `.backup` / `VACUUM INTO` + swap atómico).
- Compresión `zstd` por propiedad del archivo (a validar en Fase 10).

**Escritura:**
- Un solo hilo escritor MPSC; escrituras por lotes; merge de eventos consecutivos de la misma app.

## Alternativas consideradas

- **Una sola tabla de logs sin retención**: DB sin límite y consultas lentas en rangos largos.
- **`VACUUM` periódico**: reescribe el archivo completo; genera nuevos extents en BTRFS (write amplification).
- **Bases separadas por tier**: complejidad de migración innecesaria hoy; los agregados en la misma DB bastan.

## Consecuencias

- Consultas de analíticas siempre sobre agregados compactos.
- DB acotada y predecible; mantenimiento programado, no reactivo.
- Requiere validar nodatacow y snapshots en disco real (Fase 10, feature 10.3).

## Referencias

- `docs/vision.md` (2.8) · `docs/development-plan.md` (BTRFS) · `docs/phases.md` (Fases 2 y 10)
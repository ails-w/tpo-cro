# ADR-007: Crate `tpt-cli` — Cliente de Control, Report e Import

- **Estado**: Aceptado
- **Fecha**: 2026-09-10

## Contexto

La visión original mencionaba `tpt-cli` para reportes e import, pero el plan de crates solo contemplaba `tpt-core`/`tpt-daemon`/`tpt-tui`. El usuario quiere además poder **ajustar e identificar tiempo por comando** sin abrir la TUI, y el daemon debe mantener un consumo de memoria mínimo sin arrastrar dependencias de reporte/import.

## Decisión

4º crate **`tpt-cli`** (D1): binario delgado que es **cliente IPC** del daemon y usa `tpt-core` para lógica compartida.

- Comandos: `status`, `focus start|abort`, `pomodoro start|abort`, `timer adjust ±`, `task switch`, `flowtime start`, `report --range weekly|monthly --format json|csv`, `import --super-productivity save.json`.
- El daemon sigue siendo el único dueño de la captura/persistencia/enforcement; el CLI solo envía comandos y lee estado/reportes.
- El protocolo IPC compartido vive en `tpt-core` (ver `ADR-001`), reutilizado por TUI y CLI.

## Alternativas consideradas

- **Subcomandos dentro de `tpt-daemon`**: mezcla responsabilidades (daemon = always-on; CLI = on-demand), obliga a cargar deps de CSV/import en el proceso crítico y a tener el daemon corriendo para importar/reportar. Menos limpio.
- **CLI que lee SQLite directamente**: duplica lógica de consulta y riesgo de corrupción concurrente con el escritor del daemon.

## Consecuencias

- SRP: daemon captura, CLI controla/reporta/importa.
- RAM y build del daemon acotados (sin deps de reporte/import).
- Import/report funcionan incluso con el daemon apagado (sobre la DB, vía `tpt-core`).
- Más superficie de comando a mantener (mitigado por `clap` y tests por comando).

## Referencias

- `docs/vision.md` (2.6, 2.7, sección 3) · `docs/architecture.md` (crates) · `docs/phases.md` (Fases 6 y 8)
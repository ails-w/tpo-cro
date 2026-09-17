# ADR-005: Crate `tpt-cli`

- **Estado**: Aceptado
- **Fecha**: 2026-09-14 · *(antes `ADR-007`, renumerado por el pivote)*

## Contexto

Hay operaciones que no necesitan la TUI: consultar estado, extender una sesión, generar reportes, importar datos y controlar el ciclo de vida del daemon. Además, `hypridle` necesita un **punto de entrada ejecutable** para reportar eventos de presencia (`ADR-008`).

## Decisión

`tpt-cli` es un cliente IPC delgado, con subcomandos agrupados por dominio:

| Grupo | Comandos |
|-------|----------|
| Estado | `tpt-cli status` |
| Sesiones | `tpt-cli session start --activity <id> --mode <MODE>` · `extend --minutes N` · `abort` · `switch --activity <id>` · `break skip` |
| Actividades | `tpt-cli activity list` · `new` · `done <id>` · `archive <id>` |
| Notas | `tpt-cli notes --range <d\|w\|m>` |
| Reportes | `tpt-cli report --range daily\|weekly\|monthly --format json\|csv` |
| Import | `tpt-cli import --super-productivity <save.json> [--dry-run]` |
| **Presencia** | `tpt-cli presence --state idle\|active\|locked\|unlocked\|resumed` ← **hook para `hypridle`** |
| **Daemon** | `tpt-cli daemon start` · `stop` · `restart` · `status` |

### Ciclo de vida del daemon

- `tpt-cli daemon stop` pide al daemon un **apagado ordenado**: persiste el estado de la sesión activa, cierra el socket y sale.
- Si hay una sesión `RUNNING`, el daemon la marca **`ABORTED_PENALIZED` con `aborted_with = DAEMON_KILLED`** — apagar el daemon no es una vía de escape.
- La **TUI expone la misma acción** (`[C]onfig › Apagar daemon`) para no obligar a abrir otra terminal.

## Alternativas consideradas

- **Fusionar CLI dentro de la TUI**: obliga a levantar la TUI para automatizar o para que `hypridle` reporte. Descartada.
- **Scripts shell sueltos**: sin validación, sin tipado, sin tests. Descartada.

## Consecuencias

- `hypridle` reporta presencia invocando un binario estable, sin acoplarse a la TUI.
- El reporte y el import se pueden automatizar (cron, scripts) sin abrir interfaz.
- El daemon es apagable desde terminal **y** desde la TUI, siempre de forma ordenada.

## Referencias

- `ADR-001` (IPC) · `ADR-008` (presencia) · `docs/architecture.md`

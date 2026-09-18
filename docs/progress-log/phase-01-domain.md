# Fase 1: Pivote documental + dominio, config y puertos — Log

> Log HISTÓRICO de la fase. Se acumula, no se borra.
> Estado actual → `docs/handoff.md` · Conceptos → `docs/learning/phase-01-domain.md`

## Estado

**Estado**: En Progreso
**Última Actualización**: 2026-09-14

## Objetivos

- Alinear toda la documentación con el pivote *timer-first*.
- Dejar en `tpt-core` los modelos del dominio, la validación de config y los puertos.

## Progreso

- [x] Feature 1.0 — Pivote documental (2026-09-14)
- [ ] Feature 1.1 — Configuración TOML
- [ ] Feature 1.2 — Validación de presets y niveles
- [ ] Feature 1.3 — Actividades, proyectos y agenda
- [ ] Feature 1.4 — Sesiones y entradas de tiempo
- [ ] Feature 1.5 — Puertos

## Tareas Completadas

### 2026-09-14 — Feature 1.0: Pivote documental

- **Descripción**: Reescritura completa de la documentación para el alcance *timer-first*. Se abandonó el tracking pasivo de ventanas.
- **Reescritos**: `docs/vision.md`, `docs/phases.md`, `docs/architecture.md`, `docs/index.md`, `docs/handoff.md`, `README.md`, `AGENTS.md`, `config.toml`.
- **ADRs reestructurados**: 001 (arquitectura), 002 (esquema SQLite), 003 (puertos), 004 (contrato y penalizaciones). **Nuevos**: 005 (tpt-cli), 006 (branching), 007 (pivote), 008 (presencia).
- **Documentos eliminados**: ADRs de blocklist, retención/BTRFS y capas de tracking; `docs/development-plan.md` (fusionado); `docs/diagrams/`, `docs/learning/` y `docs/progress-log/` READMEs (duplicaban índices).
- **Tests**: N/A (fase de documentación).

## Decisiones

1. **Sin tracking de apps** — el compromiso pasa a ser autoimpuesto y el timer se vuelve el centro. Registrado en `ADR-007`.
2. **Presencia vía hooks de `caelestia-shell` (quickshell) + salto de reloj** — cero dependencias nuevas; se descarta D-Bus porque **nadie setea** `IdleHint`/`LockedHint` en esta sesión (`ADR-008`). El **bloqueo de pantalla se ignora**; la señal severa es la **pantalla apagada** (`dpms off`).
3. **Contrato aditivo** — una sesión solo puede extenderse, nunca acortarse (`ADR-004`).
4. **SQLite sin tiers** — sin flood de eventos, las métricas se calculan on-the-fly (`ADR-002`).
5. **Contrato de compromiso en la DB con checksum**, no en `config.toml`, que es editable a mano (`ADR-004`).
6. **Renumeración de ADRs** en vez de acumular ADRs superados, por ser un proyecto pre-código en reestructuración.

## Problemas

1. **`dev` quedó vacía de Fase 0** — se creó desde `main` y los commits de Fase 0 viven en la rama remota `chore/phase-0-setup`. Pendiente llevarlos a `dev`.
2. **ADRs del diseño viejo dispersos** — se resolvió renumerando, en lugar de cargar el repo con ADRs "Reemplazado por…" para decisiones que nunca llegaron a implementarse.

## Pendientes

- Features 1.1 a 1.5 (dominio, config y puertos).
- Fórmula de **Focus Quality** y umbral de "día trabajado" → Fase 8.

### 2026-09-17 — Revisión de presencia: quickshell reemplaza a hypridle

- **Descripción**: Se auditó el entorno real. El lock y el idle los maneja **`caelestia-shell`** (quickshell), no `hypridle` (instalado, sin configurar) ni `hyprlock`. Se reescribió `ADR-008` y se propagó el cambio a `ADR-001/002/003/004/005/007`, `architecture.md`, `vision.md`, `phases.md`, `config.toml`, `README.md`, `AGENTS.md` y el learning de la fase.
- **Cambios clave**: la señal severa pasa de **bloqueo de pantalla** a **pantalla apagada** (`dpms off`); el lock se **ignora**; el adaptador pasa a `QuickshellHookSource`; `session_gaps.kind` y `aborted_with` usan `SCREEN_OFF`.
- **Sistema**: `~/.config/caelestia/shell.json` → lock 5 min, idle TPT 7 min, `dpms off` 12 min, suspend 15 min; `inhibitWhenAudio` global en `false` con override por entrada para que el audio no apague el hook de TPT.
- **Hallazgos**: `CanIdle`/`CanLock` están en `yes` pero **nadie setea** `IdleHint`/`LockedHint`; el lock **no** inhibe el idle; el swap es **zram**, así que `suspend-then-hibernate` no puede hibernar (se cambió a `suspend` plano).
- **Tests**: N/A (docs + config del sistema).

### 2026-09-14 — Diseño del contrato (`ADR-004` + `ADR-002`)

- **Descripción**: Cierre del diseño de sesiones y penalizaciones. Reescritos `ADR-004` y `ADR-002`; actualizados `config.toml`, `docs/phases.md` (Fases 4 y 5), `docs/vision.md` y `docs/handoff.md`.
- **Decisiones**: gap a **7 min**; `L1` aborta a **35%** y `L2` a **25%** (pantalla apagada = aborto inmediato); aborto **manual conserva crédito**, inactividad lo lleva a 0 solo en `L2` severo; **deuda de reparación** 5/8 min pagada con foco 1:1, +5 min/día, caducidad 15/7 días, cobro como entrada negativa; **refinanciación** = `+N%` de lo acreditado, acumulable con tope 40%; `L1`/`L2` son **presets de solo lectura** con `params_snapshot` + checksum; **Pomodoro** con N ciclos y salida libre en descanso o borde de ciclo.
- **Tests**: N/A (diseño).

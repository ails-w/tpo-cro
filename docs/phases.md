# Plan de Fases — TPT (por Feature)

> Índice maestro del desarrollo. La fase activa tiene detalle expandido.
> Conceptos → `docs/learning/phase-NN-name.md` · Log → `docs/progress-log/phase-NN-name.md`
> Estado mutable → `docs/handoff.md`

## Reglas globales

- TDD estricto: RED → GREEN → REFACTOR (test ANTES de implementar).
- Código en inglés, docs en español.
- Commits convencionales en inglés (feat:, fix:, test:, docs:, refactor:, chore:).
- **Commit después de cada FEATURE** · **PR después de cada FASE terminada**.
- `docs/learning/` y `docs/progress-log/` se escriben MIENTRAS se avanza la fase (justo-a-tiempo).
- Al cerrar una fase: actualizar log + learning + handoff + commit.

## Resumen de fases

| # | Feature (fase) | Estado | Conceptos | Log |
|---|----------------|--------|-----------|-----|
| 0 | Setup: workspace + CI | ✅ | `learning/phase-00-setup.md` | `progress-log/phase-00-setup.md` |
| 1 | Configuración y dominio (puertos) | ⏳ | `learning/phase-01-config.md` | `progress-log/phase-01-config.md` |
| 2 | Store SQLite (esquema, WAL, retención) | ⏳ | `learning/phase-02-store.md` | `progress-log/phase-02-store.md` |
| 3 | Captura y categorización (Hyprland, $PWD, regex) | ⏳ | `learning/phase-03-capture.md` | `progress-log/phase-03-capture.md` |
| 4 | Inactividad y blocklist (enforcement) | ⏳ | `learning/phase-04-inactivity.md` | `progress-log/phase-04-inactivity.md` |
| 5 | Motor de relojes (Pomodoro/Focus/Flowtime) | ⏳ | `learning/phase-05-timers.md` | `progress-log/phase-05-timers.md` |
| 6 | Protocolo IPC versionado | ⏳ | `learning/phase-06-ipc.md` | `progress-log/phase-06-ipc.md` |
| 7 | TUI Dashboard | ⏳ | `learning/phase-07-tui.md` | `progress-log/phase-07-tui.md` |
| 8 | Analíticas y reportes (CLI) | ⏳ | `learning/phase-08-analytics.md` | `progress-log/phase-08-analytics.md` |
| 9 | Rutinas, rachas e import | ⏳ | `learning/phase-09-routines.md` | `progress-log/phase-09-routines.md` |
| 10 | Ops, packaging y pulido | ⏳ | `learning/phase-10-ops.md` | `progress-log/phase-10-ops.md` |

---

## Fase 0 — Setup: workspace + CI ✅

**Objetivo:** dejar lista la infraestructura del workspace Rust y el pipeline de CI.

### Scope

- `Cargo.toml` workspace con crates skeleton (`lib` + `bin`): `tpt-core`, `tpt-daemon`, `tpt-tui`, `tpt-cli`.
- Rust estable (edition 2024, resolver 3, MSRV 1.85). Sin `rust-toolchain.toml` (no hay `rustup` en el entorno).
- `.github/workflows/ci.yml` (fmt + clippy + test).
- Convenciones de `Cargo.toml` por crate (edition, lints de workspace).

### Fuera de scope

- Lógica de negocio, configuración, DB, IPC.

### Conceptos de aprendizaje

- [x] Workspace multi-crate en Rust → `docs/learning/phase-00-setup.md`
- [x] `cargo clippy --all-targets -- -D warnings` como puerta → `docs/learning/phase-00-setup.md`
- [x] Lints de crate y edition → `docs/learning/phase-00-setup.md`

### Criterio de salida

- [x] `cargo build` compila los 4 crates.
- [x] `cargo test` corre al menos un test de humo por crate.
- [x] CI verde (fmt + clippy + test).

### Features (TDD)

#### Feature 0.1: Workspace
- [x] Escribir test: `tpt_core_smoke_lib_exposes_version` (RED)
- [x] Crear `Cargo.toml` workspace + 4 crates skeleton (GREEN)
- [x] Refactorizar

#### Feature 0.2: CI
- [x] Crear `.github/workflows/ci.yml` (GREEN)
- [x] Verificar local: `cargo fmt --check` + `cargo clippy --all-targets -- -D warnings` + `cargo test`

---

## Fase 1 — Configuración y dominio (puertos) ⏳

**Objetivo:** modelos del dominio y puertos (traits) en `tpt-core`; parsing/validación de `config.toml`.

### Scope

- Modelos: `AppConfig`, `Category`, `Rule`, `TimerPreset`, `BlockedApp`, `FocusSession`, `Task`.
- Parsing TOML con defaults y validación (presets, umbrales idle, enforcement, retención).
- Puertos (traits): `WindowSource`, `IdleSource`, `Clock`, `Store`, `ConfigSource`, `Notifier`, `IpcTransport`.
- Errores tipados con `thiserror`.

### Fuera de scope

- Implementación de adaptadores OS, DB, IPC.

### Conceptos de aprendizaje

- [ ] Hexagonal: puertos vs adaptadores → `docs/learning/phase-01-config.md`
- [ ] TOML + serde round-trip y defaults → `docs/learning/phase-01-config.md`
- [ ] Errores tipados con `thiserror` → `docs/learning/phase-01-config.md`

### Criterio de salida

- [ ] `config.toml` parsea con defaults; config inválida falla con error tipado.
- [ ] Presets validados: corto ≤15, largo ≤20.
- [ ] Todos los puertos definidos y compilando.

### Features (TDD)

#### Feature 1.1: Modelos de configuración
- [ ] Test: `app_config_parse_valid_toml_returns_expected` (RED)
- [ ] Modelos + parsing (GREEN)

#### Feature 1.2: Validación de presets
- [ ] Test: `timer_preset_short_break_over_15_rejected` (RED)
- [ ] Validación (GREEN)

#### Feature 1.3: Puertos (traits)
- [ ] Test: `window_source_event_shape_compiles` (RED)
- [ ] Definir traits en `ports/` (GREEN)

---

## Fase 2 — Store SQLite ⏳

**Objetivo:** esquema SQLite, WAL, hilo escritor MPSC y retención modular.

### Scope

- Esquema (ver `ADR-002-sqlite-schema.md`): categorías, tareas, blocklist, logs, sesiones, agregados.
- Migración/versión de esquema.
- Hilo escritor MPSC (no bloquea captura).
- WAL + checkpoint + `auto_vacuum=INCREMENTAL`.
- Retención por tiers (raw/hourly/daily) y agregación.

### Fuera de scope

- Captura, IPC, enforcement.

### Conceptos de aprendizaje

- [ ] `rusqlite` + WAL y escritor dedicado → `docs/learning/phase-02-store.md`
- [ ] Migraciones y versionado de esquema → `docs/learning/phase-02-store.md`
- [ ] Agregados y retención por tiers → `docs/learning/phase-02-store.md`

### Criterio de salida

- [ ] Store inicializa esquema en DB temporal.
- [ ] Escrituras por MPSC no bloquean al caller.
- [ ] Retención purga raw > N días y consolida hourly/daily.

### Features (TDD)

#### Feature 2.1: Esquema
- [ ] Test: `store_creates_schema_and_indexes` (RED)
- [ ] `SqliteStore::open` + migración (GREEN)

#### Feature 2.2: Escritor MPSC
- [ ] Test: `store_writer_persists_event_without_blocking` (RED)
- [ ] Hilo escritor (GREEN)

#### Feature 2.3: Retención
- [ ] Test: `retention_purges_raw_keeps_daily` (RED)
- [ ] Mantenimiento de retención (GREEN)

---

## Fase 3 — Captura y categorización ⏳

**Objetivo:** adaptador Hyprland (ventana activa + $PWD), motor de reglas regex y categorizador.

### Scope

- `HyprlandWindowSource` (socket2): class, title, pid, pwd (`/proc/<pid>/cwd`).
- `RuleEngine`: evaluación secuencial de reglas (class/title/pwd) → categoría.
- `Categorizer` con `is_productive`.
- `SystemClock` (monotónico + wall-clock; D5).

### Fuera de scope

- Idle, IPC, enforcement, persistencia de alto nivel.

### Conceptos de aprendizaje

- [ ] Socket IPC de Hyprland y parsing de eventos → `docs/learning/phase-03-capture.md`
- [ ] Lectura `/proc/<pid>/cwd` y symlinks → `docs/learning/phase-03-capture.md`
- [ ] `regex` en Rust y diseño de reglas → `docs/learning/phase-03-capture.md`

### Criterio de salida

- [ ] Con `WindowSource` fake, los eventos se categorizan correctamente.
- [ ] `$PWD` con repos anidados/symlinks se canonicaliza.
- [ ] Sin Hyprland activo, degradación elegante (sin panics).

### Features (TDD)

#### Feature 3.1: Motor de reglas
- [ ] Test: `rule_engine_class_regex_assigns_category` (RED)
- [ ] `RuleEngine` (GREEN)
- [ ] Test: `rule_engine_pwd_prefix_wins_over_generic` (RED)

#### Feature 3.2: Ventana activa
- [ ] Test: `hyprland_source_parses_activewindow_event` (RED)
- [ ] Adapter + `/proc` cwd (GREEN)

#### Feature 3.3: Canonicalización
- [ ] Test: `canonicalize_pwd_resolves_symlink` (RED)
- [ ] Helper (GREEN)

---

## Fase 4 — Inactividad y blocklist ⏳

**Objetivo:** enforcement por blocklist por tarea y manejo de inactividad sin dañar métricas.

### Scope

- `IdleSource` (sin libwayland; helper/fallback): señal de input idle.
- Aviso informativo tras `warn_after_seconds` (default 300 s) — nunca penaliza por sí solo.
- Clasificación del gap como `idle/offline` (no productivo).
- Blocklist por tarea: app bloqueada → aviso → countdown 3–5 min → penalización por modo.
- Máx 2 avisos/sesión; switch de ventanas libre.
- Suspensión: gap = idle/offline (D5).

### Fuera de scope

- TUI final de modales (Fase 7); motor de relojes completo (Fase 5).

### Conceptos de aprendizaje

- [ ] `ext-idle-notify` sin linkear libwayland → `docs/learning/phase-04-inactivity.md`
- [ ] Máquina de estados de enforcement (aviso → countdown → penalización) → `docs/learning/phase-04-inactivity.md`
- [ ] Tiempo monotónico y suspensión → `docs/learning/phase-04-inactivity.md`

### Criterio de salida

- [ ] Idle prolongado no altera métricas (solo registra offline).
- [ ] App bloqueada dispara aviso → countdown → penalización según modo.
- [ ] Máx 2 avisos por sesión.

### Features (TDD)

#### Feature 4.1: Idle informativo
- [ ] Test: `idle_source_warns_after_threshold_without_penalty` (RED)
- [ ] `IdleSource` + aviso (GREEN)

#### Feature 4.2: Blocklist
- [ ] Test: `blocked_app_triggers_warning_then_countdown` (RED)
- [ ] Enforcement blocklist (GREEN)
- [ ] Test: `blocked_app_closed_within_grace_no_penalty` (RED)

#### Feature 4.3: Suspensión
- [ ] Test: `clock_gap_on_resume_treated_as_idle` (RED)
- [ ] Manejo de gap (GREEN)

---

## Fase 5 — Motor de relojes ⏳

**Objetivo:** state machine de Pomodoro, Focus y Flowtime con presets, penalizaciones, challenge, reflexión y cooldown.

### Scope

- **Pomodoro:** sin pausa; `+`/`-` (step/límites); abortar = reinicio de ciclos + sin racha + reflexión obligatoria + challenge; completado solo con tiempo productivo ≥ target.
- **Focus:** contrato inmutable; abortar = crédito 0 + cooldown 5 min + challenge + reflexión obligatoria + impacto en métricas.
- **Flowtime:** sesión variable, descanso proporcional, sin penalización.
- Presets editables: `25/5`, `50/10`, `52/15`, `90/15(20)`; corto ≤15, largo ≤20 (52/17 clampado a 15).
- Switch de tareas programadas en Pomodoro y Focus (sin penalización).
- Cooldown 5 min visible (estado expuesto).
- Estados: `COMPLETED`, `ABORTED_PENALIZED`, `ABORTED`, `IN_PROGRESS`, `SUSPENDED_IDLE`.

### Fuera de scope

- TUI, IPC.

### Conceptos de aprendizaje

- [ ] State machine en Rust (enum + transiciones) → `docs/learning/phase-05-timers.md`
- [ ] Loss aversion / commitment device aplicado a software → `docs/learning/phase-05-timers.md`
- [ ] Persistencia de estado de sesión → `docs/learning/phase-05-timers.md`

### Criterio de salida

- [ ] Los 3 modos completan/abortan con las transiciones correctas.
- [ ] Aborto Focus: crédito 0, cooldown 5 min, challenge + reflexión requeridos.
- [ ] Switch de tarea no penaliza.

### Features (TDD)

#### Feature 5.1: Estado base
- [ ] Test: `session_start_in_progress_persists` (RED)
- [ ] `SessionState` + Store (GREEN)

#### Feature 5.2: Pomodoro
- [ ] Test: `pomodoro_abort_resets_cycles_no_metric_penalty` (RED)
- [ ] Máquina Pomodoro (GREEN)
- [ ] Test: `pomodoro_adjust_plus_minus_bounded` (RED)

#### Feature 5.3: Focus contrato
- [ ] Test: `focus_abort_credits_zero_and_sets_cooldown` (RED)
- [ ] Máquina Focus + cooldown (GREEN)
- [ ] Test: `focus_immutable_rejects_adjust` (RED)

#### Feature 5.4: Flowtime
- [ ] Test: `flowtime_break_proportional_to_block` (RED)
- [ ] Máquina Flowtime (GREEN)

#### Feature 5.5: Challenge + reflexión
- [ ] Test: `abort_requires_challenge_and_reflection` (RED)
- [ ] Challenge + reflexión obligatoria (GREEN)

#### Feature 5.6: Switch de tarea
- [ ] Test: `task_switch_mid_session_no_penalty` (RED)
- [ ] Re-asignación de tarea (GREEN)

---

## Fase 6 — Protocolo IPC versionado ⏳

**Objetivo:** protocolo UDS versionado, servidor en daemon y clientes TUI/CLI.

### Scope

- Framing longitud + JSON, `protocol_version` + `min_supported`.
- `IpcServer` en daemon; socket `0600` en `$XDG_RUNTIME_DIR/tpt.sock`.
- `IpcClient` compartido (TUI/CLI).
- Mensajes: status, focus_start/abort, timer_adjust, task_switch, pomodoro_*, flowtime_*, blocklist_event, ok, error.
- Mismatch de versión manejado con error claro.

### Fuera de scope

- Dashboard TUI (Fase 7), analíticas (Fase 8).

### Conceptos de aprendizaje

- [ ] Unix domain sockets y permisos → `docs/learning/phase-06-ipc.md`
- [ ] Framing de longitud y protocolo versionado → `docs/learning/phase-06-ipc.md`

### Criterio de salida

- [ ] Handshake con versión compatible; mismatch devuelve error tipado.
- [ ] Comandos de control llegan al motor y responden `ok`/`error`.

### Features (TDD)

#### Feature 6.1: Protocolo
- [ ] Test: `ipc_handshake_negotiates_version` (RED)
- [ ] Framing + handshake (GREEN)
- [ ] Test: `ipc_version_mismatch_returns_error` (RED)

#### Feature 6.2: Servidor
- [ ] Test: `ipc_server_routes_status_and_focus` (RED)
- [ ] `IpcServer` (GREEN)

#### Feature 6.3: Cliente
- [ ] Test: `ipc_client_sends_and_parses` (RED)
- [ ] `IpcClient` (GREEN)

---

## Fase 7 — TUI Dashboard ⏳

**Objetivo:** vistas Ratatui con timeline, barras, focus score, switches y timer con cooldown visible.

### Scope

- Vistas: Dashboard (timeline 24h, barras por categoría/proyecto), Timer (Pomodoro/Focus/Flowtime + presets + `+`/`-`), Analytics (D/S/M), Rutinas.
- Focus Score (0–100, fórmula configurable).
- Context switch frequency por hora.
- Modales: aviso idle, aviso blocklist, challenge, reflexión, clasificación offline.
- Cooldown 5 min visible.
- Navegación y atajos de teclado.

### Fuera de scope

- Agregados/analíticas avanzadas (Fase 8), import (Fase 9).

### Conceptos de aprendizaje

- [ ] Ratatui: layout, widgets, eventos → `docs/learning/phase-07-tui.md`
- [ ] Componentes testables (render con datos fake) → `docs/learning/phase-07-tui.md`

### Criterio de salida

- [ ] TUI arranca, navega y renderiza con datos reales (daemon) o fakes.
- [ ] Cooldown y modales de enforcement funcionan.
- [ ] Tests de componentes verdes.

### Features (TDD)

#### Feature 7.1: Esqueleto TUI
- [ ] Test: `app_creates_main_window` (RED)
- [ ] Orquestación Ratatui (GREEN)

#### Feature 7.2: Dashboard
- [ ] Test: `dashboard_renders_timeline_and_bars` (RED)
- [ ] Vistas dashboard (GREEN)

#### Feature 7.3: Timer
- [ ] Test: `timer_view_adjust_and_cooldown_visible` (RED)
- [ ] Vista timer + modales (GREEN)

#### Feature 7.4: Focus score
- [ ] Test: `focus_score_formula_configurable` (RED)
- [ ] Cálculo + render (GREEN)

---

## Fase 8 — Analíticas y reportes ⏳

**Objetivo:** agregados persistentes, vistas D/S/M y `tpt-cli report`.

### Scope

- Agregados diarios/semanales/mensuales desde `daily_aggregates`/`hourly_aggregates`.
- Métricas: horas invertidas (observado y acreditado), días trabajados, promedios por día trabajado Y por día calendario, productivo vs distracción, media de sesión, switches, racha, abortadas.
- Vistas de TUI para D/S/M.
- `tpt-cli report --range weekly|monthly --format json|csv`.

### Fuera de scope

- Import (Fase 9).

### Conceptos de aprendizaje

- [ ] Queries de agregación SQL → `docs/learning/phase-08-analytics.md`
- [ ] Export JSON/CSV estable y versionado → `docs/learning/phase-08-analytics.md`

### Criterio de salida

- [ ] Métricas correctas en D/S/M con ambos denominadores.
- [ ] `tpt-cli report` genera json/csv sin abrir la TUI.

### Features (TDD)

#### Feature 8.1: Agregados
- [ ] Test: `analytics_daily_aggregates_from_raw` (RED)
- [ ] Rollup raw → hourly → daily (GREEN)

#### Feature 8.2: Métricas
- [ ] Test: `analytics_averages_worked_day_and_calendar` (RED)
- [ ] Queries de métricas (GREEN)

#### Feature 8.3: Report CLI
- [ ] Test: `cli_report_weekly_json_valid` (RED)
- [ ] Comando `report` (GREEN)

#### Feature 8.4: Vistas D/S/M
- [ ] Test: `analytics_view_renders_range` (RED)
- [ ] Vistas en TUI (GREEN)

---

## Fase 9 — Rutinas, rachas e import ⏳

**Objetivo:** rutinas diarias auto-generadas, habit streak e import de Super Productivity.

### Scope

- Rutinas diarias (barra `X/Y` min combinando tracking pasivo).
- Habit streak mensual (sesiones abortadas no acreditan).
- `tpt-cli import --super-productivity save.json`: validación, `--dry-run`, idempotencia/dedup.

### Fuera de scope

- Nuevas features de analíticas.

### Conceptos de aprendizaje

- [ ] Modelado de rutinas recurrentes y streaks → `docs/learning/phase-09-routines.md`
- [ ] Parsing/adaptación de esquemas externos → `docs/learning/phase-09-routines.md`

### Criterio de salida

- [ ] Rutinas se auto-generan y progresan con el tracking.
- [ ] Streak correcto con días no acreditados.
- [ ] Import idempotente con `--dry-run`.

### Features (TDD)

#### Feature 9.1: Rutinas
- [ ] Test: `routine_auto_generates_daily_with_progress` (RED)
- [ ] Rutinas (GREEN)

#### Feature 9.2: Streak
- [ ] Test: `streak_excludes_aborted_days` (RED)
- [ ] Streak mensual (GREEN)

#### Feature 9.3: Import
- [ ] Test: `import_sp_json_maps_and_dedups` (RED)
- [ ] Adapter import + `--dry-run` (GREEN)

---

## Fase 10 — Ops, packaging y pulido ⏳

**Objetivo:** robustez, packaging Arch, BTRFS y documentación final.

### Scope

- Manejo de errores y degradación elegante (daemon caído, config corrupta).
- Unit systemd + PKGBUILD.
- BTRFS: nodatacow, checkpoints, retención validada.
- Cobertura y documentación final.

### Fuera de scope

- Nuevas features.

### Conceptos de aprendizaje

- [ ] Empaquetado Arch (PKGBUILD) y systemd → `docs/learning/phase-10-ops.md`
- [ ] Cobertura y casos borde → `docs/learning/phase-10-ops.md`

### Criterio de salida

- [ ] Suite completa verde; cobertura >80% unit, >60% integración.
- [ ] Instalación limpia en Arch (daemon + TUI + CLI).
- [ ] Docs finalizadas (README, architecture, development-plan).

### Features (TDD)

#### Feature 10.1: Errores
- [ ] Test: `daemon_down_client_degrades_gracefully` (RED)
- [ ] Degradación elegante (GREEN)
- [ ] Test: `config_corrupt_falls_back_to_defaults` (RED)

#### Feature 10.2: Empaquetado
- [ ] Unit systemd + PKGBUILD (GREEN)
- [ ] Instalación y arranque verificados

#### Feature 10.3: BTRFS final
- [ ] Verificar nodatacow + checkpoint + retención en disco real (GREEN)

#### Feature 10.4: Suite y docs
- [ ] Cobertura objetivo alcanzada
- [ ] README, architecture, development-plan finalizados
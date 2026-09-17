# Plan de Fases — TPT (por Feature)

> Índice maestro del desarrollo. **La fase activa tiene detalle expandido**; las siguientes, resumen.
> Conceptos → `docs/learning/phase-NN-name.md` · Log → `docs/progress-log/phase-NN-name.md`
> Estado mutable → `docs/handoff.md`

## Reglas globales

- TDD estricto: RED → GREEN → REFACTOR (test ANTES de implementar).
- Código en inglés, docs en español.
- Commits convencionales en inglés (feat:, fix:, test:, docs:, refactor:, chore:).
- **Commit después de cada FEATURE** · **PR después de cada FASE terminada**.
- `docs/learning/` y `docs/progress-log/` se escriben MIENTRAS se avanza la fase (justo-a-tiempo).
- Al cerrar una fase: actualizar log + learning + handoff + commit.

### Estructura de cada fase

Toda fase se documenta con las mismas secciones, sin excepción:

1. **Objetivo** — qué entrega la fase en una línea.
2. **Scope** — qué se construye.
3. **Fuera de scope** — qué NO se toca (evita el scope creep).
4. **Conceptos de aprendizaje** — qué se aprende; alimenta `docs/learning/`.
5. **Criterio de salida** — cómo se sabe que la fase terminó (todo verificable).
6. **Features (TDD)** — los tests que definen cada feature, en orden RED → GREEN.

## Resumen de fases

| # | Feature (fase) | Estado | Conceptos | Log |
|---|----------------|--------|-----------|-----|
| 0 | Setup: workspace + CI | ✅ | `learning/phase-00-setup.md` | `progress-log/phase-00-setup.md` |
| 1 | Pivote documental + dominio, config y puertos | ⏳ | `learning/phase-01-domain.md` | `progress-log/phase-01-domain.md` |
| 2 | Store SQLite | ⏳ | `learning/phase-02-store.md` | `progress-log/phase-02-store.md` |
| 3 | Actividades, proyectos, tags, agenda y cola | ⏳ | `learning/phase-03-activities.md` | `progress-log/phase-03-activities.md` |
| 4 | Motor de sesiones y contrato | ⏳ | `learning/phase-04-sessions.md` | `progress-log/phase-04-sessions.md` |
| 5 | Presencia, niveles y deuda | ⏳ | `learning/phase-05-presence.md` | `progress-log/phase-05-presence.md` |
| 6 | Protocolo IPC versionado | ⏳ | `learning/phase-06-ipc.md` | `progress-log/phase-06-ipc.md` |
| 7 | TUI | ⏳ | `learning/phase-07-tui.md` | `progress-log/phase-07-tui.md` |
| 8 | Métricas y reportes | ⏳ | `learning/phase-08-analytics.md` | `progress-log/phase-08-analytics.md` |
| 9 | Rutinas, rachas e import | ⏳ | `learning/phase-09-routines.md` | `progress-log/phase-09-routines.md` |
| 10 | Ops, packaging y pulido | ⏳ | `learning/phase-10-ops.md` | `progress-log/phase-10-ops.md` |

---

## Fase 0 — Setup: workspace + CI ✅

**Objetivo:** infraestructura del workspace Rust y pipeline de CI.

### Scope

- `Cargo.toml` workspace con crates skeleton (`lib` + `bin`): `tpt-core`, `tpt-daemon`, `tpt-tui`, `tpt-cli`.
- Rust estable (edition 2024, resolver 3, MSRV 1.85). Sin `rust-toolchain.toml` (no hay `rustup` en el entorno).
- `.github/workflows/ci.yml` (fmt + clippy + test).
- Convenciones de `Cargo.toml` por crate (edition, lints de workspace).

### Fuera de scope

- Lógica de negocio, configuración, base de datos, IPC, TUI.

### Conceptos de aprendizaje

- [x] Workspace multi-crate en Rust → `docs/learning/phase-00-setup.md`
- [x] `cargo clippy --all-targets -- -D warnings` como puerta → `docs/learning/phase-00-setup.md`
- [x] Lints de crate y edition → `docs/learning/phase-00-setup.md`

### Criterio de salida

- [x] `cargo build` compila los 4 crates.
- [x] `cargo test` corre al menos un test de humo por crate.
- [x] CI verde (fmt + clippy + test).

### Features (TDD)

- [x] `tpt_core_smoke_lib_exposes_version` (RED → GREEN)
- [x] `tpt_daemon_smoke_lib_exposes_version` (RED → GREEN)
- [x] `tpt_tui_smoke_lib_exposes_version` (RED → GREEN)
- [x] `tpt_cli_smoke_lib_exposes_version` (RED → GREEN)

---

## Fase 1 — Pivote documental + dominio, config y puertos ⏳

**Objetivo:** alinear toda la documentación con el pivote *timer-first* (`ADR-007`) y dejar en `tpt-core` los modelos, la validación de config y los puertos.

### Scope

- **Feature 1.0 (docs):** reescritura de `vision.md`, `phases.md`, `architecture.md`, `AGENTS.md`, `README.md`, `config.toml`, `index.md`; ADRs 001–008; retiro de los ADRs de tracking/blocklist/retención.
- **Modelos:** `AppConfig`, `TimerPreset`, `StrictnessLevel`, `TrackingMode`, `Activity`, `Project`, `Tag`, `ScheduleRule`, `TimeEntry` (`Manual`/`Session`/`Imported`/`Penalty`), `Session`, `SessionMode`, `SessionStatus`, `Reflection`, `Debt`, `CommitmentContract`.
- **Parsing TOML** con defaults y validación (presets, umbrales de presencia, niveles, retención de notas).
- **Puertos:** `Clock`, `PresenceSource`, `Store`, `ConfigSource`, `Notifier`, `IpcTransport`.
- **Errores tipados** con `thiserror`.

### Fuera de scope

- Persistencia real, adaptadores de OS, IPC, TUI.

### Conceptos de aprendizaje

- [ ] Hexagonal: puertos vs adaptadores en un dominio sin SO → `docs/learning/phase-01-domain.md`
- [ ] TOML + serde round-trip y defaults → `docs/learning/phase-01-domain.md`
- [ ] Contrato aditivo como invariante de dominio → `docs/learning/phase-01-domain.md`

### Criterio de salida

- [ ] `config.toml` parsea con defaults; config inválida falla con error tipado.
- [ ] Presets validados: corto ≤15, largo ≤20.
- [ ] Modelos del dominio compilan y sus tests pasan.
- [ ] Los 6 puertos definidos, object-safe y con fake.
- [ ] fmt + clippy limpios.

### Features (TDD)

- [x] Feature 1.0 — Pivote documental (docs + ADRs)
- [ ] `app_config_parse_valid_toml_returns_expected` (RED → GREEN)
- [ ] `timer_preset_short_break_over_15_rejected` (RED → GREEN)
- [ ] `activity_without_schedule_is_valid` (RED → GREEN)
- [ ] `activity_requires_tracking_mode` (RED → GREEN)
- [ ] `session_starts_running_with_activity` (RED → GREEN)
- [ ] `manual_time_entry_adds_to_activity_total` (RED → GREEN)
- [ ] `ports_are_object_safe` (RED → GREEN)

---

## Fase 2 — Store SQLite ⏳

**Objetivo:** implementar el esquema de `ADR-002` con WAL, hilo escritor y retención de notas.

### Scope

- `SqliteStore::open` + migración y versionado de esquema.
- Hilo escritor MPSC (no bloquea el loop de sesión).
- WAL, checkpoint `TRUNCATE`, `auto_vacuum=INCREMENTAL`.
- Retención de **notas** (`notes_retention_days`); sin tiers de eventos.
- Estado derivado por cálculo: crecimiento y vencimiento de deuda sin jobs.
- `chattr +C` documentado y validado en disco real.

### Fuera de scope

- Lógica de contrato y sesión (Fase 4), adaptadores de presencia (Fase 5), IPC (Fase 6).

### Conceptos de aprendizaje

- [ ] `rusqlite` + WAL con escritor dedicado → `docs/learning/phase-02-store.md`
- [ ] Migraciones y versionado de esquema → `docs/learning/phase-02-store.md`
- [ ] Derivar estado por cálculo en vez de escribirlo (sin cron) → `docs/learning/phase-02-store.md`

### Criterio de salida

- [ ] Store inicializa el esquema completo en una DB temporal.
- [ ] Las escrituras por MPSC no bloquean al caller.
- [ ] La retención purga notas antiguas y conserva sesiones y entradas.
- [ ] Deuda: crecimiento y vencimiento se calculan correctamente desde `created_at`.

### Features (TDD)

- [ ] `store_creates_schema_and_indexes` (RED → GREEN)
- [ ] `store_writer_persists_session_without_blocking` (RED → GREEN)
- [ ] `retention_purges_notes_keeps_sessions` (RED → GREEN)
- [ ] `debt_amount_is_computed_from_created_at` (RED → GREEN)
- [ ] `penalty_entry_accepts_negative_seconds` (RED → GREEN)

---

## Fase 3 — Actividades, proyectos, tags, agenda y cola ⏳

**Objetivo:** CRUD de actividades y armado automático de la cola del día.

### Scope

- CRUD de `Activity` / `Project` / `Tag`; archivar y completar.
- `ScheduleRule`: lunes–viernes, días específicos, toda la semana, rango de fechas.
- Cola del día: agenda + pendientes; siguiente actividad al completar.
- Tiempo manual por actividad (alta, edición, borrado), solo en `tracking_mode = MANUAL`.

### Fuera de scope

- Motor de sesiones (Fase 4), métricas (Fase 8), vista TUI (Fase 7).

### Conceptos de aprendizaje

- [ ] Modelado de recurrencia con reglas → `docs/learning/phase-03-activities.md`
- [ ] Derivar la cola del día en vez de almacenarla → `docs/learning/phase-03-activities.md`
- [ ] Archivado sin perder historia → `docs/learning/phase-03-activities.md`

### Criterio de salida

- [ ] Una actividad sin agenda es válida y no entra en la cola.
- [ ] La cola del día respeta las 4 formas de agenda.
- [ ] Completar una actividad la saca de la cola y ofrece la siguiente.
- [ ] El tiempo manual se rechaza en actividades `TIMER`.

### Features (TDD)

- [ ] `schedule_weekdays_matches_only_weekdays` (RED → GREEN)
- [ ] `schedule_range_excludes_dates_outside` (RED → GREEN)
- [ ] `queue_builds_from_schedule_and_pending` (RED → GREEN)
- [ ] `manual_entry_rejected_when_tracking_mode_is_timer` (RED → GREEN)
- [ ] `manual_entry_is_marked_and_sums_to_total` (RED → GREEN)

---

## Fase 4 — Motor de sesiones y contrato ⏳

**Objetivo:** máquina de estados de Flowtime / Pomodoro / Focus con el contrato aditivo y el compromiso congelado.

### Scope

- Estados: `RUNNING`, `BREAK`, `AWAITING`, `SUSPENDED`, `COMPLETED`, `ABORTED`, `ABORTED_PENALIZED`.
- Contrato aditivo: `+` extiende, nunca resta. Sin pausa salvo Flowtime.
- **Pomodoro:** N ciclos explícitos; salida **gratis** en descanso o borde de ciclo; `AWAITING` con confirmación tras el descanso; cierre limpio si no confirma.
- **Focus:** target inmutable, contrato duro.
- **Flowtime:** libre y pausable.
- Switch de actividad a mitad de sesión: parte el tiempo, no reinicia el contrato.
- **Contrato de compromiso:** nivel + término + `params_snapshot` con checksum; el daemon usa el snapshot e ignora `config.toml` mientras esté activo.

### Fuera de scope

- Detección de presencia y deuda (Fase 5), IPC (Fase 6), TUI (Fase 7).

### Conceptos de aprendizaje

- [ ] Máquina de estados en Rust (enum + transiciones) → `docs/learning/phase-04-sessions.md`
- [ ] Hacer irrepresentable el estado inválido (contrato aditivo por tipos) → `docs/learning/phase-04-sessions.md`
- [ ] Congelar configuración con snapshot + checksum → `docs/learning/phase-04-sessions.md`

### Criterio de salida

- [ ] Los 3 modos completan y abortan con las transiciones correctas.
- [ ] Es imposible restar tiempo o pausar Pomodoro/Focus.
- [ ] Un Pomodoro se puede cerrar sin deuda en cualquier borde de ciclo.
- [ ] El snapshot congela los parámetros y resiste ediciones de `config.toml`.

### Features (TDD)

- [ ] `session_extend_only_increases_target` (RED → GREEN)
- [ ] `pomodoro_has_no_pause` (RED → GREEN)
- [ ] `pomodoro_boundary_exit_is_free` (RED → GREEN)
- [ ] `pomodoro_awaits_confirmation_after_break` (RED → GREEN)
- [ ] `pomodoro_awaiting_expiry_closes_completed` (RED → GREEN)
- [ ] `flowtime_pause_stops_credit` (RED → GREEN)
- [ ] `task_switch_mid_session_splits_time` (RED → GREEN)
- [ ] `commitment_contract_blocks_downgrade` (RED → GREEN)
- [ ] `contract_snapshot_ignores_config_changes` (RED → GREEN)

---

## Fase 5 — Presencia, niveles y deuda ⏳

**Objetivo:** adaptadores de `PresenceSource`, escalada de avisos, abortos por nivel, deuda de reparación y refinanciación.

### Scope

- `HypridleHookSource` (comandos `tpt-cli presence`) + `ClockGapSource` (salto monotónico vs wall-clock).
- **Gap a los 7 min**: por debajo no existe; desde ahí se acumula y descuenta.
- Avisos **inmediatos** a los 3 y 5 min (informativos, nunca descuentan).
- Niveles `L1` (aborta 35%) y `L2` (aborta 25%, lock inmediato) como presets de solo lectura.
- Crédito al abortar: manual conserva; inactividad cero solo en L2 severo (lock o gap único ≥30%).
- **Deuda de reparación:** 5/8 min +1 por aborto (topes 20/25), +5 min/día calculado, caducidad 15/7 días, cobro como entrada negativa.
- **Refinanciación:** `pending_extra` acumulable (tope 40%) que se suma al target de la próxima sesión.
- Puntaje obligatorio (1–7 en L1, 1–5 en L2) y notas obligatorias en L2.
- Notificaciones de escritorio + sonido; pantalla de descanso.
- Heartbeat de presencia: sin señal fresca no se arranca en L1/L2.

### Fuera de scope

- Vista TUI de modales y contrato (Fase 7), reportes (Fase 8).

### Conceptos de aprendizaje

- [ ] Detección de presencia sin dependencias (hooks + reloj) → `docs/learning/phase-05-presence.md`
- [ ] Política de inactividad como función pura y testeable → `docs/learning/phase-05-presence.md`
- [ ] Degradación elegante y notificaciones fiables → `docs/learning/phase-05-presence.md`

### Criterio de salida

- [ ] Un gap por debajo de 7 min no descuenta ni acumula; uno mayor descuenta completo.
- [ ] L1 aborta a 35% acumulado; L2 a 25% o al instante con lock.
- [ ] Un aborto manual conserva el crédito pero crea deuda; uno de inactividad puede llevar el crédito a 0.
- [ ] La deuda crece por cálculo (sin job) y al vencer deja una entrada negativa.
- [ ] Refinanciar suma al target de la próxima sesión y no se puede esquivar.
- [ ] Sin `hypridle`, la app funciona (degradación elegante, sin panics).

### Features (TDD)

- [ ] `gap_below_threshold_is_ignored` (RED → GREEN)
- [ ] `gap_above_threshold_deducts_full` (RED → GREEN)
- [ ] `locked_gap_deducts_full_time` (RED → GREEN)
- [ ] `clock_jump_detected_as_suspend_gap` (RED → GREEN)
- [ ] `l1_aborts_at_35_percent_accumulated` (RED → GREEN)
- [ ] `l2_aborts_at_25_percent_and_lock_is_immediate` (RED → GREEN)
- [ ] `l2_severe_abort_zeroes_credit_manual_keeps_it` (RED → GREEN)
- [ ] `debt_grows_five_minutes_per_day_without_job` (RED → GREEN)
- [ ] `debt_expiry_writes_negative_entry` (RED → GREEN)
- [ ] `refinance_accumulates_pending_extra_capped_at_40` (RED → GREEN)
- [ ] `l2_abort_requires_notes` (RED → GREEN)
- [ ] `daemon_kill_marks_session_penalized` (RED → GREEN)

---

## Fase 6 — Protocolo IPC versionado ⏳

**Objetivo:** UDS versionado, servidor en el daemon y clientes TUI/CLI.

### Scope

- Framing longitud + JSON, `protocol_version` + `min_supported`.
- `IpcServer`; socket `0600` en `$XDG_RUNTIME_DIR/tpt.sock`.
- `IpcClient` compartido.
- Mensajes: `status`, `session_*`, `break_skip`, `presence_event`, `activity_*`, `notes_list`, `daemon_stop`, `ok`, `error`.
- Mismatch de versión con error claro.

### Fuera de scope

- Lógica de contrato (Fase 4), dashboard TUI (Fase 7), reportes (Fase 8).

### Conceptos de aprendizaje

- [ ] Unix domain sockets y permisos → `docs/learning/phase-06-ipc.md`
- [ ] Framing de longitud y protocolo versionado → `docs/learning/phase-06-ipc.md`

### Criterio de salida

- [ ] Handshake compatible; mismatch devuelve error tipado.
- [ ] Los comandos de control llegan al motor y responden `ok`/`error`.
- [ ] `hypridle` puede reportar presencia por CLI sin abrir la TUI.

### Features (TDD)

- [ ] `ipc_handshake_negotiates_version` (RED → GREEN)
- [ ] `ipc_version_mismatch_returns_error` (RED → GREEN)
- [ ] `ipc_server_routes_session_start` (RED → GREEN)
- [ ] `ipc_client_sends_and_parses` (RED → GREEN)
- [ ] `daemon_stop_marks_running_session_penalized` (RED → GREEN)

---

## Fase 7 — TUI ⏳

**Objetivo:** interfaz Ratatui eficiente y visualmente atractiva.

### Scope

- Vistas: `[H]oy` (cola + timer mini) · `[T]imer` · `[M]étricas` · `[A]ctividades` · `[I]historial` · `[C]onfig`.
- **Config con dos paneles:** *Ajustes* (editable) y *Contrato activo* (solo lectura, con deuda y % en vivo).
- **Timer:** layout de foco a pantalla completa, **beacon permanente** en todas las vistas, `lock_focus_view` opt-in (3 capas).
- **Contadores en vivo:** `⏱ Focus 31:12 · Inactividad 4:12/17:30 (24%) · Abortos 2 · Deuda 12 min`.
- **Historial:** notas y puntajes agrupados por fecha, filtrables por actividad y rating.
- Modales: reflexión (puntaje + notas según nivel), challenge de refinanciación, `AWAITING` de Pomodoro.
- Acción para apagar el daemon desde la TUI.
- Widgets propios: heatmap y gauge circular.

### Fuera de scope

- Agregados avanzados (Fase 8), import (Fase 9).

### Conceptos de aprendizaje

- [ ] Ratatui: layout, widgets y eventos → `docs/learning/phase-07-tui.md`
- [ ] Componentes testables (render con datos fake) → `docs/learning/phase-07-tui.md`
- [ ] El beacon como contrato visual de presencia → `docs/learning/phase-07-tui.md`

### Criterio de salida

- [ ] La TUI navega y renderiza con datos fake y con el daemon real.
- [ ] El beacon muestra el tiempo restante en toda vista durante una sesión.
- [ ] El panel de contrato muestra los parámetros de L1/L2 bloqueados y legibles.
- [ ] Los modales de reflexión permiten solo nota, solo puntaje, ambos o nada (salvo donde es obligatorio).

### Features (TDD)

- [ ] `app_creates_main_window` (RED → GREEN)
- [ ] `timer_view_shows_beacon_and_blocks_only_if_configured` (RED → GREEN)
- [ ] `contract_panel_renders_preset_locked` (RED → GREEN)
- [ ] `reflection_modal_allows_empty_and_partial_input` (RED → GREEN)
- [ ] `history_groups_notes_by_date` (RED → GREEN)

---

## Fase 8 — Métricas y reportes ⏳

**Objetivo:** agregaciones D/S/M, heatmap, jerarquías y `tpt-cli report`.

### Scope

- Heatmap anual por día (estilo GitHub).
- Línea de sesiones/minutos con rangos `2 semanas` / `1 mes` / `Máx`.
- Jerarquía **Año → Mes → Semana → Día** con rangos horarios y conteo de actividades.
- Totales, días trabajados, meses trabajados, promedios por día trabajado y por día calendario.
- **Focus Quality** (promedio de rating normalizado por `rating_scale`) por actividad y franja horaria.
- Desglose por `source` (sesión / manual / importado) y **línea propia de penalizaciones**.
- Desglose de `aborted_with` (USER vs sistema).
- `tpt-cli report --range daily|weekly|monthly --format json|csv`.

### Fuera de scope

- Rutinas y rachas (Fase 9).

### Conceptos de aprendizaje

- [ ] Queries de agregación SQL → `docs/learning/phase-08-analytics.md`
- [ ] Denominadores: día trabajado vs día calendario → `docs/learning/phase-08-analytics.md`
- [ ] Normalización de puntajes entre escalas → `docs/learning/phase-08-analytics.md`

### Criterio de salida

- [ ] Las métricas cuadran contra un dataset de prueba conocido.
- [ ] Los dos denominadores se muestran por separado.
- [ ] Las penalizaciones aparecen como línea propia, sin ensuciar el total.
- [ ] El reporte JSON/CSV se genera sin abrir la TUI.

### Features (TDD)

- [ ] `analytics_daily_totals_from_sessions_and_entries` (RED → GREEN)
- [ ] `analytics_averages_worked_day_and_calendar` (RED → GREEN)
- [ ] `focus_quality_normalizes_rating_scale` (RED → GREEN)
- [ ] `penalty_entries_render_as_their_own_line` (RED → GREEN)
- [ ] `heatmap_buckets_by_day` (RED → GREEN)
- [ ] `cli_report_weekly_json_valid` (RED → GREEN)

---

## Fase 9 — Rutinas, rachas e import ⏳

**Objetivo:** rachas, consistencia e import de Super Productivity.

### Scope

- Racha con política *never miss twice*; solo un **aborto real** la corta.
- Racha atada al **checkpoint de confirmación**: lo cumplido en ciclos cerrados cuenta siempre.
- Consistencia mensual de rutinas.
- `tpt-cli import --super-productivity save.json`: validación, `--dry-run`, idempotencia por `external_id`.
- Mapeo: `project` → `Project`, `task` → `Activity`, `timeEstimate` → `target_minutes`, `timeSpentOnDay` → `TimeEntry{IMPORTED}`.

### Fuera de scope

- Nuevas features de analíticas (Fase 8).

### Conceptos de aprendizaje

- [ ] Modelado de rachas y "never miss twice" → `docs/learning/phase-09-routines.md`
- [ ] Parsing y adaptación de esquemas externos → `docs/learning/phase-09-routines.md`
- [ ] Idempotencia y dedupe en importaciones → `docs/learning/phase-09-routines.md`

### Criterio de salida

- [ ] La racha sobrevive a un día perdido y se corta a los dos.
- [ ] Salir en un borde de ciclo o en un descanso nunca afecta la racha.
- [ ] Import idempotente: correrlo dos veces no duplica.
- [ ] `--dry-run` reporta el mapeo sin escribir.

### Features (TDD)

- [ ] `streak_survives_one_missed_day` (RED → GREEN)
- [ ] `boundary_exit_does_not_break_streak` (RED → GREEN)
- [ ] `import_sp_json_maps_and_dedups` (RED → GREEN)
- [ ] `import_dry_run_writes_nothing` (RED → GREEN)

---

## Fase 10 — Ops, packaging y pulido ⏳

**Objetivo:** robustez, packaging Arch y documentación final.

### Scope

- Degradación elegante (daemon caído, config corrupta).
- Unit systemd + PKGBUILD.
- BTRFS: nodatacow + checkpoint validados en disco real.
- Medición real de consumo de memoria (sin declarar límites antes).
- Cobertura y documentación final.

### Fuera de scope

- Nuevas features.

### Conceptos de aprendizaje

- [ ] Empaquetado Arch (PKGBUILD) y systemd → `docs/learning/phase-10-ops.md`
- [ ] Medición real de memoria en un daemon → `docs/learning/phase-10-ops.md`
- [ ] Cobertura y casos borde → `docs/learning/phase-10-ops.md`

### Criterio de salida

- [ ] Suite completa verde; cobertura >80% unit, >60% integración.
- [ ] Instalación limpia en Arch (daemon + TUI + CLI).
- [ ] Consumo de memoria medido y documentado.
- [ ] `README.md` y `docs/` finalizados.

### Features (TDD)

- [ ] `daemon_down_client_degrades_gracefully` (RED → GREEN)
- [ ] `config_corrupt_falls_back_to_defaults` (RED → GREEN)
- [ ] Unit systemd + PKGBUILD
- [ ] Verificación en disco real (nodatacow + vencimiento de deuda)

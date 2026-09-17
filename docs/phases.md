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

## Resumen de fases

| # | Feature (fase) | Estado | Conceptos | Log |
|---|----------------|--------|-----------|-----|
| 0 | Setup: workspace + CI | ✅ | `learning/phase-00-setup.md` | `progress-log/phase-00-setup.md` |
| 1 | Pivote documental + dominio, config y puertos | ⏳ | `learning/phase-01-domain.md` | `progress-log/phase-01-domain.md` |
| 2 | Store SQLite | ⏳ | `learning/phase-02-store.md` | `progress-log/phase-02-store.md` |
| 3 | Actividades, proyectos, tags, agenda y cola | ⏳ | `learning/phase-03-activities.md` | `progress-log/phase-03-activities.md` |
| 4 | Motor de sesiones y contrato | ⏳ | `learning/phase-04-sessions.md` | `progress-log/phase-04-sessions.md` |
| 5 | Presencia: idle, lock y suspensión | ⏳ | `learning/phase-05-presence.md` | `progress-log/phase-05-presence.md` |
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

### Criterio de salida

- [x] `cargo build` compila los 4 crates.
- [x] `cargo test` corre al menos un test de humo por crate.
- [x] CI verde (fmt + clippy + test).

---

## Fase 1 — Pivote documental + dominio, config y puertos ⏳

**Objetivo:** alinear toda la documentación con el pivote *timer-first* (`ADR-007`) y dejar en `tpt-core` los modelos, la validación de config y los puertos.

### Scope

- **Feature 1.0 (docs):** reescritura de `vision.md`, `phases.md`, `architecture.md`, `AGENTS.md`, `README.md`, `config.toml`, `index.md`; ADRs 001–008; retiro de los ADRs de tracking/blocklist/retención.
- **Modelos:** `AppConfig`, `TimerPreset`, `StrictnessLevel`, `Activity`, `Project`, `Tag`, `ScheduleRule`, `TimeEntry` (`Manual`/`Session`/`Imported`), `Session`, `SessionMode`, `SessionStatus`, `Reflection`.
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

#### Feature 1.0: Pivote documental
- [ ] Reescribir visión, fases, arquitectura, AGENTS, README, config, index
- [ ] Crear ADRs 001–008 y retirar los obsoletos
- [ ] Borrar/fusionar docs duplicados

#### Feature 1.1: Configuración TOML
- [ ] Test: `app_config_parse_valid_toml_returns_expected` (RED)
- [ ] Modelos + parsing (GREEN)

#### Feature 1.2: Validación de presets y niveles
- [ ] Test: `timer_preset_short_break_over_15_rejected` (RED)
- [ ] Validación (GREEN)

#### Feature 1.3: Actividades, proyectos y agenda
- [ ] Test: `activity_without_schedule_is_valid` (RED)
- [ ] Modelos + `ScheduleRule` (GREEN)

#### Feature 1.4: Sesiones y entradas de tiempo
- [ ] Test: `session_starts_running_with_activity` (RED)
- [ ] Modelos + estados (GREEN)
- [ ] Test: `manual_time_entry_adds_to_activity_total` (RED)

#### Feature 1.5: Puertos
- [ ] Test: `ports_are_object_safe` (RED)
- [ ] Definir traits en `ports/` (GREEN)

---

## Fase 2 — Store SQLite ⏳

**Objetivo:** implementar el esquema de `ADR-002` con WAL, hilo escritor y retención de notas.

### Scope

- `SqliteStore::open` + migración/versionado de esquema.
- Hilo escritor MPSC (no bloquea el loop de sesión).
- WAL, checkpoint, `auto_vacuum=INCREMENTAL`.
- Retención de **notas** (`notes_retention_days`); no hay tiers de eventos.
- `chattr +C` documentado y validado en disco real.

### Criterio de salida

- [ ] Store inicializa el esquema en una DB temporal.
- [ ] Las escrituras por MPSC no bloquean al caller.
- [ ] La retención purga notas antiguas y conserva sesiones/entradas.

### Features (TDD)

#### Feature 2.1: Esquema
- [ ] Test: `store_creates_schema_and_indexes` (RED) → `SqliteStore::open` + migración (GREEN)

#### Feature 2.2: Escritor MPSC
- [ ] Test: `store_writer_persists_session_without_blocking` (RED) → hilo escritor (GREEN)

#### Feature 2.3: Retención de notas
- [ ] Test: `retention_purges_notes_keeps_sessions` (RED) → mantenimiento (GREEN)

---

## Fase 3 — Actividades, proyectos, tags, agenda y cola ⏳

**Objetivo:** CRUD de actividades y armado automático de la cola del día.

### Scope

- CRUD de `Activity` / `Project` / `Tag`; archivar y completar.
- `ScheduleRule`: lunes–viernes, días específicos, toda la semana, rango de fechas.
- Cola del día: agenda + pendientes; siguiente actividad al completar.
- Tiempo manual por actividad (alta, edición, borrado).

### Criterio de salida

- [ ] Una actividad sin agenda es válida y no entra en la cola.
- [ ] La cola del día respeta las 4 formas de agenda.
- [ ] Completar una actividad la saca de la cola y ofrece la siguiente.

### Features (TDD)

- [ ] `schedule_weekdays_matches_only_weekdays` (RED → GREEN)
- [ ] `schedule_range_excludes_dates_outside` (RED → GREEN)
- [ ] `queue_builds_from_schedule_and_pending` (RED → GREEN)
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

### Criterio de salida

- [ ] Handshake compatible; mismatch devuelve error tipado.
- [ ] Los comandos de control llegan al motor y responden `ok`/`error`.

### Features (TDD)

- [ ] `ipc_handshake_negotiates_version` (RED → GREEN)
- [ ] `ipc_version_mismatch_returns_error` (RED → GREEN)
- [ ] `ipc_server_routes_session_start` (RED → GREEN)
- [ ] `ipc_client_sends_and_parses` (RED → GREEN)

---

## Fase 7 — TUI ⏳

**Objetivo:** interfaz Ratatui eficiente y visualmente atractiva.

### Scope

- Vistas: `[H]oy` (cola + timer mini) · `[T]imer` · `[M]étricas` · `[A]ctividades` · `[I]historial` · `[C]onfig`.
- **Timer:** layout de foco a pantalla completa, **beacon permanente** en todas las vistas, `lock_focus_view` opt-in (3 capas, `ADR-004`/`ADR-008`).
- **Historial:** notas y puntajes agrupados por fecha, filtrables por actividad y rating.
- Modales: reflexión (rating 1–10 + notas, ambos opcionales), challenge, cooldown visible.
- Acción para apagar el daemon desde la TUI.
- Widgets propios: heatmap y gauge circular.

### Criterio de salida

- [ ] La TUI navega y renderiza con datos fake y con el daemon real.
- [ ] El beacon muestra el tiempo restante en toda vista durante una sesión.
- [ ] Los modales de reflexión permiten guardar solo nota, solo rating, ambos o nada.

### Features (TDD)

- [ ] `app_creates_main_window` (RED → GREEN)
- [ ] `timer_view_shows_beacon_and_blocks_only_if_configured` (RED → GREEN)
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
- **Focus Quality** (promedio de rating) por actividad y franja horaria.
- Desglose por `source` (sesión / manual / importado) sobre el total.
- `tpt-cli report --range daily|weekly|monthly --format json|csv`.

### Criterio de salida

- [ ] Las métricas cuadran contra un dataset de prueba conocido.
- [ ] Los dos denominadores (día trabajado / día calendario) se muestran por separado.
- [ ] El reporte JSON/CSV se genera sin abrir la TUI.

### Features (TDD)

- [ ] `analytics_daily_totals_from_sessions_and_entries` (RED → GREEN)
- [ ] `analytics_averages_worked_day_and_calendar` (RED → GREEN)
- [ ] `focus_quality_averages_rating_by_activity` (RED → GREEN)
- [ ] `heatmap_buckets_by_day` (RED → GREEN)
- [ ] `cli_report_weekly_json_valid` (RED → GREEN)

---

## Fase 9 — Rutinas, rachas e import ⏳

**Objetivo:** rachas, consistencia e import de Super Productivity.

### Scope

- Racha con política *never miss twice*; sesiones abortadas no acreditan.
- Consistencia mensual de rutinas.
- `tpt-cli import --super-productivity save.json`: validación, `--dry-run`, idempotencia por `external_id`.
- Mapeo: `project` → `Project`, `task` → `Activity`, `timeEstimate` → `target_minutes`, `timeSpentOnDay` → `TimeEntry{IMPORTED}`.

### Criterio de salida

- [ ] La racha sobrevive a un día perdido y se corta a los dos.
- [ ] Import idempotente: correrlo dos veces no duplica.
- [ ] `--dry-run` reporta el mapeo sin escribir.

### Features (TDD)

- [ ] `streak_survives_one_missed_day` (RED → GREEN)
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

### Criterio de salida

- [ ] Suite completa verde; cobertura >80% unit, >60% integración.
- [ ] Instalación limpia en Arch (daemon + TUI + CLI).
- [ ] `README.md` y `docs/` finalizados.

### Features (TDD)

- [ ] `daemon_down_client_degrades_gracefully` (RED → GREEN)
- [ ] `config_corrupt_falls_back_to_defaults` (RED → GREEN)
- [ ] Unit systemd + PKGBUILD
- [ ] Verificación en disco real (nodatacow + retención)

# Fase 2: Store SQLite — Log

> Log HISTÓRICO de la fase. Se acumula, no se borra.
> Estado actual → `docs/handoff.md` · Conceptos → `docs/learning/phase-02-store.md`

## Estado

**Estado**: Completada (código, verificación y docs; PR `dev → main` pendiente)
**Última Actualización**: 2026-09-23

## Objetivos

- Implementar el adaptador `SqliteStore` que cumpla el puerto `Store` de `tpt-core`.
- Esquema de `ADR-002` con WAL, migración versionada y `auto_vacuum` efectivo.
- Hilo escritor MPSC que no bloquee el loop de sesión.
- Retención de notas y deuda calculada (crecimiento **y** vencimiento) sin jobs.

## Progreso

- [x] Feature 2.1 — Esquema y apertura (2026-09-23)
- [x] Feature 2.2 — Hilo escritor MPSC (2026-09-23)
- [x] Feature 2.3 — Retención de notas (2026-09-23)
- [x] Feature 2.4 — Deuda calculada (2026-09-23)
- [x] Feature 2.5 — Penalización negativa (2026-09-23)
- [x] Corrección post-verificación (2026-09-23)
- [x] `chattr +C` validado en disco real (2026-09-23)

## Tareas Completadas

### 2026-09-23 — Feature 2.1: Esquema y apertura (`7c6fe71`)

- **Descripción**: `SqliteStore::open` con resolución de path (`~`, `$XDG_DATA_HOME`), `create_dir_all`, pragmas verificados y migración transaccional. `SqliteStore` todavía **no** implementa el puerto: solo abre y crea el esquema, para que cada test RED tenga su razón de ser.
- **Archivos**: `crates/tpt-daemon/src/adapters/sqlite/{mod,schema}.rs`, `crates/tpt-daemon/Cargo.toml` (`rusqlite` bundled), `Cargo.lock`.
- **Tests**: `store_creates_schema_and_indexes` (10 tablas, 9 índices, `user_version=1`, `journal_mode=wal`, `foreign_keys=on`), `resolve_db_path_*`.
- **Decisión**: el DDL + `PRAGMA user_version` van en **una transacción** (`BEGIN IMMEDIATE` … `COMMIT`); sin eso, un crash deja un esquema a medias y los `CREATE TABLE` sin `IF NOT EXISTS` vuelven la DB inabrible.

### 2026-09-23 — Feature 2.2: Hilo escritor (`39b7fa7`)

- **Descripción**: `writer.rs` (hilo + `WriteCommand` + `sync_channel(1024)`) y `mapping.rs` (fila ↔ dominio). `impl Store` completo sobre el handle: `save_*` asigna el id y encola **sin I/O**; el hilo aplica el SQL en serie.
- **Archivos**: `crates/tpt-daemon/src/adapters/sqlite/{mod,mapping,writer}.rs`, `crates/tpt-core/src/{ports/store.rs,testing/mod.rs}`.
- **Tests**: `store_writer_persists_session_without_blocking` (seam `BlockUntil` + `recv_timeout`), `store_does_not_reuse_ids_after_explicit_id_save`, `store_returns_backend_when_writer_thread_is_dead`, `checkpoint_truncates_wal`, `store_rejects_unknown_enum_text`, `store_rejects_future_schema_version`, round-trips de las 6 entidades.
- **Decisión**: el puerto gana **solo** `flush()`. `save_*` conserva `Result<i64, _>`: el id se asigna en el cliente, así que devolverlo no obliga a esperar al disco. `flush()` y `checkpoint()` son operaciones distintas — el primero es barrera de commit, el segundo es mantenimiento.

### 2026-09-23 — Feature 2.3: Retención de notas (`eca7ecb`)

- **Descripción**: `purge_expired_notes(days)` enrutado por el hilo escritor (`WriteCommand::Purge`), seguido de `PRAGMA incremental_vacuum`.
- **Archivos**: `crates/tpt-daemon/src/adapters/sqlite/retention.rs`.
- **Tests**: `retention_purges_notes_keeps_sessions`, `retention_with_zero_days_is_a_no_op`.
- **Decisión**: la retención **anula** `time_entries.note` y `sessions.reflection`; nunca borra filas. Corte local para `day` (`date('now','localtime',…)`) y UTC para `started_at` (`datetime('now',…)`).

### 2026-09-23 — Feature 2.4: Deuda calculada (`7e82570`)

- **Descripción**: `Debt::outstanding_seconds(now)` y `Debt::is_expired(now)` en el dominio, sin job de mantenimiento.
- **Archivos**: `crates/tpt-core/src/domain/debt.rs`.
- **Tests**: `debt_amount_is_computed_from_created_at`, `debt_stops_growing_at_due_at`, `debt_base_is_capped`, `debt_amount_subtracts_paid_seconds`, `debt_amount_never_goes_below_zero`, `closed_debt_has_no_outstanding_amount`, `debt_expires_at_due_at`.
- **Decisión**: el crecimiento diario va **fuera** del tope y se detiene en `due_at` (ver `docs/learning/phase-02-store.md`).

### 2026-09-23 — Feature 2.5: Penalización negativa (`13aacde`)

- **Descripción**: verificación de que una entrada `PENALTY` con `seconds` negativo persiste y de que el `CHECK` de la DB rechaza negativos no-penalty.
- **Archivos**: `crates/tpt-daemon/src/adapters/sqlite/mod.rs`.
- **Tests**: `penalty_entry_accepts_negative_seconds`, `time_entries_check_rejects_negative_non_penalty_seconds`.

### 2026-09-23 — Corrección post-verificación (`565f58e`)

- **Descripción**: la verificación independiente encontró que el fake `InMemoryStore` violaba el invariante de ids que dice modelar (`save(Some(1))` + `save(None)` devolvía `1` y pisaba el registro) y que devolvía el contrato activo **primero** en vez del **último**. También faltaba cobertura del rechazo de `TimeEntry::new` y de JSON inválido.
- **Archivos**: `crates/tpt-core/src/{testing/mod.rs,domain/time_entry.rs,ports/store.rs}`, `crates/tpt-daemon/src/adapters/sqlite/mod.rs`.
- **Tests**: paridad del fake (2), `time_entry_rejects_negative_seconds_outside_penalty`, `penalty_time_entry_accepts_negative_seconds`, `store_rejects_malformed_schedule_rule_json`; se reforzó el test del `CHECK` (cuenta de filas) y se corrigió la doc de `flush()` (commit en WAL ≠ fsync).

### 2026-09-23 — `chattr +C` en disco real

- **Descripción**: validación del ítem de scope de la fase en el BTRFS real.
- **Evidencia**: `lsattr -d ~/.local/share/tpt` y `lsattr ~/.local/share/tpt/metrics.db` devuelven `---------------C------` (atributo NOCOW) en el directorio **y** en el archivo.

## Decisiones

1. **Evaluación dual adversarial del plan ANTES de codear** — 2 jueces ciegos en paralelo (`jd-judge-a`/`jd-judge-b`) sobre el plan congelado. Encontraron 3 CRITICAL (fórmula de deuda, `chattr +C` diferido, propagación de identidad del puerto) y el re-juece acotado detectó 2 defectos que el **propio fix** había introducido (deriva del contador de ids, crecimiento sin tope en `due_at`). Todo se corrigió antes de escribir código.
2. **Ids asignados en el cliente** — `save_*` devuelve el id sin I/O; contador por tabla sembrado con `MAX(id)` y regla `counter = max(counter, id_explicito)` en todo save. Supuesto: un solo proceso escritor.
3. **`flush()` y `checkpoint()` separados** — `flush()` espera el commit; `wal_checkpoint(TRUNCATE)` (verificando el flag `busy`) + `incremental_vacuum` es mantenimiento explícito, no algo que deba pagar cada lectura.
4. **Canal acotado (`sync_channel(1024)`) con contrapresión** — el no-bloqueo no puede comprarse con memoria ilimitada.
5. **El writer no cortocircuita** — registra el primer error y sigue; `flush()` lo reporta y lo limpia.
6. **La deuda se calcula, no se escribe** — sin job diario; crecimiento fuera del tope y clampeado en `due_at`.
7. **`insert` vs `update` explícito, nunca `INSERT OR REPLACE`** — con `foreign_keys=ON`, `OR REPLACE` borra la fila conflictiva y dispara `ON DELETE CASCADE`/`SET NULL` sobre gaps, additions y entradas ligadas a sesión.

## Problemas

1. **La feature 2.5 no tuvo RED legítimo** — el mapeo genérico de 2.2 ya persistía `PENALTY` negativa, así que el test pasó de entrada. Quedó como *test de caracterización* (bloquea el comportamiento) y el hueco real que el verificador detectó —`TimeEntry::new` rechazando negativos no-penalty, sin cobertura— se cerró en `565f58e`.
2. **`origin/dev` quedó con los hashes pre-rebase** de la PR de Fase 1 (mismo contenido, commits distintos). Requiere un `git push --force-with-lease` único antes de la PR.
3. **`rusqlite` con `bundled`** compila SQLite desde fuente: alarga el primer build, pero evita depender del SQLite del sistema.
4. **Un camino de error exigió `PRAGMA ignore_check_constraints`** en el test de enum corrupto: el `CHECK` del esquema rechazaba el valor inválido antes de que el mapeo pudiera verlo (buena señal, no un workaround).

## Métricas

- Tests en `tpt-core`: 27 (16 al cerrar Fase 1)
- Tests en `tpt-daemon`: 24 (1 al cerrar Fase 1; 1 `#[ignore]` de disco real)
- Tests pasando: 100% (`cargo test --workspace --all-targets --locked`)
- Cobertura: no medida (se mide en Fase 10)

## Pendientes

- **PR `dev → main`** de cierre de la fase.
- `session_additions` se crea pero el puerto no la expone → deuda para Fase 4 (contrato aditivo).
- El seed de ids es `MAX(id)`: revisar si alguna fase agrega un `DELETE` de filas.
- El test de disco real no asserta BTRFS: la evidencia del ítem de scope es el `lsattr`.
- `:memory:` no soportado (dos conexiones verían DBs distintas) — documentado.

# Handoff — Continuidad

> ⚠️ **ESTADO MUTABLE.** Se SOBREESCRIBE al cerrar sesión. No es historial.
> Historial por fase → `docs/progress-log/`. Conceptos → `docs/learning/`.
> Aprendizajes y decisiones persistentes → Engram (memoria).

**Última actualización**: 2026-09-21

---

## ⏭️ CONTINUAR ACÁ

**Fase 1 cerrada en `dev`** (features 1.1–1.5, 6 commits). Falta abrir la PR `dev → main` (ver MÓDULO: Git).

**Siguiente: Fase 2 — Store SQLite.**

### Orden de trabajo (Fase 2)

| # | Feature | Test RED (nombre exacto) | Archivo destino |
|---|---------|--------------------------|-----------------|
| 2.1 | Esquema e inicialización | `store_creates_schema_and_indexes` | `crates/tpt-daemon/src/adapters/` |
| 2.2 | Hilo escritor MPSC | `store_writer_persists_session_without_blocking` | `crates/tpt-daemon/src/adapters/` |
| 2.3 | Retención de notas | `retention_purges_notes_keeps_sessions` | `crates/tpt-daemon/src/adapters/` |
| 2.4 | Deuda por cálculo | `debt_amount_is_computed_from_created_at` | `crates/tpt-daemon/src/adapters/` |
| 2.5 | Penalización negativa | `penalty_entry_accepts_negative_seconds` | `crates/tpt-daemon/src/adapters/` |

Antes de codear: crear `docs/learning/phase-02-store.md` y `docs/progress-log/phase-02-store.md`.

### Reglas que no se negocian

- TDD estricto: **test primero**, `cargo test` en RED por la razón correcta, después GREEN.
- Puerta: `cargo fmt --all --check` + `cargo clippy --workspace --all-targets --locked -- -D warnings` + `cargo test`.
- Nada de `unwrap`/`expect`/`panic` en `src/`. En tests, `#![cfg_attr(test, allow(...))]`.
- Cada feature cierra con **su commit** convencional en inglés, sin atribución IA.

### Especificación de referencia (leer antes de codear)

- `docs/adr/ADR-002-sqlite-schema.md` → esquema completo, `CHECK`s, índices y reglas de datos.
- `docs/adr/ADR-001-architecture-ipc-persistence.md` + `docs/architecture.md` (Persistencia y BTRFS) → `chattr +C`, WAL, checkpoint, `auto_vacuum`.
- `docs/adr/ADR-003-hexagonal-ports.md` → `Store` es el puerto a implementar (`SqliteStore`); los fakes viven en `tpt-core::testing`.
- `docs/adr/ADR-004-clock-modes-and-penalties.md` §6 → qué se calcula (crecimiento y vencimiento de deuda) en vez de persistirse.
- `docs/phases.md` (Fase 2) → scope, criterios de salida y features.
- `docs/learning/phase-01-domain.md` → decisiones que el adaptador debe respetar (timestamps `i64`, enums `SCREAMING_SNAKE_CASE`).

### Cierre de fase

Al terminar 2.1–2.5: actualizar `docs/learning/phase-02-store.md`, `docs/progress-log/phase-02-store.md`, `docs/handoff.md`, `docs/phases.md` y abrir PR `dev → main`.

---

## MÓDULO: Estado actual

| | |
|---|---|
| **Fase activa** | Fase 2 — Store SQLite |
| **Última completada** | Fase 1 — Dominio, config y puertos (código en `dev`; PR a `main` pendiente) |
| **Progreso** | Fase 1 ✅: features 1.0–1.5 cerradas, 16 tests en `tpt-core`, fmt/clippy/test verdes. Fase 2 sin comenzar. |

## MÓDULO: Pivote de alcance (2026-09)

La app dejó de ser un **monitor pasivo de ventanas** y pasó a ser un **gestor de actividades con contrato de tiempo** (`ADR-007`).

- **Se retiró:** tracking de ventanas (Hyprland `socket2`), `$PWD` vía `/proc`, reglas regex, categorías, blocklist de apps, tiers de retención.
- **Se conservó:** modos de reloj, métricas, rutinas, import de Super Productivity, arquitectura de 4 crates.
- **Se agregó:** presencia, contrato aditivo, niveles de estrictez, deuda de reparación, refinanciación, tiempo manual etiquetado, reflexión opcional, agenda/cola de actividades.

## MÓDULO: Decisiones pendientes

- Fórmula de **Focus Quality** (promedio de rating normalizado por `rating_scale`) → Fase 8.
- Umbral de **"día trabajado"** → Fase 8.
- Definición fina de la **racha** (≥1 ciclo o ≥X min acreditados) → Fase 9.
- ~~Niveles de estrictez y default~~ ✅ `L0` default; `L1`/`L2` presets fijos.
- ~~Challenge y cooldown~~ ✅ deuda de reparación + refinanciación.
- ~~Fuente de presencia~~ ✅ hooks de quickshell (ver abajo).

## MÓDULO: Entorno de presencia (auditado 2026-09-17)

**El lock y el idle los maneja `caelestia-shell` (quickshell)**, no `hypridle` (instalado, **sin configurar ni correr**) ni `hyprlock`.

`~/.config/caelestia/shell.json`:

| Idle | Acción |
|---|---|
| 5 min (300 s) | bloqueo de pantalla — **TPT lo ignora** |
| **7 min (420 s)** | hook TPT: `tpt-cli presence --state idle` |
| 12 min (720 s) | `dpms off` — **señal severa de TPT** + hook `screen-off` |
| 15 min (900 s) | `suspend` |

- `inhibitWhenAudio` **global en `false`**, con `inhibitWhenAudio: true` por entrada en lock/dpms/suspend. Si el global vuelve a `true`, **el hook de TPT se apaga cuando suena audio**.
- ⚠️ `tpt-cli` **todavía no existe**: hasta la Fase 6 los hooks fallan silenciosamente en el log del shell.
- El lock **no** inhibe el idle (verificado): los hooks siguen disparando con la pantalla bloqueada.

## MÓDULO: Riesgos activos / Gotchas

- ⚠️ **Hibernación no disponible**: el swap es **zram (4 GB)**, que no puede contener una imagen de hibernación. Por eso la acción de suspensión se cambió a `suspend` plano.
- ⚠️ **logind `IdleHint`/`LockedHint` inservibles**: `CanIdle=yes` y `CanLock=yes`, pero **nadie llama** `SetIdleHint()`/`SetLockedHint()` (quickshell no habla D-Bus). Verificado con `loginctl show-session`.
- ⚠️ **El compromiso es autoimpuesto**: sin tracking de apps, el escape real (navegador, celular) no está cubierto. Decisión consciente (`ADR-007`), no un pendiente.
- ⚠️ **BTRFS + SQLite**: sin `chattr +C` en el directorio de la DB hay write amplification por CoW. Aplicar **antes** de crear la DB.
- ⚠️ **Bajo consumo de memoria**: no agregar tokio ni dependencias pesadas. Por eso se descartó D-Bus (`ADR-008`).
- ℹ️ **Scope `workflow` de GitHub**: el token OAuth de `gh` no puede pushear **ni mergear** PRs que toquen `.github/workflows/`. Usar SSH, o un PAT con permiso Workflows (`ADR-006`). La PR #2 ya se cerró; el gotcha sigue vigente para cualquier PR que toque el CI.
- ℹ️ **`thiserror` y el campo `source`**: cualquier campo con ese nombre se trata como la causa del error. Por eso `TimeEntrySource` implementa `Error`. Si aparece otro error de dominio con un campo `source`, renombrarlo antes de pelear con la macro.
- ℹ️ **`include_str!("../../../../config.toml")`** cruza el package root de `tpt-core`: `cargo package` deja el TOML afuera y el `.crate` no compila sus tests. Resolver en Fase 10 (packaging).

## MÓDULO: Entorno de desarrollo

- OS: Arch Linux (Wayland/Hyprland) con `caelestia-shell` (quickshell).
- Toolchain: Rust **1.98.1** del paquete de Arch (**sin `rustup`**); `rustfmt` y `clippy` incluidos.
- Edition del proyecto: **2024** (resolver 3), MSRV **1.85**.
- Comandos: `cargo build` · `cargo test` · `cargo clippy --all-targets -- -D warnings`.
- Convención: código en inglés, docs en español.
- Editor: Neovim (LazyVim) con `rust-analyzer` (Mason) + extra `lang.rust`.

## MÓDULO: Git

- Remoto: `origin` → `github.com:ails-w/tpo-cro` (HTTPS; la llave SSH está registrada pero tiene passphrase).
- Flujo: `feat/* → dev → main` (`ADR-006`). **Fase 1 se trabajó directo sobre `dev`** a pedido del usuario.
- **Sin PRs abiertas.** `main` y `dev` venían del mismo commit (`2f56409`); la Fase 1 agregó 6 commits a `dev`.
- **Pendiente inmediato**: abrir PR `dev → main` con el cierre de Fase 1.
- Ramas: `dev` (trabajo), `main` (estable).

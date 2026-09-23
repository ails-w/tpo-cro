# Handoff — Continuidad

> ⚠️ **ESTADO MUTABLE.** Se SOBREESCRIBE al cerrar sesión. No es historial.
> Historial por fase → `docs/progress-log/`. Conceptos → `docs/learning/`.
> Aprendizajes y decisiones persistentes → Engram (memoria).

**Última actualización**: 2026-09-23

---

## ⏭️ CONTINUAR ACÁ

**Fase 2 cerrada** (features 2.1–2.5 + corrección + docs, 7 commits). **PR #5 abierta** (`feat/phase-2-store → main`), CI verde — ver MÓDULO: Git.

**Siguiente: Fase 3 — Actividades, proyectos, tags, agenda y cola.**

### Orden de trabajo (Fase 3)

| # | Feature | Test RED (nombre exacto) | Archivo destino |
|---|---------|--------------------------|-----------------|
| 3.1 | Recurrencia L-V | `schedule_weekdays_matches_only_weekdays` | `crates/tpt-core/src/domain/schedule.rs` |
| 3.2 | Rango de fechas | `schedule_range_excludes_dates_outside` | `crates/tpt-core/src/domain/schedule.rs` |
| 3.3 | Cola del día | `queue_builds_from_schedule_and_pending` | `crates/tpt-core/src/domain/` + consultas en el store |
| 3.4 | Manual rechazado en TIMER | `manual_entry_rejected_when_tracking_mode_is_timer` | `crates/tpt-core/src/domain/` |
| 3.5 | Manual suma al total | `manual_entry_is_marked_and_sums_to_total` | `crates/tpt-core/src/domain/` |

Antes de codear: crear `docs/learning/phase-03-activities.md` y `docs/progress-log/phase-03-activities.md`.

**Deuda técnica que esta fase probablemente salde**: `Store` no expone `projects` ni `tags` (las tablas existen desde la Fase 2); el CRUD los necesita.

### Reglas que no se negocian

- TDD estricto: **test primero**, `cargo test` en RED por la razón correcta, después GREEN.
- Puerta: `cargo fmt --all --check` + `cargo clippy --workspace --all-targets --locked -- -D warnings` + `cargo test`.
- Nada de `unwrap`/`expect`/`panic` en `src/`. En tests, `#![cfg_attr(test, allow(...))]`.
- Cada feature cierra con **su commit** convencional en inglés, sin atribución IA.

### Especificación de referencia (leer antes de codear)

- `docs/phases.md` (Fase 3) → scope, criterios de salida y features.
- `docs/adr/ADR-004-clock-modes-and-penalties.md` §9 → `timer` vs `manual`: la única diferencia es si se puede cargar tiempo a mano.
- `docs/adr/ADR-002-sqlite-schema.md` → `activities`, `projects`, `tags`, `activity_tags`, `schedule_rule` (JSON), `pending_extra_ratio`; el `CHECK` de `time_entries` ya limita el signo.
- `docs/adr/ADR-003-hexagonal-ports.md` → `Store` es el puerto a extender (proyectos y tags todavía no están expuestos).
- `docs/learning/phase-02-store.md` → cómo escribir en el store: `save_*` asigna el id sin I/O, `flush()` antes de leer, nunca `INSERT OR REPLACE`.
- `docs/learning/phase-01-domain.md` → modelos ya definidos (`ScheduleRule` tiene las 4 formas pero **sin comportamiento**: la Fase 3 lo implementa).

### Cierre de fase

Al terminar 3.1–3.5: actualizar `docs/learning/phase-03-activities.md`, `docs/progress-log/phase-03-activities.md`, `docs/handoff.md`, `docs/phases.md` y abrir PR `dev → main`.

---

## MÓDULO: Estado actual

| | |
|---|---|
| **Fase activa** | Fase 3 — Actividades, proyectos, tags, agenda y cola |
| **Última completada** | Fase 2 — Store SQLite (PR #5 abierta hacia `main`, CI verde) |
| **Progreso** | Fase 2 ✅: features 2.1–2.5 + corrección. 27 tests en `tpt-core`, 24 en `tpt-daemon` (1 `#[ignore]` de disco real). fmt/clippy/test verdes. Fase 3 sin comenzar. |

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
- ⚠️ **BTRFS + SQLite**: sin `chattr +C` en el directorio de la DB hay write amplification por CoW. Aplicar **antes** de crear la DB. ✅ Ya aplicado y verificado: `~/.local/share/tpt` y `metrics.db` tienen el atributo `C`.
- ⚠️ **`auto_vacuum` va antes del primer `CREATE TABLE`**: si la DB ya tiene tablas, cambiarlo exige un `VACUUM` completo, prohibido en BTRFS.
- ⚠️ **`flush()` no es `fsync`**: con `synchronous=NORMAL` garantiza commit en WAL, no supervivencia a un corte de energía. No leerle más de lo que promete.
- ℹ️ **Ids asignados en el cliente**: `save_*` devuelve el id sin tocar el disco y el contador por tabla avanza con `max(counter, id_explicito)`. Supuesto: **un solo proceso escritor** (el daemon). Si alguna fase agrega `DELETE`, revisar el seed `MAX(id)`.
- ℹ️ **`:memory:` no está soportado**: el store abre dos conexiones (lectura y escritura) y verían dos bases distintas. Los tests usan archivos temporales.
- ℹ️ **`rusqlite` con `bundled`**: compila SQLite desde fuente. Alarga el primer build, pero no depende del SQLite del sistema.
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
- Flujo: `feat/* → dev → main` (`ADR-006`). **Las Fases 1 y 2 se trabajaron directo sobre `dev`** a pedido del usuario.
- **La PR de Fase 1 se mergeó con rebase**: `main` tiene los mismos 7 commits con hashes nuevos. Por eso `origin/dev` quedó con los hashes viejos, divergente de `main`.
- ⚠️ **`dev` no acepta force-push** (regla del repo, verificada). Consecuencia: la Fase 2 se subió como rama `feat/phase-2-store` y su PR va **directo a `main`** (`feat/* → main`).
- **PR #5 abierta**: `feat/phase-2-store → main`, 17 archivos / +3206 / −93, CI verde, mergeable.
- **Deuda de ramas**: `origin/dev` sigue en `7ce1c15` (hashes pre-rebase, contenido ya presente en `main`). Opciones: (a) habilitar temporalmente el force-push y alinearla con `main`; (b) mergear `main` en `dev` (deja un merge commit y duplica commits en el historial); (c) dejarla y trabajar con `feat/*` por fase. **Decisión pendiente del usuario.**
- Ramas: `dev` (trabajo), `main` (estable), `feat/phase-2-store` (esta fase).

# Handoff — Continuidad

> ⚠️ **ESTADO MUTABLE.** Se SOBREESCRIBE al cerrar sesión. No es historial.
> Historial por fase → `docs/progress-log/`. Conceptos → `docs/learning/`.
> Aprendizajes y decisiones persistentes → Engram (memoria).

**Última actualización**: 2026-09-17

---

## ⏭️ CONTINUAR ACÁ

**Fase 1 — Features 1.1 a 1.5** (el pivote documental, Feature 1.0, ya está cerrado).

### Orden de trabajo

| # | Feature | Test RED (nombre exacto) | Archivo destino |
|---|---------|--------------------------|-----------------|
| 1.1 | Configuración TOML | `app_config_parse_valid_toml_returns_expected` | `crates/tpt-core/src/config/` |
| 1.2 | Validación de presets | `timer_preset_short_break_over_15_rejected` | `crates/tpt-core/src/config/` |
| 1.3 | Actividades y agenda | `activity_without_schedule_is_valid` · `activity_requires_tracking_mode` | `crates/tpt-core/src/domain/` |
| 1.4 | Sesiones y entradas | `session_starts_running_with_activity` · `manual_time_entry_adds_to_activity_total` | `crates/tpt-core/src/domain/` |
| 1.5 | Puertos | `ports_are_object_safe` | `crates/tpt-core/src/ports/` |

### Reglas que no se negocian

- TDD estricto: **test primero**, `cargo test` en RED por la razón correcta, después GREEN.
- Puerta: `cargo fmt --all --check` + `cargo clippy --workspace --all-targets --locked -- -D warnings` + `cargo test`.
- Nada de `unwrap`/`expect`/`panic` en `src/`. En tests, `#![cfg_attr(test, allow(...))]`.
- Cada feature cierra con **su commit** convencional en inglés, sin atribución IA.

### Especificación de referencia (leer antes de codear)

- `docs/adr/ADR-004-clock-modes-and-penalties.md` → contrato, niveles, deuda, refinanciación.
- `docs/adr/ADR-002-sqlite-schema.md` → campos que los modelos deben poder representar.
- `docs/adr/ADR-003-hexagonal-ports.md` → los 6 puertos.
- `docs/phases.md` (Fase 1) → scope, criterios de salida y features.
- `docs/learning/phase-01-domain.md` → los 3 conceptos de la fase.

### Cierre de fase

Al terminar 1.1–1.5: actualizar `docs/learning/phase-01-domain.md`, `docs/progress-log/phase-01-domain.md`, `docs/handoff.md`, `docs/phases.md` y abrir PR `dev → main`.

---

## MÓDULO: Estado actual

| | |
|---|---|
| **Fase activa** | Fase 1 — Pivote documental + dominio, config y puertos |
| **Última completada** | Fase 0 — Setup (workspace Rust + CI) |
| **Progreso** | Feature 1.0 ✅ (pivote, contrato y presencia cerrados). Features 1.1–1.5 pendientes. |

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
- ℹ️ **Scope `workflow` de GitHub**: el token OAuth de `gh` no puede pushear **ni mergear** PRs que toquen `.github/workflows/`. Usar SSH, o un PAT con permiso Workflows (`ADR-006`). **La PR #2 está bloqueada por esto.**

## MÓDULO: Entorno de desarrollo

- OS: Arch Linux (Wayland/Hyprland) con `caelestia-shell` (quickshell).
- Toolchain: Rust **1.98.1** del paquete de Arch (**sin `rustup`**); `rustfmt` y `clippy` incluidos.
- Edition del proyecto: **2024** (resolver 3), MSRV **1.85**.
- Comandos: `cargo build` · `cargo test` · `cargo clippy --all-targets -- -D warnings`.
- Convención: código en inglés, docs en español.
- Editor: Neovim (LazyVim) con `rust-analyzer` (Mason) + extra `lang.rust`.

## MÓDULO: Git

- Remoto: `origin` → `github.com:ails-w/tpo-cro` (HTTPS; la llave SSH está registrada pero tiene passphrase).
- Flujo: `feat/* → dev → main` (`ADR-006`).
- **PR #2 abierta** (`dev → main`): CI verde y mergeable, **bloqueada por el scope `workflow`**.
- Ramas: `dev` (trabajo), `main` (estable).

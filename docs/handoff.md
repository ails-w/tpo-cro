# Handoff — Continuidad

> ⚠️ **ESTADO MUTABLE.** Se SOBREESCRIBE al cerrar sesión. No es historial.
> Historial por fase → `docs/progress-log/`. Conceptos → `docs/learning/`.
> Aprendizajes y decisiones persistentes → Engram (memoria).

**Última actualización**: 2026-09-14

## Estado actual

| | |
|---|---|
| **Fase activa** | — (Fase 1 es la próxima) |
| **Última completada** | Fase 0 — Setup (workspace Rust + CI) |
| **Progreso** | Workspace de 4 crates compilando, 4 smoke tests verdes, CI definido y verificado localmente |

## Próximo paso

Fase 1 — Feature 1.1: modelos de configuración. Test RED `app_config_parse_valid_toml_returns_expected`, luego modelos + parsing TOML con defaults (GREEN) en `crates/tpt-core/src/config/`.

## Decisiones consolidadas (D1–D20)

Ver `docs/vision.md` (tabla D1–D20) y `docs/adr/`. Las críticas: D2 (hexagonal), D6–D11 (modos/penalizaciones), D12–D13 (inactividad/blocklist), D17–D18 (retención/BTRFS), D19 (presets). El modelo de tracking (3 capas) quedó explícito en `ADR-008-tracking-layers.md`.

## Decisiones pendientes

- Fórmula concreta del **Focus Score** (D16 lo deja configurable; definir pesos en Fase 7).
- Exacta definición de "día trabajado" (umbral de actividad > 0 min, configurable) en Fase 8.
- Presets default: confirmar `25/5`, `50/10`, `52/15`, `90/15(20)` como lista inicial.
- **Modelo de `Task`**: ¿solo rutinas recurrentes (`recurring_tasks` de ADR-002) o también tareas ad-hoc con su propia blocklist? Se define en Fase 1; la blocklist cuelga de ese modelo (ver `ADR-008`).

## Riesgos activos / Gotchas

- ⚠️ **BTRFS + SQLite**: sin `chattr +C` en el directorio de la DB hay write amplification por CoW. Aplicar antes de crear la DB.
- ⚠️ **Idle sin libwayland**: la integración `ext-idle-notify-v1` debe ir tras `IdleSource`; evaluar helper `hypridle` o lectura de eventos Hyprland con fallback polling.
- ⚠️ **Hyprland socket2**: ruta depende de `$HYPRLAND_INSTANCE_SIGNATURE`; manejar ausencia con degradación elegante.
- ⚠️ **Bajo consumo de memoria**: no agregar tokio ni dependencias pesadas al daemon; revisar cada crate en `Cargo.toml` y medir el consumo real antes de declarar límites.
- ℹ️ **Lints de "cero panics"**: `unwrap_used`/`expect_used`/`panic` están en `deny` a nivel workspace. En tests se exceptúan con `#![cfg_attr(test, allow(...))]`. Si un `tests/` de integración usa `.unwrap()`, hay que exceptuarlo por archivo.

## Entorno

- OS: Arch Linux (Wayland/Hyprland).
- Toolchain: Rust **1.98.1** del paquete de Arch (**sin `rustup`**); `rustfmt` 1.9.0 y `clippy` 0.1.98 incluidos.
- Edition del proyecto: **2024** (resolver 3), MSRV **1.85**.
- Build/test: `cargo build` / `cargo test`.
- Lint: `cargo clippy --all-targets -- -D warnings`.
- TDD estricto: tests ANTES de implementar.
- Convención: código en inglés, docs en español (bloques de código con `rust`).
- Editor: Neovim (LazyVim) con `rust-analyzer` (Mason) + extra `lang.rust`.

## Git

- Remoto: `origin` → `git@github.com:ails-w/tpo-cro.git` (SSH sin key registrada; usar el credential helper de `gh` para push).
- Flujo: commit por feature, PR por fase. Rama de Fase 0: `chore/phase-0-setup`.
